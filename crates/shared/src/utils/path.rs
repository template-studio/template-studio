//! 用户输入路径的安全校验与拼接
//!
//! 所有接受客户端提供的路径 / 文件名 / zip 条目名的代码必须通过本模块拼接，
//! 防止路径穿越（`../`、反斜杠变体 `..\`、绝对路径、Windows 盘符）。
//!
//! 注意：`:` 被整体拒绝——Windows 文件名本就不允许该字符，Unix 下含 `:` 的
//! 模板文件名也极罕见，从严处理。

use std::path::{Path, PathBuf};

/// 校验用户提供的相对路径片段，拒绝穿越与绝对路径
pub fn validate_relative_path(user_path: &str) -> Result<(), String> {
    let normalized = user_path.replace('\\', "/");
    if normalized.trim().is_empty() {
        return Err("路径不能为空".to_string());
    }
    // 绝对路径（Unix）或 Windows 盘符（C:）
    if normalized.starts_with('/') || normalized.contains(':') {
        return Err(format!("不允许使用绝对路径: {}", user_path));
    }
    for comp in normalized.split('/') {
        if comp == ".." {
            return Err(format!("路径不允许包含 ..: {}", user_path));
        }
    }
    Ok(())
}

/// 安全拼接：先做组件级校验再 join。
/// 组件级校验不依赖目标存在，读写路径均可使用。
pub fn safe_join(base: &Path, user_path: &str) -> Result<PathBuf, String> {
    validate_relative_path(user_path)?;
    Ok(base.join(user_path.replace('\\', "/")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_paths_pass() {
        assert!(validate_relative_path("src/main.go").is_ok());
        assert!(validate_relative_path(".github\\workflows\\release.yml").is_ok()); // Windows 分隔符的合法路径
        assert!(validate_relative_path("dir with space/file.txt").is_ok());
        assert!(safe_join(Path::new("/base"), "a/b.txt").is_ok());
    }

    #[test]
    fn test_traversal_rejected() {
        assert!(validate_relative_path("../secret").is_err());
        assert!(validate_relative_path("..\\..\\config.toml").is_err()); // 反斜杠变体
        assert!(validate_relative_path("a/../../b").is_err());
        assert!(validate_relative_path("/etc/passwd").is_err()); // 绝对路径
        assert!(validate_relative_path("C:\\Windows\\system32").is_err()); // 盘符
        assert!(validate_relative_path("").is_err());
    }

    #[test]
    fn test_safe_join_result() {
        let joined = safe_join(Path::new("/base"), "sub\\file.txt").unwrap();
        assert_eq!(joined, Path::new("/base").join("sub/file.txt"));
        assert!(safe_join(Path::new("/base"), "../escape").is_err());
    }
}
