use std::sync::Arc;
use anyhow::{anyhow, Result};
use bcrypt::{hash, DEFAULT_COST};
use template_studio_shared::models::pat::{CreatePatRequest, CreatePatResponse, PatListItem, PatValidation};
use template_studio_repositories::PatRepository;

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
        let token_prefix = format!("{}{}…", TOKEN_PREFIX, &raw_token[TOKEN_PREFIX.len()..TOKEN_PREFIX.len() + 8]);

        let expires_at = req.expires_in_days.map(|days| {
            chrono::Utc::now()
                .checked_add_signed(chrono::Duration::days(days))
                .unwrap()
                .naive_utc()
        });

        let validated_scopes = validate_scopes(&req.scopes)?;
        let scopes_json = serde_json::to_string(&validated_scopes)?;

        let id = self.pat_repo.create(user_id, &req.name, &token_hash, &token_prefix, &scopes_json, expires_at).await?;

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
            let prefix_part = &raw_token[..TOKEN_PREFIX.len() + 8.min(raw_token.len() - TOKEN_PREFIX.len())];
            let all = self.pat_repo.list_by_prefix_like(prefix_part).await?;
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
    (0..len).map(|_| CHARSET[rng.gen_range(0..CHARSET.len())] as char).collect()
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
