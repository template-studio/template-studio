/// 构建变量分析 prompt
pub fn build_analysis_prompt(template_content: &str) -> String {
    super::VARIABLE_ANALYSIS_PROMPT
        .replace("{template_content}", template_content)
}

/// 构建变量填充 prompt
pub fn build_fill_prompt(variables_json: &str, project_context: &str) -> String {
    super::VARIABLE_FILL_PROMPT
        .replace("{variables}", variables_json)
        .replace("{project_context}", project_context)
}
