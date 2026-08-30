//! 操作审计日志服务
//!
//! 记录关键管理操作（谁、何时、对什么、做了什么、从哪来）。
//! 写入失败只记告警日志、不影响业务请求——审计是旁路关注点，
//! 不能因为审计库故障拖垮主流程。

use anyhow::Result;
use sqlx::MySqlPool;
use tracing::warn;

/// 审计日志条目
#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub user_id: i64,
    pub username: String,
    /// 动作标识，如 "template.delete"
    pub action: String,
    /// 资源类型，如 "template"
    pub resource_type: String,
    /// 资源 ID（字符串兼容）
    pub resource_id: Option<String>,
    /// 补充信息（JSON 摘要）
    pub detail: Option<String>,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
}

pub struct AuditService {
    pool: MySqlPool,
}

impl AuditService {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// 写入审计日志（失败仅告警，不影响业务）
    pub async fn record(&self, entry: &AuditEntry) {
        let result: Result<()> = async {
            sqlx::query(
                "INSERT INTO audit_logs (user_id, username, action, resource_type, resource_id, detail, ip, user_agent)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(entry.user_id)
            .bind(&entry.username)
            .bind(&entry.action)
            .bind(&entry.resource_type)
            .bind(&entry.resource_id)
            .bind(&entry.detail)
            .bind(&entry.ip)
            .bind(&entry.user_agent)
            .execute(&self.pool)
            .await?;
            Ok(())
        }
        .await;

        if let Err(e) = result {
            warn!(
                "审计日志写入失败（不影响业务）: action={}, user={}, error={}",
                entry.action, entry.username, e
            );
        }
    }

    /// 查询审计日志（管理界面用）
    pub async fn list(
        &self,
        action: Option<&str>,
        resource_type: Option<&str>,
        limit: u32,
        offset: u32,
    ) -> Result<(Vec<AuditLogRow>, i64)> {
        let mut where_clauses = vec!["1=1".to_string()];
        if action.is_some() {
            where_clauses.push("action = ?".to_string());
        }
        if resource_type.is_some() {
            where_clauses.push("resource_type = ?".to_string());
        }
        let where_sql = where_clauses.join(" AND ");

        let rows = sqlx::query_as::<_, AuditLogRow>(&format!(
            "SELECT id, user_id, username, action, resource_type, resource_id, detail, ip, user_agent, created_at
             FROM audit_logs WHERE {} ORDER BY id DESC LIMIT ? OFFSET ?",
            where_sql
        ))
        .bind(action)
        .bind(resource_type)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let total: i64 = sqlx::query_scalar(&format!(
            "SELECT COUNT(*) FROM audit_logs WHERE {}",
            where_sql
        ))
        .bind(action)
        .bind(resource_type)
        .fetch_one(&self.pool)
        .await?;

        Ok((rows, total))
    }
}

/// 审计日志查询行
#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct AuditLogRow {
    pub id: i64,
    pub user_id: i64,
    pub username: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub detail: Option<String>,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
