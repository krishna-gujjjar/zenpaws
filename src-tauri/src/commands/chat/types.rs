use serde::Serialize;
use zenpaws_database::MessageRecord;

/// Frontend-safe message projection without database internals.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub author_id: String,
    pub author_name: String,
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

/// Direct-message acknowledgement state for the frontend.
#[derive(Serialize)]
pub struct ChatMessageStatus {
    pub delivered: bool,
    pub read: bool,
}

pub fn project_message(
    message: &MessageRecord,
    author_name: String,
    reactions: Vec<String>,
) -> ChatMessage {
    ChatMessage {
        author_id: message.author.as_uuid().to_string(),
        author_name,
        body: message.body.clone(),
        created_at: message.created_at,
        id: message.id.to_string(),
        reply_to: message.reply_to.map(|id| id.to_string()),
        reactions,
        room: message.room.clone(),
    }
}
