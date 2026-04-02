use std::sync::Arc;
use anyhow::{anyhow, Result};
use bcrypt::{hash, DEFAULT_COST};
use template_studio_shared::models::pat::{CreatePatRequest, CreatePatResponse, PatListItem};
use template_studio_repositories::PatRepository;

const MAX_TOKENS_PER_USER: i64 = 20;
const TOKEN_PREFIX: &str = "ts_pat_";

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

        let id = self.pat_repo.create(user_id, &req.name, &token_hash, &token_prefix, expires_at).await?;

        Ok(CreatePatResponse {
            id,
            name: req.name.clone(),
            token: raw_token,
            token_prefix,
            expires_at,
        })
    }

    pub async fn list(&self, user_id: i64) -> Result<Vec<PatListItem>> {
        self.pat_repo.list_by_user(user_id).await
    }

    pub async fn delete(&self, id: i64, user_id: i64) -> Result<bool> {
        self.pat_repo.delete(id, user_id).await
    }

    /// 通过原始令牌值验证，返回 user_id
    pub async fn validate(&self, raw_token: &str) -> Result<i64> {
        let token_hash = {
            // 遍历所有 PAT 找到匹配的（bcrypt 无法反向查询）
            // 优化：先用 prefix 缩小范围
            let prefix_part = &raw_token[..TOKEN_PREFIX.len() + 8.min(raw_token.len() - TOKEN_PREFIX.len())];
            let all = self.pat_repo.list_by_prefix_like(prefix_part).await?;
            let mut found = None;
            for pat in all {
                if bcrypt::verify(raw_token, &pat.token_hash).unwrap_or(false) {
                    found = Some(pat);
                    break;
                }
            }
            found.ok_or_else(|| anyhow!("令牌无效"))?
        };

        // 检查过期
        if let Some(exp) = token_hash.expires_at {
            if chrono::Utc::now().naive_utc() > exp {
                return Err(anyhow!("令牌已过期"));
            }
        }

        // 更新最后使用时间
        let _ = self.pat_repo.update_last_used(token_hash.id).await;

        Ok(token_hash.user_id)
    }
}

fn generate_random_string(len: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    (0..len).map(|_| CHARSET[rng.gen_range(0..CHARSET.len())] as char).collect()
}
