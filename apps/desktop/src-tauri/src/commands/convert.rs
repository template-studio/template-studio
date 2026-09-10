//! 项目转模板:阶段 0 克隆镜像 + 阶段 A 规则扫描
//! 设计文档: dev-docs/project-to-template.md
//! 隔离原则: 所有来源统一 clone 到持久镜像(workspace/repos/<指纹>/),原始目录只读。

use std::path::{Path, PathBuf};
use tauri::Emitter;

/// 进度日志回调:(stage, text)。命令包装层转发为 convert://log 事件,测试传 no-op。
type ProgressLog<'a> = &'a (dyn Fn(&str, &str) + Send + Sync);

fn emit_log(app: &tauri::AppHandle, stage: &str, text: &str) {
    let _ = app.emit(
        "convert://log",
        serde_json::json!({ "stage": stage, "text": text }),
    );
}

// ===== 规则包(声明式,内置四份 + 用户目录覆盖) =====

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
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
    /// 构建验证命令(§13.1 存储前门禁;渲染产物落盘后执行)
    #[serde(default)]
    pub build_cmd: Option<String>,
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

/// 内置四份(include_str! 打包进二进制);BUILTIN_SOURCES 供规则视图回显原文
const BUILTIN_SOURCES: [(&str, &str); 5] = [
    ("node", include_str!("../../rules/node.json")),
    ("go", include_str!("../../rules/go.json")),
    ("java", include_str!("../../rules/java.json")),
    ("python", include_str!("../../rules/python.json")),
    ("rust", include_str!("../../rules/rust.json")),
];

fn builtin_packs() -> Vec<RulePack> {
    BUILTIN_SOURCES
        .iter()
        .filter_map(|(_, raw)| parse_pack(raw, "builtin"))
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

/// 流式 git:stderr 按 \r/\n 增量切块转发 progress(clone --progress 的下载进度在 stderr)。
/// 与 run_git 的差异:进度实时可见,而非结束后一次性返回。
fn run_git_streaming(
    args: &[&str],
    cwd: Option<&Path>,
    log: ProgressLog<'_>,
) -> Result<String, String> {
    use std::io::Read;
    let mut cmd = std::process::Command::new("git");
    cmd.args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    if let Some(c) = cwd {
        cmd.current_dir(c);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("无法启动 git(请确认已安装): {e}"))?;

    // stdout 单独线程排空,防止管道写满死锁(clone 的 stdout 很小)
    let stdout = child.stdout.take();
    let out_handle = std::thread::spawn(move || {
        let mut buf = String::new();
        if let Some(mut o) = stdout {
            let _ = o.read_to_string(&mut buf);
        }
        buf
    });

    let mut err_raw: Vec<u8> = Vec::new();
    let mut pending: Vec<u8> = Vec::new();
    if let Some(mut se) = child.stderr.take() {
        let mut chunk = [0u8; 4096];
        loop {
            match se.read(&mut chunk) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    err_raw.extend_from_slice(&chunk[..n]);
                    pending.extend_from_slice(&chunk[..n]);
                    while let Some(pos) = pending.iter().position(|&b| b == b'\r' || b == b'\n') {
                        let line: Vec<u8> = pending.drain(..=pos).collect();
                        let l = String::from_utf8_lossy(&line).trim().to_string();
                        if !l.is_empty() {
                            log("clone", &l);
                        }
                    }
                }
            }
        }
        if !pending.is_empty() {
            let l = String::from_utf8_lossy(&pending).trim().to_string();
            if !l.is_empty() {
                log("clone", &l);
            }
        }
    }

    let status = child.wait().map_err(|e| format!("等待 git 失败: {e}"))?;
    let stdout_text = out_handle.join().unwrap_or_default();
    if !status.success() {
        let stderr = String::from_utf8_lossy(&err_raw).trim().to_string();
        return Err(if stderr.is_empty() {
            format!("git {:?} 失败(退出码 {:?})", args, status.code())
        } else {
            stderr
        });
    }
    Ok(stdout_text.trim().to_string())
}

/// 镜像根目录 workspace/repos/
fn repos_dir() -> PathBuf {
    studio_home("workspace").join("repos")
}

/// 克隆来源到持久镜像:已存在则 fetch 增量并 fast-forward;返回 {dir, branch, commit, reused}
/// source 支持远程 URL(github/gitee/gitlab,凭据走本机 git 配置)与本地 git 仓库路径。
#[tauri::command]
pub async fn convert_clone(
    app: tauri::AppHandle,
    source: String,
    branch: Option<String>,
) -> Result<String, String> {
    let log = move |s: &str, t: &str| emit_log(&app, s, t);
    clone_impl(source, branch, &log).await
}

