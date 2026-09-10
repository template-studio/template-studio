//! 项目转模板:阶段 0 克隆镜像 + 阶段 A 规则扫描
//! 设计文档: dev-docs/project-to-template.md
//! 隔离原则: 所有来源统一 clone 到持久镜像(workspace/repos/<指纹>/),原始目录只读。

use std::path::{Path, PathBuf};

// ===== 规则包(声明式,内置四份 + 用户目录覆盖) =====

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
// 消费方在二期分析管线(启发式通道),一期仅加载
#[allow(dead_code)]
pub struct ConstantRule {
    /// 捕获组 1 为候选值
    pub pattern: String,
    pub var_name: String,
    #[serde(default = "default_var_type")]
    pub r#type: String,
}

fn default_var_type() -> String {
    "string".to_string()
}

fn default_max_file_size() -> u64 {
    1024 * 1024
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
// constants 字段由二期分析管线消费
#[allow(dead_code)]
pub struct RulePack {
    pub id: String,
    /// 命中任一文件即识别为该技术栈
    #[serde(rename = "match")]
    pub match_files: Vec<String>,
    #[serde(default)]
    pub exclude_dirs: Vec<String>,
    #[serde(default)]
    pub exclude_files: Vec<String>,
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
    #[serde(default)]
    pub binary_extensions: Vec<String>,
    #[serde(default = "default_max_file_size")]
    pub max_file_size: u64,
    /// 入口文件(分析阶段优先读取);支持 * 通配
    #[serde(default)]
    pub entry_files: Vec<String>,
    #[serde(default)]
    pub constants: Vec<ConstantRule>,
}

fn parse_pack(raw: &str, origin: &str) -> Option<RulePack> {
    match serde_json::from_str::<RulePack>(raw) {
        Ok(p) => Some(p),
        Err(e) => {
            eprintln!("规则包解析失败({origin}): {e}");
            None
        }
    }
}

/// 内置四份(include_str! 打包进二进制)
fn builtin_packs() -> Vec<RulePack> {
    [
        include_str!("../../rules/node.json"),
        include_str!("../../rules/go.json"),
        include_str!("../../rules/java.json"),
        include_str!("../../rules/python.json"),
        include_str!("../../rules/rust.json"),
    ]
    .iter()
    .filter_map(|raw| parse_pack(raw, "builtin"))
    .collect()
}

fn studio_home(sub: &str) -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".cicbyte")
        .join("template_studio")
        .join(sub)
}

/// 用户规则包目录 ~/.cicbyte/template_studio/rules/*.json(同 id 覆盖内置)
pub fn all_packs() -> Vec<RulePack> {
    let mut packs = builtin_packs();
    let dir = studio_home("rules");
    if let Ok(entries) = std::fs::read_dir(&dir) {
        let mut user: Vec<RulePack> = entries
            .flatten()
            .filter(|e| {
                let n = e.file_name().to_string_lossy().to_string();
                n.ends_with(".json")
            })
            .filter_map(|e| {
                let raw = std::fs::read_to_string(e.path()).ok()?;
                parse_pack(&raw, &e.path().display().to_string())
            })
            .collect();
        for u in user.drain(..) {
            if let Some(b) = packs.iter().position(|p| p.id == u.id) {
                packs[b] = u;
            } else {
                packs.push(u);
            }
        }
    }
    packs
}

// ===== 阶段 0: 克隆镜像 =====

/// FNV-1a 64 位:repoUrl 指纹(镜像目录名),不引入额外 crate
fn fnv1a(s: &str) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("{h:016x}")
}

fn run_git(args: &[&str], cwd: Option<&Path>) -> Result<String, String> {
    let mut cmd = std::process::Command::new("git");
    cmd.args(args);
    if let Some(c) = cwd {
        cmd.current_dir(c);
    }
    // Windows 下避免弹窗/继承控制台编码问题
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    let out = cmd
        .output()
        .map_err(|e| format!("无法启动 git(请确认已安装): {e}"))?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            format!("git {:?} 失败(退出码 {:?})", args, out.status.code())
        } else {
            stderr
        });
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// 镜像根目录 workspace/repos/
fn repos_dir() -> PathBuf {
    studio_home("workspace").join("repos")
}

