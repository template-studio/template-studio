use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PersonalAccessToken {
    pub id: i64,
    pub user_id: i64,
    pub name: String,
    pub token_hash: String,
    pub token_prefix: String,
    pub scopes: String, // JSON 字符串
    pub last_used_at: Option<chrono::NaiveDateTime>,
    pub expires_at: Option<chrono::NaiveDateTime>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PatListItem {
    pub id: i64,
    pub name: String,
    pub token_prefix: String,
    pub scopes: String, // JSON 字符串
    pub last_used_at: Option<chrono::NaiveDateTime>,
    pub expires_at: Option<chrono::NaiveDateTime>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreatePatRequest {
    pub name: String,
    pub expires_in_days: Option<i64>,
    #[serde(default)]
    pub scopes: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CreatePatResponse {
    pub id: i64,
    pub name: String,
    pub token: String,
    pub token_prefix: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<chrono::NaiveDateTime>,
}

/// PAT 验证结果
#[derive(Debug)]
pub struct PatValidation {
    pub user_id: i64,
    pub scopes: Vec<String>,
}
