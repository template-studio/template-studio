/// API常量定义
pub struct ApiConstants;

impl ApiConstants {
    /// API版本
    pub const VERSION: &str = "v1";

    /// 分页默认设置
    pub const DEFAULT_PAGE: u32 = 1;
    pub const DEFAULT_PAGE_SIZE: u32 = 20;
    pub const MAX_PAGE_SIZE: u32 = 100;

    /// 模板类型
    pub const TEMPLATE_TYPE_BASIC: &str = "basic";
    pub const TEMPLATE_TYPE_SCAFFOLD: &str = "scaffold";
    pub const TEMPLATE_TYPE_DATA_DRIVEN: &str = "data_driven";

    /// 推荐状态
    pub const FEATURED_NO: i32 = 0;
    pub const FEATURED_YES: i32 = 1;

    /// 主要语言标识
    pub const PRIMARY_LANGUAGE_NO: i32 = 0;
    pub const PRIMARY_LANGUAGE_YES: i32 = 1;

    /// 热门语言标识
    pub const POPULAR_NO: i32 = 0;
    pub const POPULAR_YES: i32 = 1;
}