/// 克隆来源到持久镜像:已存在则 fetch 增量并 fast-forward;返回 {dir, branch, commit, reused}
/// source 支持远程 URL(github/gitee/gitlab,凭据走本机 git 配置)与本地 git 仓库路径。
#[tauri::command]
pub async fn convert_clone(source: String, branch: Option<String>) -> Result<String, String> {
    let src = source.trim().to_string();
    if src.is_empty() {
        return Err("来源不能为空".to_string());
    }

    // 本地来源必须已是 git 仓库(不碰原始目录,一律 clone)
    let is_local = !src.starts_with("http://")
        && !src.starts_with("https://")
        && !src.starts_with("git@")
        && !src.starts_with("ssh://");
    if is_local {
        let p = Path::new(&src);
        if !p.is_dir() {
            return Err("本地路径不存在或不是目录".to_string());
        }
        if !p.join(".git").exists() {
            return Err("仅支持 git 项目:该目录不是 git 仓库(未提交内容也不参与转换,请先 git init/commit)".to_string());
        }
    }

    let repo_dir = repos_dir().join(fnv1a(&src));
    let reused = repo_dir.exists();

    if reused {
        // 增量更新:fetch + 重置到远端分支(镜像是我们的副本,reset 安全)
        run_git(&["fetch", "--all", "--prune"], Some(&repo_dir))?;
        if let Some(b) = branch.as_deref().filter(|s| !s.is_empty()) {
            run_git(&["checkout", b], Some(&repo_dir))?;
            let tracking = format!("origin/{b}");
            // 远端分支存在则对齐;本地仅分支则保留现状
            if run_git(&["rev-parse", "--verify", &tracking], Some(&repo_dir)).is_ok() {
                run_git(&["reset", "--hard", &tracking], Some(&repo_dir))?;
            }
        } else {
            if run_git(&["rev-parse", "--verify", "origin/HEAD"], Some(&repo_dir)).is_ok() {
                run_git(&["reset", "--hard", "origin/HEAD"], Some(&repo_dir))?;
            } else {
                run_git(&["reset", "--hard", "@{u}"], Some(&repo_dir))?; // 无 origin/HEAD 时跟上游
            }
        }
    } else {
        std::fs::create_dir_all(repos_dir()).map_err(|e| format!("创建镜像目录失败: {e}"))?;
        let mut args: Vec<&str> = vec!["clone"];
        if let Some(b) = branch.as_deref().filter(|s| !s.is_empty()) {
            args.extend(["--branch", b]);
        }
        // 完整 clone(保留历史,为项目更新留基线,见设计文档 §5.1)
        args.push(&src);
        let dir_str = repo_dir.to_string_lossy().to_string();
        args.push(&dir_str);
        run_git(&args, None)?;
    }

    let commit = run_git(&["rev-parse", "HEAD"], Some(&repo_dir))?;
    let cur_branch = run_git(&["rev-parse", "--abbrev-ref", "HEAD"], Some(&repo_dir))?;

    Ok(serde_json::json!({
        "dir": repo_dir.to_string_lossy(),
        "branch": if cur_branch == "HEAD" { branch.unwrap_or_default() } else { cur_branch },
        "commit": commit,
        "reused": reused,
    })
    .to_string())
}

// ===== 阶段 A: 规则扫描 =====