pub(crate) async fn clone_impl(
    source: String,
    branch: Option<String>,
    log: ProgressLog<'_>,
) -> Result<String, String> {
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
        log("clone", "镜像已存在,增量更新(fetch --all --prune)…");
        run_git(&["fetch", "--all", "--prune"], Some(&repo_dir))?;
        if let Some(b) = branch.as_deref().filter(|s| !s.is_empty()) {
            log("clone", &format!("切换分支 {b} 并对齐远端…"));
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
        let mut args: Vec<&str> = vec!["clone", "--progress"];
        if let Some(b) = branch.as_deref().filter(|s| !s.is_empty()) {
            args.extend(["--branch", b]);
        }
        // 完整 clone(保留历史,为项目更新留基线,见设计文档 §5.1)
        args.push(&src);
        let dir_str = repo_dir.to_string_lossy().to_string();
        args.push(&dir_str);
        log("clone", &format!("完整克隆到 {}…", repo_dir.display()));
        run_git_streaming(&args, None, log)?;
    }

    let commit = run_git(&["rev-parse", "HEAD"], Some(&repo_dir))?;
    let cur_branch = run_git(&["rev-parse", "--abbrev-ref", "HEAD"], Some(&repo_dir))?;
    log("clone", &format!("克隆完成:{cur_branch} @ {}", &commit[..7.min(commit.len())]));

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
pub async fn convert_scan(app: tauri::AppHandle, root: String) -> Result<String, String> {
    let log = move |s: &str, t: &str| emit_log(&app, s, t);
    scan_impl(root, &log).await
}

pub(crate) async fn scan_impl(root: String, log: ProgressLog<'_>) -> Result<String, String> {
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
    log("scan", &format!("识别项目类型:{}", pack.id));

    let mut files: Vec<serde_json::Value> = Vec::new();
    let mut stat = ScanStat { kept: 0, excluded: 0, total_size: 0 };
    walk_scan(&root_path, &root_path, &pack, &mut files, &mut stat)?;
    log(
        "scan",
        &format!(
            "扫描完成:共 {} 文件,保留 {} / 规则剔除 {}({:.1} MB)",
            stat.kept + stat.excluded,
            stat.kept,
            stat.excluded,
            stat.total_size as f64 / 1024.0 / 1024.0
        ),
    );

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

// ===== 阶段 C: 确定性替换(词边界+次数对账+渲染校验) =====

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// 词边界感知替换:返回(新内容, 合法替换数);8080 不会命中 18080 内部
fn replace_with_boundaries(content: &str, from: &str, to: &str) -> (String, usize) {
    let first_is_word = from.chars().next().map(is_word_char).unwrap_or(false);
    let last_is_word = from.chars().last().map(is_word_char).unwrap_or(false);
    let mut positions: Vec<usize> = Vec::new();
    let mut start = 0usize;
    while let Some(idx) = content[start..].find(from) {
        let pos = start + idx;
        let end = pos + from.len();
        let prev_ok = !first_is_word
            || pos == 0
            || !content[..pos].chars().next_back().is_some_and(is_word_char);
        let next_ok = !last_is_word
            || end >= content.len()
            || !content[end..].chars().next().is_some_and(is_word_char);
        if prev_ok && next_ok {
            positions.push(pos);
        }
        start = end.max(pos + 1);
    }
    let mut out = content.to_string();
    for &pos in positions.iter().rev() {
        out.replace_range(pos..pos + from.len(), to);
    }
    (out, positions.len())
}

/// 对确认后的变量表执行模板化:AI 零参与,纯确定性代码。
/// 输出 {outputs:[{path,content,replaced}], conflicts:[], warnings:[], validationErrors:[]}
#[tauri::command]
pub async fn convert_apply(
    app: tauri::AppHandle,
    root: String,
    files: Vec<String>,
    variables: Vec<serde_json::Value>,
) -> Result<String, String> {
    let log = move |s: &str, t: &str| emit_log(&app, s, t);
    apply_impl(root, files, variables, &log).await
}

pub(crate) async fn apply_impl(
    root: String,
    files: Vec<String>,
    variables: Vec<serde_json::Value>,
    log: ProgressLog<'_>,
) -> Result<String, String> {
    let root_path = PathBuf::from(&root);
    if !root_path.is_dir() {
        return Err("镜像目录不存在".to_string());
    }

    // 变量校验集(默认值注入渲染)
    let mut defaults = serde_json::Map::new();
    for v in &variables {
        let name = v["name"].as_str().unwrap_or("").to_string();
        let value = v["defaultValue"].as_str().unwrap_or("").to_string();
        if !name.is_empty() {
            defaults.insert(name, serde_json::Value::String(value));
        }
    }
    let vars_value = serde_json::Value::Object(defaults);

    // 文件级替换计划:path → [(original, placeholder, expected_count)]
    let mut plan: HashMap<String, Vec<(String, String, u64)>> = HashMap::new();
    for v in &variables {
        let name = v["name"].as_str().unwrap_or("");
        if name.is_empty() {
            continue;
        }
        let placeholder = format!("{{{{ {name} }}}}");
        if let Some(occ) = v["occurrences"].as_array() {
            for o in occ {
                let p = o["path"].as_str().unwrap_or("");
                let original = o["original"].as_str().unwrap_or("");
                if p.is_empty() || original.is_empty() {
                    continue;
                }
                let expected = o["count"].as_u64().unwrap_or(0);
                plan.entry(p.to_string())
                    .or_default()
                    .push((original.to_string(), placeholder.clone(), expected));
            }
        }
    }

    let mut outputs: Vec<serde_json::Value> = Vec::new();
    let mut conflicts: Vec<serde_json::Value> = Vec::new();
    let mut warnings: Vec<serde_json::Value> = Vec::new();
    let mut validation_errors: Vec<serde_json::Value> = Vec::new();

    for rel in &files {
        if rel.contains("..") || Path::new(rel).is_absolute() {
            continue;
        }
        let full = root_path.join(rel);
        let Ok(original_content) = std::fs::read_to_string(&full) else {
            conflicts.push(serde_json::json!({ "path": rel, "reason": "读取失败(二进制/编码)" }));
            continue;
        };
        let mut content = original_content.clone();
        let mut replaced_total = 0usize;

        if let Some(items) = plan.get(rel) {
            for (from, to, expected) in items {
                let raw_count = content.matches(from.as_str()).count() as u64;
                let (new_content, legal) = replace_with_boundaries(&content, from, to);
                if legal == 0 {
                    conflicts.push(serde_json::json!({
                        "path": rel, "original": from,
                        "reason": format!("词边界匹配 0 处(原文统计 {raw_count} 处,可能被更长 token 包含)"),
                    }));
                    continue; // 该值不动,原文保留
                }
                if legal as u64 != *expected {
                    warnings.push(serde_json::json!({
                        "path": rel, "original": from, "expected": expected, "actual": legal,
                        "note": "词边界过滤后与统计不一致(子串嵌套属正常),已替换全部合法位置",
                    }));
                }
                content = new_content;
                replaced_total += legal;
            }
        }

        // 渲染校验:默认值注入,模板化后语法必须仍可渲染
        if content != original_content {
            let rendered = super::template::render_string_content(content.clone(), vars_value.clone()).await;
            match rendered {
                Ok(v) if v["success"].as_bool().unwrap_or(false) => {}
                Ok(v) => validation_errors.push(serde_json::json!({
                    "path": rel,
                    "error": v["error"]["message"].as_str().unwrap_or("未知渲染错误"),
                })),
                Err(e) => validation_errors.push(serde_json::json!({ "path": rel, "error": e })),
            }
        }
        outputs.push(serde_json::json!({ "path": rel, "content": content, "replaced": replaced_total }));
    }

    log(
        "apply",
        &format!(
            "替换完成:{} 文件(其中 {} 文件有替换),冲突 {} / 警告 {} / 渲染失败 {}",
            outputs.len(),
            outputs.iter().filter(|o| o["replaced"].as_u64().unwrap_or(0) > 0).count(),
            conflicts.len(),
            warnings.len(),
            validation_errors.len()
        ),
    );

    Ok(serde_json::json!({
        "outputs": outputs, "conflicts": conflicts, "warnings": warnings,
        "validationErrors": validation_errors,
        "clean": conflicts.is_empty() && validation_errors.is_empty(),
    })
    .to_string())
}

/// 读取镜像内单个文件供前端预览(原文对照);拒绝二进制/超大/路径穿越
#[tauri::command]
pub async fn convert_read_file(root: String, path: String) -> Result<String, String> {
    let root_path = PathBuf::from(&root);
    if !root_path.is_dir() {
        return Err("镜像目录不存在".to_string());
    }
    if path.contains("..") || Path::new(&path).is_absolute() {
        return Err("非法路径".to_string());
    }
    const MAX_PREVIEW: u64 = 256 * 1024;
    let full = root_path.join(&path);
    let meta = std::fs::metadata(&full).map_err(|e| format!("读取失败: {e}"))?;
    if meta.len() > MAX_PREVIEW {
        return Err("文件超过 256 KB,不支持预览".to_string());
    }
    let content = std::fs::read_to_string(&full).map_err(|_| "二进制文件,不支持预览".to_string())?;
    Ok(serde_json::json!({ "path": path, "content": content }).to_string())
}

// ===== Agent 本地工具(转换宿主:镜像目录内执行,见设计文档 §12) =====

/// 相对路径合法性(防穿越/禁 .git 内部)
fn agent_path_ok(rel: &str) -> bool {
    !rel.is_empty() && !rel.contains("..") && !Path::new(rel).is_absolute() && !rel.starts_with(".git/") && !rel.contains("/.git/")
}

/// 列出镜像内文件(跳过 .git;含被剔除文件——agent 需见镜像真实状态)
#[tauri::command]
pub async fn convert_agent_list(root: String, sub: Option<String>) -> Result<String, String> {
    let root_path = PathBuf::from(&root);
    if !root_path.is_dir() {
        return Err("镜像目录不存在".to_string());
    }
    let mut base = root_path.clone();
    if let Some(s) = sub.as_deref().filter(|s| !s.is_empty()) {
        if !agent_path_ok(s) {
            return Err("非法路径".to_string());
        }
        base = base.join(s);
        if !base.is_dir() {
            return Err("子目录不存在".to_string());
        }
    }
    const MAX_ENTRIES: usize = 2000;
    let mut files: Vec<serde_json::Value> = Vec::new();
    let mut stack = vec![base];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name == ".git" {
                continue;
            }
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else {
                let rel = p
                    .strip_prefix(&root_path)
                    .map(|x| x.to_string_lossy().replace('\\', "/"))
                    .unwrap_or_default();
                let size = e.metadata().map(|m| m.len()).unwrap_or(0);
                files.push(serde_json::json!({ "path": rel, "size": size }));
                if files.len() >= MAX_ENTRIES {
                    return Ok(serde_json::json!({ "files": files, "truncated": true }).to_string());
                }
            }
        }
    }
    Ok(serde_json::json!({ "files": files, "truncated": false }).to_string())
}

