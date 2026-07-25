#![allow(
    clippy::needless_pass_by_value,
    reason = "Tauri command IPC deserializes owned arguments and State"
)]

use serde::Serialize;
use tauri::State;
use uuid::Uuid;
use zenpaws_database::{MessageCursor, MessageRecord};
use zenpaws_shared::{EventBus, ZenPawsEvent};

use crate::DatabaseState;

/// Frontend-safe message projection without database internals.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub author_id: String,
    pub body: String,
    pub created_at: i64,
    pub id: String,
    pub reply_to: Option<String>,
    pub room: String,
}

/// Keyset cursor accepted from the chat frontend.
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatCursor {
    pub created_at: i64,
    pub id: String,
}

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
    database
        .messages_before(&room, cursor.as_ref(), limit)
        .map(|messages| messages.iter().map(project_message).collect())
        .map_err(|error| error.to_string())
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
    database
        .search_messages(&query, limit)
        .map(|messages| messages.iter().map(project_message).collect())
        .map_err(|error| error.to_string())
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
    database
        .0
        .lock()
        .map_err(|_| "database lock poisoned".to_owned())?
        .insert_message(&message)
        .map_err(|error| error.to_string())?;
    Ok(project_message(&message))
}

/// Adds a local peer reaction to a message.
#[tauri::command]
pub fn add_reaction(
    database: State<'_, DatabaseState>,
    message_id: String,
    emoji: String,
) -> Result<(), String> {
    if emoji.is_empty() || emoji.chars().count() > 16 {
        return Err("reaction emoji is invalid".to_owned());
    }
    let message_id = Uuid::parse_str(&message_id).map_err(|error| error.to_string())?;
    let peer_id = super::network::local_peer_id()?;
    database
        .0
        .lock()
        .map_err(|_| "database lock poisoned".to_owned())?
        .add_reaction(message_id, peer_id, &emoji)
        .map_err(|error| error.to_string())
}

/// Publishes ephemeral typing state for the active local peer.
#[tauri::command]
pub fn set_typing(bus: State<'_, EventBus>, room: String, is_typing: bool) -> Result<(), String> {
    let peer_id = super::network::local_peer_id()?;
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

/// Direct-message acknowledgement state for the frontend.
#[derive(Serialize)]
pub struct ChatMessageStatus {
    pub delivered: bool,
    pub read: bool,
}

/// Edits a locally visible message body.
#[tauri::command]
pub fn edit_message(
    database: State<'_, DatabaseState>,
    id: String,
    body: String,
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
        .map_err(|error| error.to_string())
}

/// Soft-deletes a locally visible message.
#[tauri::command]
pub fn delete_message(database: State<'_, DatabaseState>, id: String) -> Result<(), String> {
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
        .map_err(|error| error.to_string())
}

fn project_message(message: &MessageRecord) -> ChatMessage {
    ChatMessage {
        author_id: message.author.as_uuid().to_string(),
        body: message.body.clone(),
        created_at: message.created_at,
        id: message.id.to_string(),
        reply_to: message.reply_to.map(|id| id.to_string()),
        room: message.room.clone(),
    }
}
