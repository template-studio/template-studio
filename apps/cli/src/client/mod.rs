use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::debug;

// 使用共享的TemplateVersion模型
pub use template_studio_shared::models::release::TemplateVersion;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    #[serde(deserialize_with = "deserialize_id")]
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub introduction: Option<String>,
    #[serde(rename = "categoryId")]
    pub category_id: i64,
    #[serde(rename = "isFeatured")]
    pub is_featured: i32,
    pub logo: Option<String>,
    pub icon: Option<String>,
    #[serde(rename = "templateType")]
    pub template_type: Option<String>,
    #[serde(rename = "typeConfig")]
    pub type_config: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
    #[serde(rename = "currentVersion")]
    #[serde(default)]
    pub current_version: Option<String>,
    #[serde(default)]
    pub languages: Vec<TemplateLanguage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<Vec<TemplateVariable>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<TemplateFile>>,
}

// 自定义反序列化函数，支持整数和字符串ID
fn deserialize_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum IdValue {
        String(String),
        Integer(i64),
    }

    match IdValue::deserialize(deserializer)? {
        IdValue::String(s) => Ok(s),
        IdValue::Integer(i) => Ok(i.to_string()),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateLanguage {
    pub id: i64,
    #[serde(rename = "isPrimary")]
    pub is_primary: i32,
    #[serde(rename = "languageId")]
    pub language_id: i64,
    #[serde(rename = "templateId")]
    pub template_id: i64,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
    pub id: i64,
    pub template_id: i64,
    pub name: String,
    pub variable_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<String>,
    pub is_required: i32,
    pub sort: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateFile {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    pub is_directory: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderedFile {
    pub path: String,
    pub content: String,
    pub is_directory: bool,
}

pub struct ApiClient {
    base_url: String,
    api_key: String,
    http_client: Client,
}

impl ApiClient {
    pub fn new(base_url: &str, api_key: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            http_client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
        }
    }

    async fn request<T: for<'de> Deserialize<'de>>(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, endpoint);

        debug!("API请求: {} 参数: {:?}", url, params);

        let mut request = self.http_client.get(&url);
        for &(key, value) in params {
            if !value.is_empty() {
                request = request.query(&[(key, value)]);
            }
        }

        // 添加API密钥头
        if !self.api_key.is_empty() {
            request = request.header("Authorization", &format!("Bearer {}", self.api_key));
        }

        let response = request.send().await?;
        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("API请求失败: {} - {}", status, error_text);
        }

        let result: T = response.json().await?;
        Ok(result)
    }

    pub async fn list_templates(&self, category: Option<&str>) -> Result<Vec<Template>> {
        let mut params = vec![("templateType", "scaffold")];
        if let Some(cat) = category {
            params.push(("categoryId", cat));
        }

        // API返回格式: {code: 0, data: {templatesList: [...], total: N}, message: "OK"}
        #[derive(Deserialize)]
        struct ApiResponse {
            data: ListData,
        }

        #[derive(Deserialize)]
        struct ListData {
            #[serde(rename = "templatesList")]
            templates_list: Vec<Template>,
            total: i64,
        }

        let response: ApiResponse = self.request("/api/v1/studio/templates/list", &params).await?;
        debug!("获取到 {} 个模板", response.data.total);

        Ok(response.data.templates_list)
    }

    pub async fn get_template_info(&self, template_name: &str) -> Result<Template> {
        // 先尝试按名称搜索
        let templates = self.list_templates(None).await?;

        let template = templates.into_iter()
            .find(|t| t.name == template_name || t.id == template_name);

        if template.is_some() {
            return Ok(template.unwrap());
        }

        // 如果没有找到,重新获取模板列表进行模糊搜索
        let templates = self.list_templates(None).await?;
        let name_lower = template_name.to_lowercase();

        templates.into_iter()
            .find(|t| t.name.to_lowercase().contains(&name_lower))
            .ok_or_else(|| anyhow::anyhow!("未找到模板: {}", template_name))
    }

    pub async fn find_template(&self, template_identifier: &str) -> Result<Template> {
        self.get_template_info(template_identifier).await
    }

    pub async fn get_template_detail(&self, template_id: &str) -> Result<Template> {
        // 获取模板完整详情（包含variables和files）
        let url = format!("/api/v1/template/templates/detail?id={}", template_id);

        debug!("获取模板详情: {}", url);

        let response = self.http_client
            .get(format!("{}{}", self.base_url, url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("获取模板详情失败: {} - {}", status, error_text);
        }

        let response_text = response.text().await?;
        debug!("模板详情响应: {}", response_text);

        // API返回格式: {code: 0, data: Template, message: "OK"}
        #[derive(Deserialize)]
        struct DetailResponse {
            data: Template,
        }

        let detail_response: DetailResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow::anyhow!("解析模板详情失败: {}", e))?;

        Ok(detail_response.data)
    }

    pub async fn get_template_versions(&self, template_id: &str) -> Result<Vec<TemplateVersion>> {
        // 获取模板版本列表
        let url = format!("/api/v1/template/templates/{}/releases", template_id);

        debug!("获取模板版本列表: {}", url);

        #[derive(Deserialize)]
        struct ApiResponse {
            code: i32,
            data: VersionListData,
            message: String,
        }

        #[derive(Deserialize)]
        struct VersionListData {
            #[serde(rename = "templateId")]
            template_id: i64,
            versions: Vec<TemplateVersion>,
        }

        let response: ApiResponse = self.request(&url, &[]).await?;

        if response.code != 0 {
            anyhow::bail!("获取模板版本失败: {}", response.message);
        }

        Ok(response.data.versions)
    }

    pub async fn render_template(
        &self,
        template_id: &str,
        variables: &serde_json::Value,
    ) -> Result<Vec<RenderedFile>> {
        #[derive(Deserialize)]
        struct RenderResponse {
            data: Vec<RenderedFile>,
        }

        // TODO: 实现实际的渲染API调用
        // 这个API端点需要根据实际的后端实现调整
        let url = format!("/api/v1/editor/templates/{}/render", template_id);

        let response = self.http_client
            .post(format!("{}{}", self.base_url, url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(variables)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("渲染模板失败: {} - {}", status, error_text);
        }

        let render_response: RenderResponse = response.json().await?;
        Ok(render_response.data)
    }

    /// 下载模板版本ZIP文件
    pub async fn download_template_version(&self, template_id: &str, version: &str) -> Result<Vec<u8>> {
        let url = format!("/api/v1/template/templates/{}/releases/{}/download", template_id, version);

        debug!("下载模板版本: {}", url);

        let response = self.http_client
            .get(format!("{}{}", self.base_url, url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("下载模板版本失败: {} - {}", status, error_text);
        }

        let zip_bytes = response.bytes().await?;
        Ok(zip_bytes.to_vec())
    }
}