/// agent 写文件。expect_hash 提供时做新鲜度守卫(§8.5):不匹配即拒,要求先读取。
#[tauri::command]
pub async fn convert_agent_write(
    root: String,
    path: String,
    content: String,
    expect_hash: Option<String>,
) -> Result<String, String> {
    let root_path = PathBuf::from(&root);
    if !root_path.is_dir() {
        return Err("镜像目录不存在".to_string());
    }
    if !agent_path_ok(&path) {
        return Err("非法路径".to_string());
    }
    let full = root_path.join(&path);
    if let Some(expected) = expect_hash.as_deref().filter(|s| !s.is_empty()) {
        match std::fs::read_to_string(&full) {
            Ok(cur) => {
                if fnv1a(&cur) != expected {
                    return Err("文件已变更(哈希不匹配),请先 read_file 获取最新内容再修改".to_string());
                }
            }
            Err(_) => return Err("文件不存在(或为二进制),新建文件不要传 expect_hash".to_string()),
        }
    }
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
    }
    std::fs::write(&full, &content).map_err(|e| format!("写入失败: {e}"))?;
    Ok(serde_json::json!({ "path": path, "hash": fnv1a(&content) }).to_string())
}

/// 按字符截断(防 UTF-8 边界 panic)
fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let t: String = s.chars().take(max).collect();
        format!("{t}\n…(输出已截断)")
    }
}

