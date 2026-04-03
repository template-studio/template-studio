use std::sync::Arc;
use anyhow::{anyhow, Result};
use bcrypt::{hash, DEFAULT_COST};
use sqlx::{MySqlPool, Row};
use template_studio_repositories::SystemSettingRepository;

pub struct EmailService {
    db: MySqlPool,
    setting_repo: Arc<SystemSettingRepository>,
    base_url: String,
}

impl EmailService {
    pub fn new(db: MySqlPool, setting_repo: Arc<SystemSettingRepository>, base_url: String) -> Self {
        Self { db, setting_repo, base_url }
    }

    /// 从系统设置获取 SMTP 配置
    async fn get_smtp_config(&self) -> Result<SmtpConfig> {
        let settings = self.setting_repo.get_by_group("smtp").await?;
        let get = |key: &str| -> String {
            settings.iter()
                .find(|s| s.key == key)
                .and_then(|s| s.value.clone())
                .unwrap_or_default()
        };

        let host = get("smtp_host");
        if host.is_empty() {
            return Err(anyhow!("SMTP 未配置，请在系统设置中配置 SMTP"));
        }

        Ok(SmtpConfig {
            host,
            port: get("smtp_port").parse::<u16>().unwrap_or(465),
            username: get("smtp_username"),
            password: get("smtp_password"),
            sender: get("smtp_sender"),
            use_tls: get("smtp_tls") != "false",
        })
    }

    /// 发送密码重置邮件
    pub async fn send_reset_email(&self, email: &str) -> Result<()> {
        // 查找用户
        let user_id: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM users WHERE email = ? AND status = 1"
        )
        .bind(email)
        .fetch_optional(&self.db)
        .await?
        .flatten();

        let user_id = match user_id {
            Some(id) => id,
            None => return Ok(()), // 邮箱不存在，静默返回
        };

        // 生成重置令牌
        let token = uuid::Uuid::new_v4().to_string();
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);

        // 保存令牌
        sqlx::query(
            "INSERT INTO password_reset_tokens (user_id, token, email, expires_at) VALUES (?, ?, ?, ?)"
        )
        .bind(user_id)
        .bind(&token)
        .bind(email)
        .bind(expires_at.naive_utc())
        .execute(&self.db)
        .await?;

        tracing::info!("Password reset token created for user {}: {}...", user_id, &token[..8]);

        // 发送邮件
        let smtp = self.get_smtp_config().await?;
        let reset_url = format!("{}/reset-password?token={}", self.base_url.trim_end_matches('/'), token);

        let subject = "Template Studio - 密码重置";
        let body = format!(
            r#"<div style="max-width:600px;margin:0 auto;font-family:sans-serif;padding:40px 20px">
                <h2 style="color:#0f172a">密码重置</h2>
                <p style="color:#475569;font-size:15px;line-height:1.6">
                    您收到此邮件是因为您（或其他人）请求重置 Template Studio 账号的密码。<br/>
                    请点击下方按钮重置密码，链接1小时内有效：
                </p>
                <div style="margin:30px 0;text-align:center">
                    <a href="{reset_url}" style="background:#0f172a;color:#fff;padding:12px 32px;border-radius:8px;text-decoration:none;font-weight:600;display:inline-block">
                        重置密码
                    </a>
                </div>
                <p style="color:#94a3b8;font-size:13px">
                    如果按钮无法点击，请复制以下链接到浏览器：<br/>
                    <a href="{reset_url}" style="color:#3b82f6;word-break:break-all">{reset_url}</a>
                </p>
                <p style="color:#94a3b8;font-size:13px">如果您没有请求重置密码，请忽略此邮件。</p>
            </div>"#
        );

        self.send_email(&smtp, email, subject, &body).await
    }

    /// 重置密码
    pub async fn reset_password(&self, token: &str, new_password: &str) -> Result<()> {
        // 先检查 token 是否存在（不管 used 状态）
        let exists: bool = sqlx::query_scalar(
            "SELECT COUNT(*) > 0 FROM password_reset_tokens WHERE token = ?"
        )
        .bind(token)
        .fetch_one(&self.db)
        .await
        .unwrap_or(false);

        if !exists {
            tracing::error!("Token not found in database: {}", &token[..8.min(token.len())]);
            return Err(anyhow!("重置链接无效或已过期"));
        }

        // 查找有效令牌
        let record = sqlx::query(
            "SELECT user_id, expires_at, used FROM password_reset_tokens WHERE token = ? AND used = 0"
        )
        .bind(token)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| anyhow!("重置链接已被使用，请重新申请"))?;

        let user_id: i64 = record.get("user_id");
        let expires_at: chrono::NaiveDateTime = record.get("expires_at");
        if chrono::Utc::now().naive_utc() > expires_at {
            return Err(anyhow!("重置链接已过期，请重新申请"));
        }

        // 更新密码
        let password_hash = hash(new_password, DEFAULT_COST)?;
        sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
            .bind(&password_hash)
            .bind(user_id)
            .execute(&self.db)
            .await?;

        // 标记令牌已使用
        sqlx::query("UPDATE password_reset_tokens SET used = 1 WHERE token = ?")
            .bind(token)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    /// 发送邮件
    async fn send_email(&self, config: &SmtpConfig, to: &str, subject: &str, html_body: &str) -> Result<()> {
        use lettre::{Message, SmtpTransport, Transport};
        use lettre::message::{header::ContentType, Mailbox};

        let from = config.sender.parse::<Mailbox>()
            .unwrap_or_else(|_| format!("Template Studio <{}>", config.username).parse().unwrap());

        let email = Message::builder()
            .from(from)
            .to(to.parse::<Mailbox>()?)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(html_body.to_string())?;

        let transport = if config.use_tls {
            SmtpTransport::relay(&config.host)?
                .port(config.port)
                .credentials(lettre::transport::smtp::authentication::Credentials::new(
                    config.username.clone(),
                    config.password.clone(),
                ))
                .build()
        } else {
            SmtpTransport::builder_dangerous(&config.host)
                .port(config.port)
                .credentials(lettre::transport::smtp::authentication::Credentials::new(
                    config.username.clone(),
                    config.password.clone(),
                ))
                .build()
        };

        transport.send(&email)?;
        Ok(())
    }

    /// 发送测试邮件
    pub async fn send_test_email(&self, to: &str) -> Result<()> {
        let smtp = self.get_smtp_config().await?;
        let subject = "Template Studio - SMTP 测试邮件";
        let body = format!(
            r#"<div style="max-width:600px;margin:0 auto;font-family:sans-serif;padding:40px 20px">
                <h2 style="color:#0f172a">SMTP 测试成功</h2>
                <p style="color:#475569;font-size:15px;line-height:1.6">
                    恭喜！您的 SMTP 邮件服务配置正确，密码重置功能已可用。
                </p>
                <p style="color:#94a3b8;font-size:13px;margin-top:30px">
                    此邮件由 Template Studio 系统自动发送，请勿回复。
                </p>
            </div>"#
        );
        self.send_email(&smtp, to, subject, &body).await
    }
}

struct SmtpConfig {
    host: String,
    port: u16,
    username: String,
    password: String,
    sender: String,
    use_tls: bool,
}
