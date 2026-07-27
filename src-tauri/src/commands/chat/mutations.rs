#![allow(
    clippy::needless_pass_by_value,
    reason = "Tauri command IPC deserializes owned arguments and State"
)]

use tauri::State;
use uuid::Uuid;
use zenpaws_database::MessageAckKind;
use zenpaws_network::{Envelope, LamportClock, MessageBody, MessagePayload};
use zenpaws_shared::{EventBus, ZenPawsEvent};

use super::types::ChatMessageStatus;
use crate::DatabaseState;
use crate::commands::network;

/// Adds a local peer reaction to a message.
#[tauri::command]
pub fn add_reaction(
    database: State<'_, DatabaseState>,
    message_id: String,
    emoji: String,
    room: String,
) -> Result<(), String> {
    if emoji.is_empty() || emoji.chars().count() > 16 {
        return Err("reaction emoji is invalid".to_owned());
    }
    let message_id = Uuid::parse_str(&message_id).map_err(|error| error.to_string())?;
    let peer_id = network::local_peer_id()?;
    database
        .0
        .lock()
        .map_err(|_| "database lock poisoned".to_owned())?
        .add_reaction(message_id, peer_id, &emoji)
        .map_err(|error| error.to_string())?;
    broadcast_mutation(peer_id, room, MessageBody::Reaction { message_id, emoji })
}

/// Publishes ephemeral typing state for the active local peer.
#[tauri::command]
pub fn set_typing(bus: State<'_, EventBus>, room: String, is_typing: bool) -> Result<(), String> {
    let peer_id = network::local_peer_id()?;
    bus.publish(ZenPawsEvent::TypingChanged {
        peer_id,
        room,
        is_typing,
    });
    Ok(())
}

/// Returns acknowledgement status for a direct-message recipient.
#[tauri::command]
pub fn message_status(
    database: State<'_, DatabaseState>,
    message_id: String,
    peer_id: String,
) -> Result<ChatMessageStatus, String> {
    let message_id = Uuid::parse_str(&message_id).map_err(|error| error.to_string())?;
    let peer_id = Uuid::parse_str(&peer_id).map_err(|error| error.to_string())?;
    let status = database
        .0
        .lock()
        .map_err(|_| "database lock poisoned".to_owned())?
        .message_status(message_id, zenpaws_shared::PeerId::from_uuid(peer_id))
        .map_err(|error| error.to_string())?;
    Ok(ChatMessageStatus {
        delivered: status.delivered,
        read: status.read,
    })
}

/// Records a direct-message read receipt for the local recipient.
#[tauri::command]
pub fn mark_message_read(
    database: State<'_, DatabaseState>,
    message_id: String,
    peer_id: String,
) -> Result<(), String> {
    let message_id = Uuid::parse_str(&message_id).map_err(|error| error.to_string())?;
    let peer_id = Uuid::parse_str(&peer_id).map_err(|error| error.to_string())?;
    let read_at: i64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_millis()
        .try_into()
        .map_err(|_| "system timestamp exceeds i64".to_owned())?;
    database
        .0
        .lock()
        .map_err(|_| "database lock poisoned".to_owned())?
        .acknowledge_message(
            message_id,
            zenpaws_shared::PeerId::from_uuid(peer_id),
            MessageAckKind::Read,
            read_at,
        )
        .map_err(|error| error.to_string())?;
    network::broadcast(Envelope::Ack {
        message_id,
        kind: zenpaws_network::AckKind::Read,
    })
}

/// Edits a locally visible message body.
#[tauri::command]
pub fn edit_message(
    database: State<'_, DatabaseState>,
    id: String,
    body: String,
    room: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&id).map_err(|error| error.to_string())?;
    let edited_at: i64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_millis()
        .try_into()
        .map_err(|_| "system timestamp exceeds i64".to_owned())?;
    database
        .0
        .lock()
        .map_err(|_| "database lock poisoned".to_owned())?
        .edit_message(id, &body, edited_at)
        .map_err(|error| error.to_string())?;
    let peer_id = network::local_peer_id()?;
    broadcast_mutation(
        peer_id,
        room,
        MessageBody::Edit {
            message_id: id,
            body,
        },
    )
}

/// Soft-deletes a locally visible message.
#[tauri::command]
pub fn delete_message(
    database: State<'_, DatabaseState>,
    id: String,
    room: String,
) -> Result<(), String> {
    let id = Uuid::parse_str(&id).map_err(|error| error.to_string())?;
    let deleted_at: i64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_millis()
        .try_into()
        .map_err(|_| "system timestamp exceeds i64".to_owned())?;
    database
        .0
        .lock()
        .map_err(|_| "database lock poisoned".to_owned())?
        .delete_message(id, deleted_at)
        .map_err(|error| error.to_string())?;
    let peer_id = network::local_peer_id()?;
    broadcast_mutation(peer_id, room, MessageBody::Delete { message_id: id })
}

fn broadcast_mutation(
    peer_id: zenpaws_shared::PeerId,
    room: String,
    body: MessageBody,
) -> Result<(), String> {
    let created_at: i64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_millis()
        .try_into()
        .map_err(|_| "system timestamp exceeds i64".to_owned())?;
    network::broadcast(Envelope::Message(MessagePayload {
        author: peer_id,
        body,
        clock: LamportClock {
            counter: created_at,
            peer_id,
        },
        created_at,
        id: Uuid::new_v4(),
        mentions: Vec::new(),
        reply_to: None,
        room,
    }))
}