/// agent bash:cwd 钉死镜像;超时默认 30s 上限 120s;输出合并截断 64KB;push 类硬拦截。
/// 非零退出码不算错误(结果返回给 agent 自行判断)。
#[tauri::command]
pub async fn convert_agent_bash(
    root: String,
    command: String,
    timeout_ms: Option<u64>,
) -> Result<String, String> {
    let root_path = PathBuf::from(&root);
    if !root_path.is_dir() {
        return Err("镜像目录不存在".to_string());
    }
    let cmd = command.trim();
    if cmd.is_empty() {
        return Err("命令为空".to_string());
    }
    let lower = cmd.to_lowercase();
    for bad in ["git push", "git send-pack"] {
        if lower.contains(bad) {
            return Err(format!("已拦截:禁止 {bad}(镜像是本地副本,不允许推送远端)"));
        }
    }
    let timeout = timeout_ms.unwrap_or(30_000).min(120_000);

    let mut proc = std::process::Command::new("bash");
    proc.args(["-lc", cmd])
        .current_dir(&root_path)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        proc.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    let mut child = proc
        .spawn()
        .map_err(|_| "无法启动 bash:请确认 Git Bash 在 PATH 中(git 安装通常自带)".to_string())?;

    let out_pipe = child.stdout.take();
    let err_pipe = child.stderr.take();
    let out_handle = std::thread::spawn(move || {
        use std::io::Read;
        let mut buf = String::new();
        if let Some(mut o) = out_pipe {
            let _ = o.read_to_string(&mut buf);
        }
        buf
    });
    let err_handle = std::thread::spawn(move || {
        use std::io::Read;
        let mut buf = String::new();
        if let Some(mut e) = err_pipe {
            let _ = e.read_to_string(&mut buf);
        }
        buf
    });

    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout);
    let status = loop {
        match child.try_wait().map_err(|e| format!("等待进程失败: {e}"))? {
            Some(s) => break s,
            None if std::time::Instant::now() >= deadline => {
                let _ = child.kill();
                return Err(format!("命令超时({timeout}ms)已终止"));
            }
            None => std::thread::sleep(std::time::Duration::from_millis(50)),
        }
    };
    let stdout = out_handle.join().unwrap_or_default();
    let stderr = err_handle.join().unwrap_or_default();
    let combined = format!("{stdout}{stderr}");
    Ok(serde_json::json!({
        "exitCode": status.code().unwrap_or(-1),
        "output": truncate_chars(combined.trim(), 64 * 1024),
    })
    .to_string())
}

// ===== 规则包查看/覆盖(内置为底,用户目录同 id 覆盖) =====

fn rules_file(dir: &Path, id: &str) -> Result<PathBuf, String> {
    if id.is_empty() || id.contains("..") || id.contains('/') || id.contains('\\') {
        return Err("规则包 id 非法".to_string());
    }
    Ok(dir.join(format!("{id}.json")))
}

/// 读取规则包:内置原文 + 用户覆盖原文 + 当前生效(覆盖优先),均原样文本
#[tauri::command]
pub fn convert_rules_get(id: String) -> Result<String, String> {
    let builtin = builtin_packs()
        .into_iter()
        .find(|p| p.id == id)
        .ok_or_else(|| format!("内置规则包不存在: {id}"))?;
    let builtin_raw = match BUILTIN_SOURCES.iter().find(|(bid, _)| *bid == id) {
        Some((_, raw)) => *raw,
        None => "",
    };
    let dir = studio_home("rules");
    let override_raw = std::fs::read_to_string(rules_file(&dir, &id)?).ok();
    let effective = all_packs().into_iter().find(|p| p.id == id).unwrap_or(builtin);
    Ok(serde_json::json!({
        "id": id,
        "builtinRaw": builtin_raw,
        "overrideRaw": override_raw,
        "effective": serde_json::to_value(&effective).map_err(|e| e.to_string())?,
    })
    .to_string())
}

/// 保存覆盖:校验可解析为 RulePack 且 id 一致后,原子写入用户规则目录
#[tauri::command]
pub fn convert_rules_save(id: String, json: String) -> Result<(), String> {
    let pack: RulePack = serde_json::from_str(&json).map_err(|e| format!("JSON 不是合法规则包: {e}"))?;
    if pack.id != id {
        return Err(format!("规则包 id 不一致(正文 {} ≠ 目标 {id})", pack.id));
    }
    let path = rules_file(&studio_home("rules"), &id)?;
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p).map_err(|e| format!("创建规则目录失败: {e}"))?;
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, json).map_err(|e| format!("写入失败: {e}"))?;
    std::fs::rename(&tmp, &path).map_err(|e| format!("落盘失败: {e}"))?;
    Ok(())
}

/// 删除覆盖,恢复内置
#[tauri::command]
pub fn convert_rules_reset(id: String) -> Result<(), String> {
    let path = rules_file(&studio_home("rules"), &id)?;
    match std::fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("删除失败: {e}")),
    }
}

