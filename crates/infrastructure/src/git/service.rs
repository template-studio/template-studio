use crate::config::settings::GitConfig;
use anyhow::Result;
use git2::{Repository, RepositoryInitOptions, Signature};
use std::path::PathBuf;
use tracing::{error, info};

/// Git服务
pub struct GitService {
    config: GitConfig,
}

impl GitService {
    pub fn new(config: GitConfig) -> Self {
        Self { config }
    }

    /// 初始化Git仓库
    pub async fn init_repository(
        &self,
        repo_path: &PathBuf,
        template_name: &str,
        author_name: Option<&str>,
        author_email: Option<&str>,
    ) -> Result<()> {
        info!("正在初始化Git仓库: {:?}", repo_path);

        // 确保父目录存在
        if let Some(parent) = repo_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // 初始化仓库（git2 句柄不跨 await：init 后立即取 workdir 并 drop，
        // configure 内部需要时重开）
        match Repository::init_opts(repo_path, RepositoryInitOptions::new().bare(false)) {
            Ok(repo) => {
                info!("Git仓库初始化成功: {:?}", repo_path);
                let workdir = repo
                    .workdir()
                    .ok_or_else(|| anyhow::anyhow!("无法获取Git仓库工作目录"))?
                    .to_path_buf();
                drop(repo);
                self.configure_repository_at(&workdir, template_name, author_name, author_email)
                    .await
            }
            Err(e) => {
                error!("Git仓库初始化失败: {:?}, 路径: {:?}", e, repo_path);
                Err(anyhow::anyhow!("Git仓库初始化失败: {}", e))
            }
        }
    }

    /// 配置仓库
    ///
    /// git2::Repository 非 Send，不能跨 await 持有（否则外层 future 不满足
    /// axum Handler 的 Send 约束）——签名与工作目录先提取，文件创建的 await
    /// 段不持 repo，提交段重新打开仓库
    async fn configure_repository_at(
        &self,
        workdir: &std::path::Path,
        template_name: &str,
        author_name: Option<&str>,
        author_email: Option<&str>,
    ) -> Result<()> {
        // git2 的 Repository 与 Signature 都是非 Send 类型，不能跨 await 持有：
        // 先提取纯数据（名字/邮箱/路径），await 段只持 Send 数据，git 对象在同步段重建
        let name = author_name.unwrap_or("Template Studio");
        let email = author_email.unwrap_or("template@studio.local");
        let workdir = workdir.to_path_buf();

        // 创建.gitignore文件（在工作目录）
        self.create_gitignore(workdir.join(".gitignore")).await?;

        // 创建README.md文件（在工作目录）
        self.create_readme(workdir.join("README.md"), template_name)
            .await?;

        // 创建初始提交——同步段重建 Signature 与 Repository
        if self.config.auto_init {
            let signature = Signature::now(name, email)?;
            let repo = Repository::open(&workdir)?;
            self.create_initial_commit_sync(&repo, &signature, template_name)?;
        }

        info!("Git仓库配置完成: {:?}", workdir);
        Ok(())
    }

    /// 创建.gitignore文件
    async fn create_gitignore(&self, gitignore_path: PathBuf) -> Result<()> {
        let gitignore_content = r#"
# 构建输出
/target/
/dist/
/build/

# IDE文件
.vscode/
.idea/
*.swp
*.swo
*~

# 操作系统文件
.DS_Store
Thumbs.db

# 依赖目录
node_modules/
vendor/

# 日志文件
*.log
logs/

# 临时文件
*.tmp
*.temp

# 环境变量文件
.env
.env.local
.env.*.local

# 数据库文件
*.db
*.sqlite

# 缓存文件
.cache/

# 包管理器文件
Cargo.lock
package-lock.json
yarn.lock

# 压缩文件
*.tar.gz
*.zip
*.rar

# 可执行文件
*.exe
"#;

        tokio::fs::write(&gitignore_path, gitignore_content).await?;
        info!("创建.gitignore文件: {:?}", gitignore_path);
        Ok(())
    }