/// 简单通配匹配:仅支持 * 与 ?,且 * 不跨 / —— 按 / 分段逐段匹配
fn glob_match(pattern: &str, name: &str) -> bool {
    fn seg_match(pattern: &[char], name: &[char]) -> bool {
        // 经典双指针 + 回溯的段内通配(* 不含 / 已由分段保证)
        let (mut pi, mut ni) = (0usize, 0usize);
        let (mut star, mut mark) = (usize::MAX, 0usize);
        while ni < name.len() {
            if pi < pattern.len() && (pattern[pi] == name[ni] || pattern[pi] == '?') {
                pi += 1;
                ni += 1;
            } else if pi < pattern.len() && pattern[pi] == '*' {
                star = pi;
                mark = ni;
                pi += 1;
            } else if star != usize::MAX {
                pi = star + 1;
                mark += 1;
                ni = mark;
            } else {
                return false;
            }
        }
        while pi < pattern.len() && pattern[pi] == '*' {
            pi += 1;
        }
        pi == pattern.len()
    }
    let ps: Vec<&str> = pattern.split('/').collect();
    let ns: Vec<&str> = name.split('/').collect();
    if ps.len() != ns.len() {
        return false;
    }
    ps.iter()
        .zip(ns.iter())
        .all(|(p, n)| seg_match(&p.chars().collect::<Vec<char>>(), &n.chars().collect::<Vec<char>>()))
}

/// 项目类型识别:root 下命中任一 match 文件的第一个规则包(顺序=内置声明序)
fn detect_pack(root: &Path, packs: &[RulePack]) -> Option<RulePack> {
    packs
        .iter()
        .find(|p| p.match_files.iter().any(|m| root.join(m).is_file()))
        .cloned()
}

struct ScanStat {
    kept: usize,
    excluded: usize,
    total_size: u64,
}

/// 扫描克隆镜像:识别类型 → 应用剔除规则 → 二进制/大小过滤 → IR.files 雏形
#[tauri::command]
pub async fn convert_scan(root: String) -> Result<String, String> {
    let root_path = PathBuf::from(&root);
    if !root_path.is_dir() {
        return Err("镜像目录不存在,请先克隆".to_string());
    }

    let packs = all_packs();
    let pack = match detect_pack(&root_path, &packs) {
        Some(p) => p,
        None => {
            return Err(
                "未识别出项目类型(缺少 package.json/go.mod/pom.xml/build.gradle/pyproject.toml 等);\
                 可在 ~/.cicbyte/template_studio/rules/ 添加自定义规则包"
                    .to_string(),
            )
        }
    };

    let mut files: Vec<serde_json::Value> = Vec::new();
    let mut stat = ScanStat { kept: 0, excluded: 0, total_size: 0 };
    walk_scan(&root_path, &root_path, &pack, &mut files, &mut stat)?;

    Ok(serde_json::json!({
        "packId": pack.id,
        "root": root,
        "files": files,
        "stats": { "kept": stat.kept, "excluded": stat.excluded, "totalSize": stat.total_size },
    })
    .to_string())
}