// ===== 构建验证(§13.1:渲染落盘 → buildCmd 冒烟,存储前可选门禁) =====

pub(crate) async fn build_check_impl(
    outputs: Vec<serde_json::Value>,
    variables: serde_json::Value,
    build_cmd: &str,
    log: ProgressLog<'_>,
) -> Result<String, String> {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let dir = studio_home("buildcheck").join(ts.to_string());
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建构建目录失败: {e}"))?;

    // 逐文件渲染落盘(失败即止——模板渲染不过,构建无从谈起)
    for o in &outputs {
        let path = o["path"].as_str().unwrap_or("").trim().to_string();
        let content = o["content"].as_str().unwrap_or("");
        if path.is_empty() || path.contains("..") || Path::new(&path).is_absolute() {
            continue;
        }
        let rendered = super::template::render_string_content(content.to_string(), variables.clone()).await;
        match rendered {
            Ok(v) if v["success"].as_bool().unwrap_or(false) => {
                let text = v["content"].as_str().unwrap_or("");
                let full = dir.join(&path);
                if let Some(p) = full.parent() {
                    std::fs::create_dir_all(p).map_err(|e| format!("创建目录失败: {e}"))?;
                }
                std::fs::write(&full, text).map_err(|e| format!("写入失败: {e}"))?;
            }
            Ok(v) => {
                let msg = v["error"]["message"].as_str().unwrap_or("未知渲染错误");
                return Ok(serde_json::json!({
                    "ok": false, "stage": "render", "path": path, "error": msg, "dir": dir.to_string_lossy(),
                })
                .to_string());
            }
            Err(e) => {
                return Ok(serde_json::json!({
                    "ok": false, "stage": "render", "path": path, "error": e, "dir": dir.to_string_lossy(),
                })
                .to_string());
            }
        }
    }
    log("build", &format!("渲染落盘 {} 文件,执行: {build_cmd}", outputs.len()));

    // 构建执行(与 agent bash 同防护语义:超时/截断;命令是我们配置的,无 push 风险)
    let t0 = std::time::Instant::now();
    let timeout = 240_000u64;
    let mut proc = std::process::Command::new("bash");
    proc.args(["-lc", build_cmd])
        .current_dir(&dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        proc.creation_flags(0x0800_0000);
    }
    let spawned = proc.spawn();
    let (exit_ok, code, combined) = match spawned {
        Err(e) => (false, -1, format!("无法启动 bash: {e}(请确认 Git Bash 在 PATH)")),
        Ok(mut child) => {
            let out_pipe = child.stdout.take();
            let err_pipe = child.stderr.take();
            let oh = std::thread::spawn(move || { use std::io::Read; let mut b = String::new(); if let Some(mut o) = out_pipe { let _ = o.read_to_string(&mut b); } b });
            let eh = std::thread::spawn(move || { use std::io::Read; let mut b = String::new(); if let Some(mut e) = err_pipe { let _ = e.read_to_string(&mut b); } b });
            let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout);
            let status = loop {
                match child.try_wait() {
                    Ok(Some(s)) => break Some(s),
                    Ok(None) if std::time::Instant::now() >= deadline => {
                        let _ = child.kill();
                        break None;
                    }
                    Ok(None) => std::thread::sleep(std::time::Duration::from_millis(50)),
                    Err(_) => break None,
                }
            };
            let combined = format!("{}{}", oh.join().unwrap_or_default(), eh.join().unwrap_or_default());
            match status {
                Some(s) => (s.success(), s.code().unwrap_or(-1), combined),
                None => (false, -1, format!("{combined}\n(构建超时 {timeout}ms 已终止)")),
            }
        }
    };
    let duration = t0.elapsed().as_millis();
    let ok = exit_ok;
    if ok {
        log("build", &format!("构建通过({duration}ms)"));
    } else {
        log("build", &format!("构建失败(exit {code},{duration}ms)"));
    }
    Ok(serde_json::json!({
        "ok": ok, "stage": "build", "command": build_cmd, "exitCode": code,
        "durationMs": duration, "output": truncate_chars(combined.trim(), 64 * 1024),
        "dir": dir.to_string_lossy(),
    })
    .to_string())
}

/// 存储前构建验证:outputs 按启用变量默认值渲染 → 落盘 buildcheck/<ts>/ → 规则包 buildCmd 冒烟
#[tauri::command]
pub async fn convert_build_check(
    app: tauri::AppHandle,
    outputs: Vec<serde_json::Value>,
    variables: serde_json::Value,
    pack_id: String,
) -> Result<String, String> {
    let log = move |s: &str, t: &str| emit_log(&app, s, t);
    let build_cmd = all_packs()
        .into_iter()
        .find(|p| p.id == pack_id)
        .and_then(|p| p.build_cmd)
        .ok_or_else(|| format!("技术栈 {pack_id} 未配置构建命令(可在规则包 buildCmd 字段补充)"))?;
    build_check_impl(outputs, variables, &build_cmd, &log).await
}

// ===== 草稿持久化(converts/<id>/,终稿前不进模板库) =====

fn convert_draft_dir(id: &str) -> Result<PathBuf, String> {
    let id = id.trim();
    if id.is_empty() || id.contains("..") || id.contains('/') || id.contains('\\') {
        return Err("草稿 id 非法".to_string());
    }
    let dir = studio_home("converts").join(id);
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建草稿目录失败: {e}"))?;
    Ok(dir)
}

