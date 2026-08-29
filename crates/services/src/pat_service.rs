use anyhow::{anyhow, Result};
use bcrypt::{hash, DEFAULT_COST};
use std::sync::Arc;
use template_studio_repositories::PatRepository;
use template_studio_shared::models::pat::{
    CreatePatRequest, CreatePatResponse, PatListItem, PatValidation,
};

const MAX_TOKENS_PER_USER: i64 = 20;
const TOKEN_PREFIX: &str = "ts_pat_";

const VALID_SCOPES: &[&str] = &[
    "template:read",
    "template:write",
    "template:delete",
    "template:publish",
    "generate:use",
    "release:create",
    "release:rollback",
];

pub struct PatService {
    pat_repo: Arc<PatRepository>,
}

impl PatService {
    pub fn new(pat_repo: Arc<PatRepository>) -> Self {
        Self { pat_repo }
    }

    pub async fn create(&self, user_id: i64, req: &CreatePatRequest) -> Result<CreatePatResponse> {
        let count = self.pat_repo.count_by_user(user_id).await?;
        if count >= MAX_TOKENS_PER_USER {
            return Err(anyhow!("令牌数量已达上限 ({})", MAX_TOKENS_PER_USER));
        }

        let raw_token = format!("{}{}", TOKEN_PREFIX, generate_random_string(32));
        let token_hash = hash(&raw_token, DEFAULT_COST)?;
        let token_prefix = format!(
            "{}{}…",
            TOKEN_PREFIX,
            &raw_token[TOKEN_PREFIX.len()..TOKEN_PREFIX.len() + 8]
        );

        // 过期时间由用户请求指定，极端值可能溢出，避免 unwrap panic
        let expires_at = match req.expires_in_days {
            Some(days) => Some(
                chrono::Utc::now()
                    .checked_add_signed(chrono::Duration::days(days))
                    .ok_or_else(|| anyhow!("过期天数超出可表示范围: {}", days))?
                    .naive_utc(),
            ),
            None => None,
        };

        let validated_scopes = validate_scopes(&req.scopes)?;
        let scopes_json = serde_json::to_string(&validated_scopes)?;

        let id = self
            .pat_repo
            .create(
                user_id,
                &req.name,
                &token_hash,
                &token_prefix,
                &scopes_json,
                expires_at,
            )
            .await?;

        Ok(CreatePatResponse {
            id,
            name: req.name.clone(),
            token: raw_token,
            token_prefix,
            scopes: validated_scopes,
            expires_at,
        })
    }

    pub async fn list(&self, user_id: i64) -> Result<Vec<PatListItem>> {
        self.pat_repo.list_by_user(user_id).await
    }

    pub async fn delete(&self, id: i64, user_id: i64) -> Result<bool> {
        self.pat_repo.delete(id, user_id).await
    }

    /// 通过原始令牌值验证，返回 user_id 和 scopes
    pub async fn validate(&self, raw_token: &str) -> Result<PatValidation> {
        let pat = {
            // raw_token 来自请求头，完全受用户控制：按字符截取避免
            // 多字节字符跨字节边界切片 panic，也避免长度不足时下标越界
            let prefix_part: String = raw_token.chars().take(TOKEN_PREFIX.len() + 8).collect();
            let all = self.pat_repo.list_by_prefix_like(&prefix_part).await?;
            let mut found = None;
            for p in all {
                if bcrypt::verify(raw_token, &p.token_hash).unwrap_or(false) {
                    found = Some(p);
                    break;
                }
            }
            found.ok_or_else(|| anyhow!("令牌无效"))?
        };

        // 检查过期
        if let Some(exp) = pat.expires_at {
            if chrono::Utc::now().naive_utc() > exp {
                return Err(anyhow!("令牌已过期"));
            }
        }

        // 更新最后使用时间
        let _ = self.pat_repo.update_last_used(pat.id).await;

        let scopes: Vec<String> = serde_json::from_str(&pat.scopes).unwrap_or_default();

        Ok(PatValidation {
            user_id: pat.user_id,
            scopes,
        })
    }
}

fn generate_random_string(len: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| CHARSET[rng.gen_range(0..CHARSET.len())] as char)
        .collect()
}

fn validate_scopes(scopes: &[String]) -> Result<Vec<String>> {
    let valid: Vec<String> = scopes
        .iter()
        .filter(|s| VALID_SCOPES.contains(&s.as_str()))
        .cloned()
        .collect();
    if valid.is_empty() {
        return Err(anyhow!("至少需要一个有效的权限范围"));
    }
    Ok(valid)
}