    /// 创建README.md文件
    async fn create_readme(&self, readme_path: PathBuf, template_name: &str) -> Result<()> {
        let readme_content = format!(
            r#"
# {}

## 描述

这是一个模板项目。

## 功能特性

- [功能1]
- [功能2]
- [功能3]

## 快速开始

### 前置要求

- [要求1]
- [要求2]

### 安装步骤

1. 步骤1
2. 步骤2
3. 步骤3

```bash
# 示例命令
cargo build
```

## 使用说明

### 基本用法

[详细的使用说明]

### 配置选项

[配置选项说明]

## 项目结构

```
src/
├── main.rs
├── lib.rs
└── modules/
```

## 贡献指南

1. Fork 本仓库
2. 创建你的特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交你的更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 打开一个 Pull Request

## 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情

## 联系方式

- 作者: Template Studio
- 邮箱: template@studio.local
- 链接: [项目地址]

## 版本历史

- 0.1.0
    - 初始版本
    - 基本功能实现
"#,
            template_name
        );

        tokio::fs::write(&readme_path, readme_content).await?;
        info!("创建README.md文件: {:?}", readme_path);
        Ok(())
    }

    /// 创建初始提交（同步版：内部无 await，供不跨 await 的调用方使用）
    fn create_initial_commit_sync(
        &self,
        repo: &Repository,
        signature: &Signature<'_>,
        template_name: &str,
    ) -> Result<()> {
        self.create_initial_commit_inner(repo, signature, template_name)
    }

