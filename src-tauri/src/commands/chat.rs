#![allow(
    clippy::needless_pass_by_value,
    reason = "Tauri command IPC deserializes owned arguments and State"
)]

use tauri::State;
use uuid::Uuid;
use zenpaws_database::{MessageCursor, MessageRecord};
use zenpaws_network::{Envelope, LamportClock, MessageBody, MessagePayload};

use crate::DatabaseState;

pub mod mutations;
mod types;

use types::{ChatCursor, ChatMessage};

/// Loads one descending keyset page of local room history.
#[tauri::command]
pub fn list_messages(
    database: State<'_, DatabaseState>,
    room: String,
    cursor: Option<ChatCursor>,
    limit: u32,
) -> Result<Vec<ChatMessage>, String> {
    let cursor = cursor
        .map(|cursor| {
            Ok::<MessageCursor, String>(MessageCursor {
                created_at: cursor.created_at,
                id: Uuid::parse_str(&cursor.id).map_err(|error| error.to_string())?,
            })
        })
        .transpose()?;
    let database = database
        .0
        .lock()
        .map_err(|_| "database lock poisoned".to_owned())?;
    let messages = database
        .messages_before(&room, cursor.as_ref(), limit)
        .map_err(|error| error.to_string())?;
    let result = messages
        .iter()
        .map(|message| project_message_for(&database, message))
        .collect();
    drop(database);
    result
}

/// Searches the local FTS5 message index.
#[tauri::command]
pub fn search_messages(
    database: State<'_, DatabaseState>,
    query: String,
    limit: u32,
) -> Result<Vec<ChatMessage>, String> {
    let database = database
        .0
        .lock()
        .map_err(|_| "database lock poisoned".to_owned())?;
    let messages = database
        .search_messages(&query, limit)
        .map_err(|error| error.to_string())?;
    let result = messages
        .iter()
        .map(|message| project_message_for(&database, message))
        .collect();
    drop(database);
    result
}

/// Persists one locally-authored message for the active peer identity.
#[tauri::command]
pub fn send_message(
    database: State<'_, DatabaseState>,
    room: String,
    body: String,
    reply_to: Option<String>,
) -> Result<ChatMessage, String> {
    if body.trim().is_empty() {
        return Err("message body cannot be empty".to_owned());
    }
    let author = super::network::local_peer_id()?;
    let created_at: i64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_millis()
        .try_into()
        .map_err(|_| "system timestamp exceeds i64".to_owned())?;
    let message = MessageRecord {
        author,
        body,
        created_at,
        id: Uuid::new_v4(),
        lamport_counter: created_at,
        lamport_peer: author,
        reply_to: reply_to
            .map(|value| Uuid::parse_str(&value).map_err(|error| error.to_string()))
            .transpose()?,
        room,
    };
    let database = database
        .0
        .lock()
        .map_err(|_| "database lock poisoned".to_owned())?;
    database
        .insert_message(&message)
        .map_err(|error| error.to_string())?;
    super::network::broadcast(Envelope::Message(MessagePayload {
        author,
        body: MessageBody::Text(message.body.clone()),
        clock: LamportClock {
            counter: message.lamport_counter,
            peer_id: message.lamport_peer,
        },
        created_at,
        id: message.id,
        mentions: Vec::new(),
        reply_to: message.reply_to,
        room: message.room.clone(),
    }))?;
    let result = project_message_for(&database, &message);
    drop(database);
    result
}

fn project_message_for(
    database: &zenpaws_database::Database,
    message: &MessageRecord,
) -> Result<ChatMessage, String> {
    let author_id = message.author.as_uuid().to_string();
    let author_name = database
        .peer_username(message.author)
        .map_err(|error| error.to_string())?
        .unwrap_or_else(|| author_id.clone());
    let reactions = database
        .reactions_for_message(message.id)
        .map_err(|error| error.to_string())?;
    Ok(types::project_message(message, author_name, reactions))
}
