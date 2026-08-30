//! 凭据加密存储（桌面端本地 SQLite）
//!
//! 模型：机器绑定密钥 + AES-GCM 加密后入库。
//!
//! - 机器密钥：优先 OS 密钥链/凭据管理器（keyring crate）；
//!   不可用时回退到用户目录下的密钥文件（0600 权限，桌面单用户场景的合理降级）。
//! - 密文格式：`v1:` + base64(nonce(12B) || ciphertext+tag)，v1 前缀留升级空间。
//! - 兼容：解密失败或非 `v1:` 前缀的旧明文原样返回（不破坏存量数据），
//!   下次写入时自动升级为密文。

use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use base64::{engine::general_purpose, Engine as _};
use rand::RngCore;

const KEYRING_SERVICE: &str = "template-studio-desktop";
const KEYRING_ACCOUNT: &str = "local-db-encryption-key";
const CIPHER_PREFIX: &str = "v1:";

fn keyring_entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT).map_err(|e| e.to_string())
}

fn fallback_key_path() -> std::path::PathBuf {
    let mut p = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    p.push(".cicbyte");
    p.push("template_studio");
    p.push(".dbkey");
    p
}

/// 生成 32 字节随机密钥并 base64 编码
fn generate_key_b64() -> String {
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    general_purpose::STANDARD.encode(key)
}

/// 获取（或首次创建）机器绑定密钥，返回 32 字节
fn get_or_create_machine_key() -> Result<[u8; 32], String> {
    // 1) OS 凭据管理器
    match keyring_entry() {
        Ok(entry) => {
            match entry.get_password() {
                Ok(k) => {
                    if let Ok(key) = general_purpose::STANDARD.decode(k) {
                        if key.len() == 32 {
                            let mut out = [0u8; 32];
                            out.copy_from_slice(&key);
                            return Ok(out);
                        }
                    }
                    // 存量格式异常：重新生成并覆写
                    let k = generate_key_b64();
                    let _ = entry.set_password(&k);
                    let key = general_purpose::STANDARD
                        .decode(&k)
                        .map_err(|e| e.to_string())?;
                    let mut out = [0u8; 32];
                    out.copy_from_slice(&key);
                    return Ok(out);
                }
                Err(keyring::Error::NoEntry) => {
                    let k = generate_key_b64();
                    entry.set_password(&k).map_err(|e| e.to_string())?;
                    let key = general_purpose::STANDARD
                        .decode(&k)
                        .map_err(|e| e.to_string())?;
                    let mut out = [0u8; 32];
                    out.copy_from_slice(&key);
                    return Ok(out);
                }
                Err(e) => {
                    // 凭据管理器不可用（部分 Linux 无 keyring 服务）→ 回退密钥文件
                    eprintln!("[凭据加密] OS 凭据管理器不可用（{}），回退密钥文件", e);
                }
            }
        }
        Err(e) => {
            eprintln!("[凭据加密] keyring 初始化失败（{}），回退密钥文件", e);
        }
    }

    // 2) 回退：用户目录密钥文件
    let path = fallback_key_path();
    if path.exists() {
        let k = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let key = general_purpose::STANDARD
            .decode(k.trim())
            .map_err(|e| e.to_string())?;
        if key.len() == 32 {
            let mut out = [0u8; 32];
            out.copy_from_slice(&key);
            return Ok(out);
        }
        return Err("密钥文件内容无效".to_string());
    }
    let k = generate_key_b64();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    // Windows 无 0600 权限模型，用户目录隔离即边界
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
    }
    std::fs::write(&path, &k).map_err(|e| e.to_string())?;
    let key = general_purpose::STANDARD
        .decode(&k)
        .map_err(|e| e.to_string())?;
    let mut out = [0u8; 32];
    out.copy_from_slice(&key);
    Ok(out)
}

/// 加密明文 → `v1:base64(nonce||ciphertext)`
pub fn encrypt(plaintext: &str) -> Result<String, String> {
    let key = get_or_create_machine_key()?;
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(
            nonce,
            Payload {
                msg: plaintext.as_bytes(),
                aad: b"cred",
            },
        )
        .map_err(|e| format!("加密失败: {}", e))?;

    let mut buf = Vec::with_capacity(12 + ciphertext.len());
    buf.extend_from_slice(&nonce_bytes);
    buf.extend_from_slice(&ciphertext);
    Ok(format!(
        "{}{}",
        CIPHER_PREFIX,
        general_purpose::STANDARD.encode(buf)
    ))
}

/// 解密 `v1:` 密文；非 v1 前缀（历史明文）原样返回，保证向后兼容
pub fn decrypt(stored: &str) -> Result<String, String> {
    if !stored.starts_with(CIPHER_PREFIX) {
        return Ok(stored.to_string());
    }
    let raw = general_purpose::STANDARD
        .decode(stored.trim_start_matches(CIPHER_PREFIX))
        .map_err(|e| format!("密文 base64 解码失败: {}", e))?;
    if raw.len() < 12 {
        return Err("密文格式无效".to_string());
    }
    let (nonce_bytes, ciphertext) = raw.split_at(12);
    let key = get_or_create_machine_key()?;
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
    let plain = cipher
        .decrypt(
            Nonce::from_slice(nonce_bytes),
            Payload {
                msg: ciphertext,
                aad: b"cred",
            },
        )
        .map_err(|_| "解密失败（密钥与加密时不一致或数据损坏）".to_string())?;
    String::from_utf8(plain).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_and_plaintext_compat() {
        let secret = "p@ssw0rd-中文密码-🔑";
        let enc = encrypt(secret).unwrap();
        assert!(enc.starts_with("v1:"));
        assert_ne!(enc, secret);
        assert_eq!(decrypt(&enc).unwrap(), secret, "加解密往返应一致");

        // 历史/错误前缀明文应原样返回（兼容）
        assert_eq!(decrypt("legacy-plaintext").unwrap(), "legacy-plaintext");
        assert_eq!(decrypt("").unwrap(), "");
    }

    #[test]
    fn test_nonce_uniqueness() {
        let a = encrypt("same").unwrap();
        let b = encrypt("same").unwrap();
        assert_ne!(a, b, "相同明文两次加密的密文应不同（随机 nonce）");
        assert_eq!(decrypt(&a).unwrap(), "same");
        assert_eq!(decrypt(&b).unwrap(), "same");
    }
}