/// 保存草稿(meta.json + ir.json,原子写)
#[tauri::command]
pub fn convert_draft_save(id: String, meta: serde_json::Value, ir: serde_json::Value) -> Result<(), String> {
    let dir = convert_draft_dir(&id)?;
    for (name, v) in [("meta.json", meta), ("ir.json", ir)] {
        let path = dir.join(name);
        let tmp = dir.join(format!("{name}.tmp"));
        let data = serde_json::to_string(&v).map_err(|e| format!("序列化失败: {e}"))?;
        std::fs::write(&tmp, data).map_err(|e| format!("写入失败: {e}"))?;
        std::fs::rename(&tmp, &path).map_err(|e| format!("落盘失败: {e}"))?;
    }
    Ok(())
}

/// 读取草稿
#[tauri::command]
pub fn convert_draft_load(id: String) -> Result<String, String> {
    let dir = convert_draft_dir(&id)?;
    let meta = std::fs::read_to_string(dir.join("meta.json")).map_err(|_| "草稿不存在".to_string())?;
    let ir = std::fs::read_to_string(dir.join("ir.json")).unwrap_or_else(|_| "{}".to_string());
    Ok(serde_json::json!({ "meta": serde_json::from_str::<serde_json::Value>(&meta).ok(), "ir": serde_json::from_str::<serde_json::Value>(&ir).ok() }).to_string())
}

/// 草稿列表(按更新时间倒序)
#[tauri::command]
pub fn convert_draft_list() -> Result<String, String> {
    let root = studio_home("converts");
    let mut items: Vec<serde_json::Value> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&root) {
        for e in entries.flatten() {
            let Ok(meta_raw) = std::fs::read_to_string(e.path().join("meta.json")) else { continue };
            let Ok(meta) = serde_json::from_str::<serde_json::Value>(&meta_raw) else { continue };
            let mtime = e
                .metadata()
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            items.push(serde_json::json!({ "id": e.file_name().to_string_lossy(), "meta": meta, "mtimeMs": mtime }));
        }
    }
    items.sort_by(|a, b| b["mtimeMs"].as_u64().unwrap_or(0).cmp(&a["mtimeMs"].as_u64().unwrap_or(0)));
    Ok(serde_json::json!({ "items": items }).to_string())
}

