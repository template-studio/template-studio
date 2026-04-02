use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PersonalAccessToken {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub token_hash: String,
    pub token_prefix: String,
    pub last_used_at: Option<chrono::NaiveDateTime>,
    pub expires_at: Option<chrono::NaiveDateTime>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PatListItem {
    pub id: i64,
    pub name: String,
    pub token_prefix: String,
    pub last_used_at: Option<chrono::NaiveDateTime>,
    pub expires_at: Option<chrono::NaiveDateTime>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreatePatRequest {
    pub name: String,
    pub expires_in_days: Option<i64>, // None = never expires
}

#[derive(Debug, Serialize)]
pub struct CreatePatResponse {
    pub id: i64,
    pub name: String,
    pub token: String,       // 完整令牌，仅创建时返回一次
    pub token_prefix: String,
    pub expires_at: Option<chrono::NaiveDateTime>,
}
