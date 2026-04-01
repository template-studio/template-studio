/// 存储常量定义
pub struct StorageConstants;

impl StorageConstants {
    /// 模板存储目录名
    pub const TEMPLATES_DIR: &str = "templates";

    /// Git目录名
    pub const GIT_DIR: &str = ".git";

    /// 元数据目录名
    pub const META_DIR: &str = ".meta";

    /// 源码目录名
    pub const SRC_DIR: &str = "src";

    /// 元数据文件名
    pub const TEMPLATE_META_FILE: &str = "template.json";
    pub const VARIABLES_META_FILE: &str = "variables.json";
    pub const CONFIG_META_FILE: &str = "config.json";

    /// Git默认分支
    pub const DEFAULT_BRANCH: &str = "main";
}