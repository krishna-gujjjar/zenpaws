use serde::Serialize;
use tauri::State;
use uuid::Uuid;
use zenpaws_database::{MessageCursor, MessageRecord};

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
            Ok(MessageCursor {
                created_at: cursor.created_at,
                id: Uuid::parse_str(&cursor.id).map_err(|error| error.to_string())?,
            })
        })
        .transpose()?;
    let database = database.0.lock().map_err(|_| "database lock poisoned".to_owned())?;
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
    let database = database.0.lock().map_err(|_| "database lock poisoned".to_owned())?;
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