/// 删除草稿
#[tauri::command]
pub fn convert_draft_delete(id: String) -> Result<(), String> {
    let dir = convert_draft_dir(&id)?;
    let _ = std::fs::remove_dir_all(dir);
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

    #[test]
    fn word_boundary_replace() {
        let (out, n) = replace_with_boundaries("port=8080; x=18080; :8080", "8080", "{{ p }}");
        assert_eq!(n, 2, "18080 内部不算");
        assert!(out.contains("port={{ p }}"));
        assert!(out.contains("x=18080"));
        let (out2, n2) = replace_with_boundaries("a\"8080\"b", "8080", "X");
        assert_eq!(n2, 1, "引号是合法边界");
        assert_eq!(out2, "a\"X\"b");
        let (_, n3) = replace_with_boundaries("id=80801", "8080", "X");
        assert_eq!(n3, 0, "前缀紧贴数字不算");
    }

    #[tokio::test]
    async fn convert_apply_flow() {
        let tmp = std::env::temp_dir().join(format!("t126-apply-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(tmp.join("src")).unwrap();
        std::fs::write(tmp.join("src/main.rs"), "listen(8080);\nconst OLD: u16 = 8080;\nlet x = 18080;\n").unwrap();
        std::fs::write(tmp.join("README.md"), "# svc\nport 8080\n").unwrap();
        let vars = vec![serde_json::json!({
            "name": "server_port", "defaultValue": "8080",
            "occurrences": [
                {"path": "src/main.rs", "original": "8080", "count": 2},
                {"path": "README.md", "original": "8080", "count": 1}
            ]
        })];
        let out = apply_impl(
            tmp.to_string_lossy().to_string(),
            vec!["src/main.rs".to_string(), "README.md".to_string()],
            vars,
            &|_, _| {},
        )
        .await
        .unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert!(v["clean"].as_bool().unwrap(), "应无冲突无校验错误: {out}");
        let main = v["outputs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|f| f["path"].as_str() == Some("src/main.rs"))
            .unwrap();
        assert_eq!(main["replaced"].as_u64(), Some(2));
        let content = main["content"].as_str().unwrap();
        assert!(content.contains("{{ server_port }}"));
        assert!(content.contains("18080"), "嵌套长 token 不被误伤");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn draft_roundtrip() {
        let id = format!("t126-draft-{}", std::process::id());
        let meta = serde_json::json!({"source": "https://x/y", "branch": "main"});
        let ir = serde_json::json!({"files": [], "variables": []});
        convert_draft_save(id.clone(), meta, ir).unwrap();
        let raw = convert_draft_load(id.clone()).unwrap();
        let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(v["meta"]["source"].as_str(), Some("https://x/y"));
        let list = convert_draft_list().unwrap();
        let lv: serde_json::Value = serde_json::from_str(&list).unwrap();
        assert!(lv["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["id"].as_str() == Some(id.as_str())));
        convert_draft_delete(id.clone()).unwrap();
        assert!(convert_draft_load(id.clone()).is_err());
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
        let out = clone_impl(repo_root, None, &|_, _| {}).await.expect("clone 本仓库失败");
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert!(v["commit"].as_str().is_some_and(|c| c.len() >= 7), "缺基线 commit");
        assert!(v["branch"].as_str().is_some_and(|b| !b.is_empty()));

        let scan = scan_impl(v["dir"].as_str().unwrap().to_string(), &|_, _| {})
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

    #[tokio::test]
    async fn agent_write_hash_guard() {
        let tmp = std::env::temp_dir().join(format!("t137-w-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("a.txt"), "old").unwrap();
        let root = tmp.to_string_lossy().to_string();

        // 错误哈希 → 拒写并要求先读
        let err = convert_agent_write(root.clone(), "a.txt".into(), "new".into(), Some("deadbeef".into()))
            .await
            .unwrap_err();
        assert!(err.contains("read_file"), "应提示先读取: {err}");
        // 正确哈希 → 通过并返回新哈希
        let ok = convert_agent_write(root.clone(), "a.txt".into(), "new".into(), Some(fnv1a("old")))
            .await
            .unwrap();
        let v: serde_json::Value = serde_json::from_str(&ok).unwrap();
        assert_eq!(v["hash"].as_str(), Some(fnv1a("new")).as_deref());
        // 新建(无哈希,父目录自动创建)
        convert_agent_write(root, "b/c.txt".into(), "x".into(), None).await.unwrap();
        assert_eq!(std::fs::read_to_string(tmp.join("b/c.txt")).unwrap(), "x");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[tokio::test]
    async fn agent_list_skips_git() {
        let tmp = std::env::temp_dir().join(format!("t137-l-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(tmp.join(".git")).unwrap();
        std::fs::create_dir_all(tmp.join("src")).unwrap();
        std::fs::write(tmp.join(".git/config"), "x").unwrap();
        std::fs::write(tmp.join("src/main.rs"), "fn main(){}").unwrap();
        let out = convert_agent_list(tmp.to_string_lossy().to_string(), None).await.unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        let paths: Vec<&str> = v["files"].as_array().unwrap().iter().filter_map(|f| f["path"].as_str()).collect();
        assert!(paths.contains(&"src/main.rs"));
        assert!(!paths.iter().any(|p| p.starts_with(".git/")));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[tokio::test]
    async fn agent_bash_guards() {
        let tmp = std::env::temp_dir().join(format!("t137-b-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let root = tmp.to_string_lossy().to_string();

        // push 拦截
        let err = convert_agent_bash(root.clone(), "git push origin main".into(), None)
            .await
            .unwrap_err();
        assert!(err.contains("拦截"), "应拦截 push: {err}");

        // 正常执行:echo + 退出码
        let ok = convert_agent_bash(root.clone(), "echo agent-ok".into(), Some(20_000)).await.unwrap();
        let v: serde_json::Value = serde_json::from_str(&ok).unwrap();
        assert_eq!(v["exitCode"].as_i64(), Some(0));
        assert!(v["output"].as_str().unwrap_or("").contains("agent-ok"));

        // 超时终止
        let err = convert_agent_bash(root, "sleep 3".into(), Some(200)).await.unwrap_err();
        assert!(err.contains("超时"), "应超时: {err}");
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[tokio::test]
    async fn build_check_renders_and_runs() {
        let outputs = vec![serde_json::json!({ "path": "src/a.txt", "content": "hello {{ name }}" })];
        let vars = serde_json::json!({ "name": "world" });
        let out = build_check_impl(outputs, vars, "echo build-ok", &|_, _| {})
            .await
            .unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert!(v["ok"].as_bool().unwrap(), "应构建通过: {out}");
        assert_eq!(v["stage"].as_str(), Some("build"));
        assert!(v["output"].as_str().unwrap_or("").contains("build-ok"));
        // 渲染落盘内容正确(变量已注入)
        let dir = std::path::PathBuf::from(v["dir"].as_str().unwrap());
        assert_eq!(std::fs::read_to_string(dir.join("src/a.txt")).unwrap(), "hello world");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn build_check_render_failure_short_circuits() {
        let outputs = vec![serde_json::json!({ "path": "b.txt", "content": "{{ name }" })]; // 未闭合
        let out = build_check_impl(outputs, serde_json::json!({ "name": "x" }), "echo never", &|_, _| {})
            .await
            .unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert!(!v["ok"].as_bool().unwrap());
        assert_eq!(v["stage"].as_str(), Some("render"));
        let dir = std::path::PathBuf::from(v["dir"].as_str().unwrap());
        let _ = std::fs::remove_dir_all(&dir);
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
/// focus_files/expose_all_files:数据驱动模式的重点文件勾选与 AI 暴露范围控制
#[tauri::command]
pub async fn convert_analyze(
    app: tauri::AppHandle,
    root: String,
    files: Vec<String>,
    provider: Option<String>,
    model: Option<String>,
    thinking: Option<String>,
    focus_files: Option<Vec<String>>,
    expose_all_files: Option<bool>,
    database: tauri::State<'_, DbState>,
) -> Result<String, String> {
    let log = move |s: &str, t: &str| emit_log(&app, s, t);
    analyze_impl(root, files, provider, model, thinking, focus_files, expose_all_files, database.as_ref(), &log).await
}

pub(crate) async fn analyze_impl(
    root: String,
    files: Vec<String>,
    provider: Option<String>,
    model: Option<String>,
    thinking: Option<String>,
    focus_files: Option<Vec<String>>,
    expose_all_files: Option<bool>,
    database: &crate::database::Database,
    log: ProgressLog<'_>,
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
    log("analyze", &format!("读取 {} 个文件(约 {} KB,单文件/总量截断已应用)", loaded.len(), total / 1024));

    // 启发式双路
    let mut cands = heuristic_regex_candidates(&pack, &loaded);
    cands.extend(heuristic_string_candidates(&root_name, &loaded));
    log("analyze", &format!("启发式通道:规则正则 + 字符串扫描 → {} 个候选", cands.len()));

    // AI 通道(默认 provider;任一批失败静默降级已产出的启发式结果)
    // 暴露范围:聚焦模式仅重点文件内容进 AI 批次,但附全量目录结构;启发式通道始终全量(本地零成本)
    let mut degraded = true;
    let mut ai_files: Vec<serde_json::Value> = Vec::new();
    let mut ai_batch_count = 0usize;
    let focus: std::collections::HashSet<String> =
        focus_files.unwrap_or_default().into_iter().collect();
    let expose_all = expose_all_files.unwrap_or(true);
    let focus_only = !focus.is_empty() && !expose_all;

    let ai_source: Vec<(String, String)> = if focus_only {
        let picked: Vec<(String, String)> = loaded
            .iter()
            .filter(|(p, _)| focus.contains(p))
            .cloned()
            .collect();
        if picked.is_empty() {
            log("analyze", "勾选的重点文件均不可读,回退为全量内容分析");
            loaded.clone()
        } else {
            picked
        }
    } else {
        loaded.clone()
    };

    let focus_ctx = if focus_only {
        log(
            "analyze",
            &format!(
                "聚焦模式:仅 {} 个重点文件内容暴露给 AI,另附目录结构({} 条路径)",
                ai_source.len(),
                files.len()
            ),
        );
        let mut paths: Vec<&str> = files.iter().map(String::as_str).collect();
        paths.truncate(400);
        let more = if files.len() > paths.len() { format!("\n- …(共 {} 条,已截断)", files.len()) } else { String::new() };
        format!(
            "\n\n目录结构(未提供内容的文件仅列路径,供理解项目布局):\n{}{more}",
            paths.iter().map(|p| format!("- {p}")).collect::<Vec<_>>().join("\n")
        )
    } else if !focus.is_empty() {
        log("analyze", &format!("携带 {} 个用户重点文件标记进入 AI 通道(优先理解)", focus.len()));
        let mut picked: Vec<&String> = focus.iter().collect();
        picked.sort();
        format!(
            "\n\n用户重点文件(优先理解这些文件的分层与交互模式):\n{}",
            picked.iter().map(|p| format!("- {p}")).collect::<Vec<_>>().join("\n")
        )
    } else {
        String::new()
    };

    if let Ok(target) = super::ai::resolve_call_target(database, provider, model).await {
        let extra = super::ai::thinking_extra(&target.provider_name, &target.model, thinking.as_deref());
        let batches = build_batches(&ai_source, &pack.entry_files);
        let n = batches.len();
        log("analyze", &format!("AI 通道:{} · 共 {} 批(入口文件优先)", target.model, n));
        for (i, batch) in batches.iter().enumerate() {
            log("analyze", &format!("AI 批次 {}/{}:{} 个文件…", i + 1, n, batch.len()));
            let payload = serde_json::json!(batch
                .iter()
                .map(|(p, c)| serde_json::json!({ "path": p, "content": c }))
                .collect::<Vec<_>>())
            .to_string();
            let system = "你是项目模板化分析师。分析给定文件,给出模板转换建议。只输出 JSON,不要任何其他文本。";
            let json_spec = r#"{"files":[{"path":"...","action":"keep|exclude|templatize","reason":"一句话"}],"candidates":[{"name":"snake_case 变量名","type":"string|number","value":"字面原值","semantic":"port|host|identity|db|path|url|timeout|generic","confidence":0.0}]}"#;
            let user = format!(
                "项目类型:{pid}(目录名:{root_name})。第 {}/{n} 批文件:\n{payload}\n\n输出 JSON(字段严格如下):\n{json_spec}\n要求:只提可参数化的环境/身份/业务参数(端口/host/URL/项目名/数据库名/超时等);逻辑常量(状态码/协议版本/数学常数)不要提;name 必须语义化。{focus_ctx}",
                i + 1, pid = pack.id
            );
            let reply = match crate::ai_runtime::chat(&target, Some(system), &user, &[], extra.clone()).await {
                Ok(r) => r,
                Err(_) => break, // 网络/model 故障:停止后续批,保留已有结果
            };
            ai_batch_count += 1;
            if let Some(v) = parse_json_block(&reply) {
                let mut accepted = 0usize;
                let mut hallucinated = 0usize;
                if let Some(arr) = v["candidates"].as_array() {
                    for c in arr {
                        let value = c["value"].as_str().unwrap_or("").trim().to_string();
                        if value.len() < 2 || value.len() > 128 {
                            continue;
                        }
                        let occ = count_occurrences(&value, &loaded);
                        if occ.is_empty() {
                            hallucinated += 1;
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
                        accepted += 1;
                    }
                }
                if let Some(arr) = v["files"].as_array() {
                    ai_files.extend(arr.iter().cloned());
                }
                let dropped_note = if hallucinated > 0 { format!(",丢弃幻觉候选 {hallucinated}") } else { String::new() };
                log("analyze", &format!("AI 批次 {}/{} 完成:接受 {accepted} 候选{dropped_note}", i + 1, n));
            }
        }
        degraded = ai_batch_count == 0;
        if degraded {
            log("analyze", "AI 通道不可用(未配置或调用失败),结果为纯启发式(降级模式)");
        }
    }

    let variables: Vec<serde_json::Value> = merge_candidates(cands).iter().map(to_json).collect();
    log("analyze", &format!("双通道合并去重 → {} 个变量", variables.len()));
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