    fn create_initial_commit_inner(
        &self,
        repo: &Repository,
        signature: &Signature<'_>,
        template_name: &str,
    ) -> Result<()> {
        let mut index = repo.index()?;

        // 添加所有文件到暂存区
        let pathspec = ["*"].iter().map(|s| s.to_string()).collect::<Vec<_>>();
        index.add_all(&pathspec, git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;

        // 创建树对象
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;

        // 获取父提交（如果是新仓库则为空）
        let parent_commit = match repo.head() {
            Ok(_) => Some(repo.head()?.peel_to_commit()?),
            Err(_) => None,
        };

        // 创建提交
        let parent_refs: Vec<&git2::Commit<'_>> = parent_commit.iter().collect();
        let commit_id = repo.commit(
            Some("HEAD"),
            signature,
            signature,
            &format!("Initial commit for template: {}", template_name),
            &tree,
            &parent_refs,
        )?;

        // 检查提交是否成功
        let commit = repo.find_commit(commit_id)?;
        info!(
            "创建初始提交: {} - {}",
            commit.id(),
            commit.message().unwrap_or("无消息")
        );
        Ok(())
    }

    /// 克隆仓库
    pub async fn clone_repository(&self, url: &str, target_path: &PathBuf) -> Result<()> {
        info!("正在克隆仓库: {} -> {}", url, target_path.display());

        // 确保目标目录的父目录存在
        if let Some(parent) = target_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        match Repository::clone(url, target_path) {
            Ok(_) => {
                info!("Git仓库克隆成功: {}", target_path.display());
                Ok(())
            }
            Err(e) => {
                error!(
                    "Git仓库克隆失败: {}, URL: {}, 路径: {}",
                    e,
                    url,
                    target_path.display()
                );
                Err(anyhow::anyhow!("Git仓库克隆失败: {}", e))
            }
        }
    }

    /// 检查是否为Git仓库
    pub fn is_repository(&self, path: &PathBuf) -> bool {
        Repository::open(path).is_ok()
    }

    /// 获取仓库状态
    pub fn get_repository_status(&self, path: &PathBuf) -> Result<Vec<String>> {
        let repo = Repository::open(path)?;
        let status = repo.statuses(None)?;

        let mut changes = Vec::new();
        for entry in status.iter() {
            if let Some(path) = entry.path() {
                let path_str = path.to_string();
                changes.push(path_str);
            }
        }

        Ok(changes)
    }

    /// 克隆并清理仓库（用于 Fork 功能）
    /// 将源模板复制到目标位置，清理原有 Git 信息，并初始化为新仓库
    pub async fn clone_and_clean(
        &self,
        source_path: &PathBuf,
        target_path: &PathBuf,
        new_name: &str,
        author_name: Option<&str>,
        author_email: Option<&str>,
    ) -> Result<()> {
        info!("正在Fork模板: {:?} -> {:?}", source_path, target_path);

        // 确保目标目录的父目录存在
        if let Some(parent) = target_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // 检查源路径是否为 Git 仓库。
        // git2::Repository 非 Send，不能跨 await 持有（否则 future 不满足 axum Handler
        // 的 Send 约束）——先取出工作目录路径再 drop
        let source_repo = Repository::open(source_path);

        if let Ok(repo) = source_repo {
            // 源是 Git 仓库，进行本地克隆
            info!("检测到源路径是Git仓库，执行本地克隆");
            let workdir = repo
                .workdir()
                .ok_or_else(|| anyhow::anyhow!("无法获取源仓库工作目录"))?
                .to_path_buf();
            drop(repo);
            self.copy_directory_recursive(&workdir, target_path).await?;

            // 清理原有的 Git 信息
            info!("清理原有的Git信息");
            self.clean_repository_info(target_path).await?;

            // 初始化新的 Git 仓库
            info!("初始化新的Git仓库");
            self.init_repository(target_path, new_name, author_name, author_email)
                .await?;
        } else {
            // 源不是 Git 仓库，直接复制目录
            info!("源路径不是Git仓库，直接复制目录");
            self.copy_directory_recursive(source_path, target_path)
                .await?;

            // 初始化新的 Git 仓库
            info!("初始化新的Git仓库");
            self.init_repository(target_path, new_name, author_name, author_email)
                .await?;
        }

        info!("Fork模板完成: {:?}", target_path);
        Ok(())
    }

    /// 清理仓库的 Git 信息（删除 .git 目录）
    async fn clean_repository_info(&self, repo_path: &PathBuf) -> Result<()> {
        let git_dir = repo_path.join(".git");

        if git_dir.exists() {
            tokio::fs::remove_dir_all(&git_dir).await?;
            info!("已删除.git目录: {:?}", git_dir);
        }

        Ok(())
    }

    /// 递归复制目录
    async fn copy_directory_recursive(&self, source: &PathBuf, target: &PathBuf) -> Result<()> {
        // 如果目标目录已存在，先删除
        if target.exists() {
            tokio::fs::remove_dir_all(target).await?;
        }

        // 创建目标目录
        tokio::fs::create_dir_all(target).await?;

        // 读取源目录
        let mut entries = tokio::fs::read_dir(source).await?;

        while let Some(entry) = entries.next_entry().await? {
            let source_path = entry.path();
            let file_name = source_path
                .file_name()
                .ok_or_else(|| anyhow::anyhow!("无法获取文件名"))?;
            let target_path = target.join(file_name);

            if source_path.is_dir() {
                // 递归复制子目录（使用 Box::pin 避免无限大小的 future）
                Box::pin(self.copy_directory_recursive(&source_path, &target_path)).await?;
            } else {
                // 复制文件
                tokio::fs::copy(&source_path, &target_path).await?;
            }
        }

        Ok(())
    }

    /// 还原文件到上次提交状态（git restore）
    pub async fn restore_file(&self, repo_path: &PathBuf, file_path: &str) -> Result<()> {
        info!("正在还原文件: {} (仓库: {:?})", file_path, repo_path);

        // 克隆用于闭包内部
        let repo_path_for_closure = repo_path.clone();
        let file_path_for_closure = file_path.to_string();

        // 使用 spawn_blocking 执行同步的 git 操作
        let result = tokio::task::spawn_blocking(move || -> Result<Option<Vec<u8>>> {
            let repo = Repository::open(&repo_path_for_closure)?;

            // 获取文件在 HEAD 中的内容
            let head = repo.head()?;
            let commit = head.peel_to_commit()?;
            let tree = commit.tree()?;

            // 查找文件在树中的条目
            let entry = tree.get_path(std::path::Path::new(&file_path_for_closure));

            match entry {
                Ok(entry) => {
                    // 文件存在于 HEAD 中，恢复到该版本
                    let blob = repo.find_blob(entry.id())?;
                    let content = blob.content().to_vec();
                    Ok(Some(content))
                }
                Err(_) => {
                    // 文件不存在于 HEAD 中（新增文件），标记为需要删除
                    Ok(None)
                }
            }
        })
        .await??;

        match result {
            Some(content) => {
                // 构建完整文件路径
                let full_path = repo_path.join(file_path);

                // 确保父目录存在
                if let Some(parent) = full_path.parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }

                // 写入文件
                tokio::fs::write(&full_path, &content).await?;
                info!("文件已还原到 HEAD 版本: {}", file_path);
            }
            None => {
                // 文件不存在于 HEAD 中（新增文件），删除它
                let full_path = repo_path.join(file_path);
                if full_path.exists() {
                    tokio::fs::remove_file(&full_path).await?;
                    info!("新增文件已删除: {}", file_path);
                }
            }
        }

        Ok(())
    }
}