fn walk_scan(
    root: &Path,
    dir: &Path,
    pack: &RulePack,
    out: &mut Vec<serde_json::Value>,
    stat: &mut ScanStat,
) -> Result<(), String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("读取目录失败 {}: {e}", dir.display()))?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();
        if path.is_dir() {
            // .git 永不进模板;规则包目录黑名单
            if name == ".git" || pack.exclude_dirs.iter().any(|d| glob_match(d, &name)) {
                continue;
            }
            walk_scan(root, &path, pack, out, stat)?;
        } else {
            let meta = entry.metadata().map_err(|e| format!("读取文件信息失败: {e}"))?;
            let rel = path
                .strip_prefix(root)
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_default();
            let ext = Path::new(&name)
                .extension()
                .map(|e| format!(".{}", e.to_string_lossy().to_lowercase()))
                .unwrap_or_default();

            let mut reason = String::new();
            if pack.exclude_files.iter().any(|f| rel == *f || name == *f) {
                reason = "锁定/环境文件".into();
            } else if pack.exclude_patterns.iter().any(|p| glob_match(p, &name)) {
                reason = "构建产物/日志".into();
            } else if pack.binary_extensions.iter().any(|b| *b == ext) {
                reason = "二进制资源".into();
            } else if meta.len() > pack.max_file_size {
                reason = "超过大小上限".into();
            }

            let excluded = !reason.is_empty();
            let is_entry = pack.entry_files.iter().any(|e| glob_match(e, &rel));
            if !excluded {
                stat.kept += 1;
                stat.total_size += meta.len();
            } else {
                stat.excluded += 1;
            }
            out.push(serde_json::json!({
                "path": rel, "size": meta.len(), "action": if excluded { "exclude" } else { "keep" },
                "reason": reason, "isEntry": is_entry,
            }));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_basics() {
        assert!(glob_match("*.log", "a.log"));
        assert!(!glob_match("*.log", "a.txt"));
        assert!(glob_match("src/index.*", "src/index.ts"));
        assert!(glob_match("cmd/*/main.go", "cmd/server/main.go"));
        assert!(!glob_match("cmd/*/main.go", "cmd/a/b/main.go")); // * 不跨 /
        assert!(glob_match("*.egg-info", "my_pkg.egg-info"));
        assert!(glob_match("exact.json", "exact.json"));
    }

    #[test]
    fn builtin_packs_parse() {
        let packs = all_packs();
        assert!(packs.len() >= 5, "内置规则包不足: {}", packs.len());
        for id in ["node", "go", "java", "python", "rust"] {
            assert!(packs.iter().any(|p| p.id == id), "缺少 {id}");
        }
    }

    #[test]
    fn fnv_stable() {
        assert_eq!(fnv1a("https://github.com/x/y"), fnv1a("https://github.com/x/y"));
        assert_ne!(fnv1a("abc"), fnv1a("abd"));
    }

    #[test]
    fn detect_pack_by_marker_file() {
        let tmp = std::env::temp_dir().join(format!("t124-detect-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("package.json", ), "{}").unwrap();
        assert_eq!(detect_pack(&tmp, &all_packs()).unwrap().id, "node");
        std::fs::remove_file(tmp.join("package.json")).unwrap();
        std::fs::write(tmp.join("Cargo.toml"), "").unwrap();
        assert_eq!(detect_pack(&tmp, &all_packs()).unwrap().id, "rust");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn parse_json_block_tolerates_fences() {
        let v = parse_json_block("```json\n{\"a\":1}\n```").unwrap();
        assert_eq!(v["a"].as_i64(), Some(1));
        let v = parse_json_block("前置说明文字 {\"b\": [1,2]} 后缀").unwrap();
        assert_eq!(v["b"].as_array().unwrap().len(), 2);
        assert!(parse_json_block("没有任何大括号").is_none());
    }

    #[test]
    fn regex_channel_splits_by_value() {
        let pack = all_packs().into_iter().find(|p| p.id == "rust").unwrap();
        let files = vec![
            ("src/main.rs".to_string(), "listen(port = 8080);".to_string()),
            ("config.yml".to_string(), "port: 8080
admin_port = 8081".to_string()),
        ];
        let cands = heuristic_regex_candidates(&pack, &files);
        let ports: Vec<&VarCandidate> = cands.iter().filter(|c| c.name == "server_port").collect();
        assert_eq!(ports.len(), 2, "同规则不同值应拆分: 8080/8081");
        let c8080 = ports.iter().find(|c| c.default_value == "8080").unwrap();
        assert_eq!(c8080.occurrences.len(), 2, "8080 跨两文件聚合");
    }

    #[test]
    fn merge_ai_naming_priority() {
        let mk = |name: &str, val: &str, sem: &str, src: &str, conf: f64| VarCandidate {
            name: name.into(), r#type: "number".into(), default_value: val.into(),
            confidence: conf, semantic: sem.into(), source: src.into(),
            occurrences: vec![serde_json::json!({"path": "a", "original": val, "count": 1})],
        };
        // 启发式与 AI 同值同语义 → 融合且 AI 命名优先
        let merged = merge_candidates(vec![
            mk("server_port", "8080", "server_port", "heuristic", 0.7),
            mk("http_listen_port", "8080", "port", "ai", 0.9),
        ]);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].name, "http_listen_port");
        assert_eq!(merged[0].confidence, 0.9);
        // 不同具体语义同值不融合(admin: 路由 vs db)
        let split = merge_candidates(vec![
            mk("route_prefix", "/admin", "path", "ai", 0.8),
            mk("db_user", "/admin", "db", "ai", 0.8),
        ]);
        assert_eq!(split.len(), 2);
        // generic 被具体语义吸收
        let absorb = merge_candidates(vec![
            mk("var_8080", "8080", "generic", "heuristic", 0.4),
            mk("app_port", "8080", "port", "ai", 0.9),
        ]);
        assert_eq!(absorb.len(), 1);
        assert_eq!(absorb[0].name, "app_port");
    }

    #[test]
    fn build_batches_entry_first_and_caps() {
        let mut files = Vec::new();
        for i in 0..20 {
            files.push((format!("src/file{i}.rs"), "x".repeat(1024)));
        }
        files.push(("Cargo.toml".to_string(), "[package]".to_string()));
        let packs = all_packs();
        let rust = packs.iter().find(|p| p.id == "rust").unwrap();
        let batches = build_batches(&files, &rust.entry_files);
        assert!(!batches.is_empty());
        assert!(batches[0].iter().any(|(p, _)| p == "Cargo.toml"), "入口文件优先");
        for b in &batches {
            assert!(b.len() <= AI_BATCH_FILES);
        }
    }

    /// 本仓库做本地 git 来源:clone 出镜像后 scan 应识别 rust 并产出非空清单
    #[tokio::test]
    async fn clone_and_scan_self() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .unwrap()
            .to_string_lossy()
            .to_string();
        let out = convert_clone(repo_root, None).await.expect("clone 本仓库失败");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert!(v["commit"].as_str().is_some_and(|c| c.len() >= 7), "缺基线 commit");
        assert!(v["branch"].as_str().is_some_and(|b| !b.is_empty()));

        let scan = convert_scan(v["dir"].as_str().unwrap().to_string())
            .await
            .expect("scan 失败");
        let sv: serde_json::Value = serde_json::from_str(&scan).unwrap();
        assert_eq!(sv["packId"].as_str(), Some("rust"));
        let files = sv["files"].as_array().unwrap();
        assert!(!files.is_empty());
        // .git 与 target 绝不进清单;Cargo.lock 被剔除
        let paths: Vec<&str> = files.iter().filter_map(|f| f["path"].as_str()).collect();
        assert!(!paths.iter().any(|p| p.starts_with(".git/") || p.contains("/.git/")));
        // 剔除项也在清单里(带 action=exclude,三色树需要),但 keep 集不含它们
        let kept: Vec<&str> = files
            .iter()
            .filter(|f| f["action"].as_str() == Some("keep"))
            .filter_map(|f| f["path"].as_str())
            .collect();
        assert!(!kept.is_empty());
        assert!(!kept.iter().any(|p| *p == "Cargo.lock"), "Cargo.lock 应被剔除");
        // 剔除的 Cargo.lock 在清单中有据可查
        assert!(files
            .iter()
            .any(|f| f["path"].as_str() == Some("Cargo.lock") && f["action"].as_str() == Some("exclude")));
    }
}

// ===== 阶段 B: 分析管线(启发式 + AI 双通道,合并去重) =====
// 设计要点:AI 只产候选与分类,occurrences 一律本地复核;无 provider 时纯启发式降级。

use crate::state::DbState;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct VarCandidate {
    name: String,
    r#type: String,
    default_value: String,
    confidence: f64,
    semantic: String,
    source: String,
    occurrences: Vec<serde_json::Value>,
}

fn to_json(c: &VarCandidate) -> serde_json::Value {
    serde_json::json!({
        "name": c.name, "type": c.r#type, "defaultValue": c.default_value,
        "confidence": (c.confidence * 100.0).round() / 100.0,
        "semantic": c.semantic, "source": c.source, "occurrences": c.occurrences,
        "occurrenceCount": c.occurrences.iter().filter_map(|o| o["count"].as_u64()).sum::<u64>(),
    })
}

/// 值在文件集合中的出现统计(AI 候选不信任模型自报位置,本地复核)
fn count_occurrences(value: &str, files: &[(String, String)]) -> Vec<serde_json::Value> {
    files
        .iter()
        .filter(|(_, content)| content.contains(value))
        .map(|(path, content)| {
            serde_json::json!({ "path": path, "original": value, "count": content.matches(value).count() })
        })
        .collect()
}

/// 通道一:规则包 constants 正则(按值分组,同规则不同值拆成多候选)
fn heuristic_regex_candidates(pack: &RulePack, files: &[(String, String)]) -> Vec<VarCandidate> {
    let mut out = Vec::new();
    for rule in &pack.constants {
        let re = match regex::Regex::new(&rule.pattern) {
            Ok(r) => r,
            Err(_) => continue,
        };
        let mut groups: HashMap<String, Vec<serde_json::Value>> = HashMap::new();
        for (path, content) in files {
            let mut by_value: HashMap<String, u64> = HashMap::new();
            for cap in re.captures_iter(content) {
                if let Some(v) = cap.get(1) {
                    *by_value.entry(v.as_str().to_string()).or_insert(0) += 1;
                }
            }
            for (v, cnt) in by_value {
                groups
                    .entry(v.clone())
                    .or_default()
                    .push(serde_json::json!({ "path": path, "original": v, "count": cnt }));
            }
        }
        for (value, occ) in groups {
            out.push(VarCandidate {
                name: rule.var_name.clone(),
                r#type: rule.r#type.clone(),
                default_value: value,
                confidence: 0.7,
                semantic: rule.var_name.clone(),
                source: "heuristic".into(),
                occurrences: occ,
            });
        }
    }
    out
}

/// 通道二:引号字符串/目录名启发式(低置信,semantic=generic,待 AI 吸收或人工判断)
fn heuristic_string_candidates(root_name: &str, files: &[(String, String)]) -> Vec<VarCandidate> {
    super::ai::heuristic_candidates(root_name, files)
        .into_iter()
        .map(|(value, count)| {
            let is_identity = count == usize::MAX / 2;
            VarCandidate {
                name: super::ai::sanitize_var_name(&value, "var"),
                r#type: if value.chars().all(|c| c.is_ascii_digit()) { "number".into() } else { "string".into() },
                default_value: value.clone(),
                confidence: if is_identity { 0.6 } else { 0.4 },
                semantic: if is_identity { "identity".into() } else { "generic".into() },
                source: "heuristic".into(),
                occurrences: count_occurrences(&value, files),
            }
        })
        .collect()
}

const AI_BATCH_FILES: usize = 8;
const AI_BATCH_BYTES: usize = 48 * 1024;
const AI_MAX_BATCHES: usize = 12;
const ANALYZE_MAX_FILE: usize = 64 * 1024;
const ANALYZE_MAX_TOTAL: usize = 1024 * 1024;

/// 入口文件优先的批次切分
fn build_batches(files: &[(String, String)], entry_set: &[String]) -> Vec<Vec<(String, String)>> {
    let mut sorted: Vec<&(String, String)> = files.iter().collect();
    sorted.sort_by_key(|(p, _)| {
        let is_entry = entry_set.iter().any(|e| glob_match(e, p));
        (!is_entry, p.clone()) // 入口在前,其余按路径稳定排序
    });
    let mut batches: Vec<Vec<(String, String)>> = Vec::new();
    let mut cur: Vec<(String, String)> = Vec::new();
    let mut cur_bytes = 0usize;
    for (p, c) in sorted {
        let len = c.len();
        if !cur.is_empty() && (cur.len() >= AI_BATCH_FILES || cur_bytes + len > AI_BATCH_BYTES) {
            batches.push(std::mem::take(&mut cur));
            cur_bytes = 0;
            if batches.len() >= AI_MAX_BATCHES {
                break;
            }
        }
        cur_bytes += len;
        cur.push((p.clone(), c.clone()));
    }
    if !cur.is_empty() && batches.len() < AI_MAX_BATCHES {
        batches.push(cur);
    }
    batches
}

/// 从模型回复提取 JSON(容错:剥围栏、截首尾大括号)
fn parse_json_block(text: &str) -> Option<serde_json::Value> {
    let t = text.trim();
    let t = t
        .strip_prefix("```json")
        .or_else(|| t.strip_prefix("```"))
        .unwrap_or(t);
    let t = t.strip_suffix("```").unwrap_or(t);
    let start = t.find('{')?;
    let end = t.rfind('}')?;
    serde_json::from_str(&t[start..=end]).ok()
}

/// 语义相容才可跨通道融合:generic/identity 宽容,具体语义须一致
fn semantic_compatible(a: &str, b: &str) -> bool {
    a == b || a == "generic" || b == "generic"
        || (a == "identity" && b.contains("name")) || (b == "identity" && a.contains("name"))
}

/// 合并去重:同值且语义相容的候选融合(AI 命名优先,置信取 max,occurrences 按 path 归并)
fn merge_candidates(cands: Vec<VarCandidate>) -> Vec<VarCandidate> {
    let mut merged: Vec<VarCandidate> = Vec::new();
    for c in cands {
        // 跨通道(heuristic×ai)同值直接融合(AI 命名/语义为准);AI 之间才严格语义判定,
        // 因启发式语义标签是 varName(server_port)而 AI 是语义类(port),字符串天然不一致
        let hit = merged.iter_mut().find(|m| {
            let cross = (m.source == "ai") ^ (c.source == "ai");
            m.default_value == c.default_value
                && (semantic_compatible(&m.semantic, &c.semantic) || cross)
        });
        match hit {
            Some(m) => {
                let ai_over = c.source == "ai" && m.source != "ai";
                if ai_over {
                    m.name = c.name.clone();
                    m.semantic = c.semantic.clone();
                    m.r#type = c.r#type.clone();
                    m.source = "ai".into();
                }
                m.confidence = m.confidence.max(c.confidence);
                // occurrences 按 path 归并(保留计数更大者)
                for o in c.occurrences {
                    let p = o["path"].as_str().unwrap_or("");
                    let exist = m.occurrences.iter_mut().find(|e| e["path"].as_str() == Some(p));
                    match exist {
                        Some(e) => {
                            if o["count"].as_u64() > e["count"].as_u64() {
                                *e = o;
                            }
                        }
                        None => m.occurrences.push(o),
                    }
                }
            }
            None => merged.push(c),
        }
    }
    merged.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
    merged
}

/// 分析镜像内 keep 文件:产出候选变量与 AI 文件分类;provider 缺省时纯启发式降级
#[tauri::command]
pub async fn convert_analyze(
    root: String,
    files: Vec<String>,
    provider: Option<String>,
    model: Option<String>,
    thinking: Option<String>,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let root_path = PathBuf::from(&root);
    if !root_path.is_dir() {
        return Err("镜像目录不存在".to_string());
    }
    let packs = all_packs();
    let pack = detect_pack(&root_path, &packs).ok_or("未识别出项目类型,请先 convert_scan")?;
    let root_name = root_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    // 读取文件(防路径穿越;单文件/总量截断)
    let mut loaded: Vec<(String, String)> = Vec::new();
    let mut total = 0usize;
    for rel in files.iter() {
        if rel.contains("..") || Path::new(rel).is_absolute() {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(root_path.join(rel)) {
            let content = if content.len() > ANALYZE_MAX_FILE {
                content[..ANALYZE_MAX_FILE].to_string()
            } else {
                content
            };
            total += content.len();
            loaded.push((rel.clone(), content));
            if total >= ANALYZE_MAX_TOTAL {
                break;
            }
        }
    }
    if loaded.is_empty() {
        return Err("没有可分析的可读文件".to_string());
    }

    // 启发式双路
    let mut cands = heuristic_regex_candidates(&pack, &loaded);
    cands.extend(heuristic_string_candidates(&root_name, &loaded));

    // AI 通道(默认 provider;任一批失败静默降级已产出的启发式结果)
    let mut degraded = true;
    let mut ai_files: Vec<serde_json::Value> = Vec::new();
    let mut ai_batch_count = 0usize;
    if let Ok(target) = super::ai::resolve_call_target(database.as_ref(), provider, model).await {
        let extra = super::ai::thinking_extra(&target.provider_name, &target.model, thinking.as_deref());
        let batches = build_batches(&loaded, &pack.entry_files);
        let n = batches.len();
        for (i, batch) in batches.iter().enumerate() {
            let payload = serde_json::json!(batch
                .iter()
                .map(|(p, c)| serde_json::json!({ "path": p, "content": c }))
                .collect::<Vec<_>>())
            .to_string();
            let system = "你是项目模板化分析师。分析给定文件,给出模板转换建议。只输出 JSON,不要任何其他文本。";
            let json_spec = r#"{"files":[{"path":"...","action":"keep|exclude|templatize","reason":"一句话"}],"candidates":[{"name":"snake_case 变量名","type":"string|number","value":"字面原值","semantic":"port|host|identity|db|path|url|timeout|generic","confidence":0.0}]}"#;
            let user = format!(
                "项目类型:{pid}(目录名:{root_name})。第 {}/{n} 批文件:\n{payload}\n\n输出 JSON(字段严格如下):\n{json_spec}\n要求:只提可参数化的环境/身份/业务参数(端口/host/URL/项目名/数据库名/超时等);逻辑常量(状态码/协议版本/数学常数)不要提;name 必须语义化。",
                i + 1, pid = pack.id
            );
            let reply = match crate::ai_runtime::chat(&target, Some(system), &user, &[], extra.clone()).await {
                Ok(r) => r,
                Err(_) => break, // 网络/model 故障:停止后续批,保留已有结果
            };
            ai_batch_count += 1;
            if let Some(v) = parse_json_block(&reply) {
                if let Some(arr) = v["candidates"].as_array() {
                    for c in arr {
                        let value = c["value"].as_str().unwrap_or("").trim().to_string();
                        if value.len() < 2 || value.len() > 128 {
                            continue;
                        }
                        let occ = count_occurrences(&value, &loaded);
                        if occ.is_empty() {
                            continue; // 模型幻觉:值在文件中不存在,丢弃
                        }
                        let semantic = c["semantic"].as_str().unwrap_or("generic").to_lowercase();
                        cands.push(VarCandidate {
                            name: super::ai::sanitize_var_name(c["name"].as_str().unwrap_or(&value), &semantic),
                            r#type: c["type"].as_str().unwrap_or("string").to_string(),
                            default_value: value,
                            confidence: c["confidence"].as_f64().unwrap_or(0.5).clamp(0.0, 1.0),
                            semantic,
                            source: "ai".into(),
                            occurrences: occ,
                        });
                    }
                }
                if let Some(arr) = v["files"].as_array() {
                    ai_files.extend(arr.iter().cloned());
                }
            }
        }
        degraded = ai_batch_count == 0;
    }

    let variables: Vec<serde_json::Value> = merge_candidates(cands).iter().map(to_json).collect();
    Ok(serde_json::json!({
        "packId": pack.id,
        "rootName": root_name,
        "variables": variables,
        "fileClasses": ai_files,
        "degraded": degraded,
        "aiBatches": ai_batch_count,
        "analyzedFiles": loaded.len(),
    })
    .to_string())
}
