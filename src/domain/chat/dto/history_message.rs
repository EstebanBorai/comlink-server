use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow, Serialize)]
pub struct HistoryMessageDTO {
    pub id: Uuid,
    pub content: String,
    pub kind: String,
    pub author_id: Uuid,
    pub chat_id: Uuid,
    pub file_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}