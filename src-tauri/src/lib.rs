//! `ZenPaws` Tauri application setup.
//!
//! This stays a thin shell: plugin registration and window/event wiring
//! only. Business logic lives in the `zenpaws-*` crates under `crates/`,
//! reached through `commands/`. See `docs/04_SYSTEM_ARCHITECTURE.md`.
//!
//! Only the plugins Phase 1 actually needs are registered here. See
//! `docs/05_TECHNICAL_DECISIONS.md`.

mod commands;

use std::sync::{Arc, Mutex};

use tauri::{Emitter, Manager};

/// Application-owned database state. Commands borrow it through Tauri state.
pub struct DatabaseState(pub Arc<Mutex<zenpaws_database::Database>>);

fn now_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| {
            duration.as_millis().try_into().unwrap_or(i64::MAX)
        })
}

fn persist_event(database: &zenpaws_database::Database, event: &zenpaws_shared::ZenPawsEvent) {
    match event {
        zenpaws_shared::ZenPawsEvent::MessageReceived {
            message_id,
            room,
            author,
            author_name,
            body,
            created_at,
            lamport_counter,
            reply_to,
        } => {
            let _ = database.update_peer_username(*author, author_name, *created_at);
            let _ = database.insert_message_if_absent(&zenpaws_database::MessageRecord {
                author: *author,
                body: body.clone(),
                created_at: *created_at,
                id: *message_id,
                lamport_counter: *lamport_counter,
                lamport_peer: *author,
                reply_to: *reply_to,
                room: room.clone(),
            });
        }
        zenpaws_shared::ZenPawsEvent::MessageEdited {
            message_id,
            body,
            edited_at,
            lamport_counter,
            lamport_peer,
        } => {
            let _ = database.apply_remote_edit(
                *message_id,
                body,
                *edited_at,
                zenpaws_database::LamportCursor {
                    counter: *lamport_counter,
                    peer_id: *lamport_peer,
                },
            );
        }
        zenpaws_shared::ZenPawsEvent::MessageDeleted {
            message_id,
            deleted_at,
            lamport_counter,
            lamport_peer,
        } => {
            let _ = database.apply_remote_delete(
                *message_id,
                *deleted_at,
                zenpaws_database::LamportCursor {
                    counter: *lamport_counter,
                    peer_id: *lamport_peer,
                },
            );
        }
        zenpaws_shared::ZenPawsEvent::MessageReactionChanged {
            message_id,
            peer_id,
            emoji,
            removed,
        } => {
            if *removed {
                let _ = database.remove_reaction(*message_id, *peer_id, emoji);
            } else {
                let _ = database.add_reaction(*message_id, *peer_id, emoji);
            }
        }
        zenpaws_shared::ZenPawsEvent::MessageAckReceived {
            message_id,
            peer_id,
            read,
        } => {
            let kind = if *read {
                zenpaws_database::MessageAckKind::Read
            } else {
                zenpaws_database::MessageAckKind::Delivered
            };
            let _ = database.acknowledge_message(*message_id, *peer_id, kind, now_millis());
        }
        _ => {}
    }
}

/// Runs the desktop application.
///
/// # Errors
///
/// Returns a `tauri::Error` if application setup or the runtime fails.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> tauri::Result<()> {
    tauri::Builder::default()
        .setup(|app| {
            let path = app.path().app_data_dir()?.join("zenpaws.sqlite");
            let database = Arc::new(Mutex::new(zenpaws_database::Database::open(path)?));
            app.manage(DatabaseState(Arc::clone(&database)));
            let bus = zenpaws_shared::EventBus::new(256);
            let mut events = bus.subscribe();
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                while let Ok(event) = events.recv().await {
                    if let zenpaws_shared::ZenPawsEvent::SyncRequested {
                        peer_id,
                        room,
                        since_counter,
                        since_peer,
                    } = &event
                    {
                        let _ = commands::network::handle_sync_request(
                            &database,
                            *peer_id,
                            room.clone(),
                            zenpaws_database::LamportCursor {
                                counter: *since_counter,
                                peer_id: *since_peer,
                            },
                        );
                    }
                    if let Ok(database) = database.lock() {
                        persist_event(&database, &event);
                    }
                    let _ = handle.emit("zenpaws://event", event);
                }
            });
            app.manage(bus);
            Ok(())
        })
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            commands::app_status,
            commands::network::start_network,
            commands::network::network_diagnostics,
            commands::network::stop_network,
            commands::pets::open_pet_window,
            commands::pets::close_pet_window,
            commands::chat::list_messages,
            commands::chat::search_messages,
            commands::chat::send_message,
            commands::chat::mutations::add_reaction,
            commands::chat::mutations::set_typing,
            commands::chat::mutations::message_status,
            commands::chat::mutations::mark_message_read,
            commands::chat::mutations::edit_message,
            commands::chat::mutations::delete_message
        ])
        .run(tauri::generate_context!())
}
