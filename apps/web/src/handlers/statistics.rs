//! 统计处理器（真实聚合数据）
//!
//! 所有指标均来自数据库/文件系统的真实统计，替代早期演示用的伪造值。

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use template_studio_shared::utils::response::ApiResponse;

pub type AppState = super::super::AppState;

#[derive(Deserialize)]
pub struct UsageTrendsQuery {
    days: Option<i32>,
}

/// 总览：模板/分类/语言计数为真实值；文件总数取各模板最新发布版本 file_count 汇总
pub async fn get_overview(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let template_count = state
        .template_service
        .get_template_count()
        .await
        .unwrap_or(0);
    let category_count = state
        .category_service
        .get_category_count()
        .await
        .unwrap_or(0);
    let language_count = state
        .language_service
        .get_language_count()
        .await
        .unwrap_or(0);
    let total_files = state
        .template_service
        .get_total_published_files()
        .await
        .unwrap_or(0);

    Ok(Json(
        serde_json::to_value(ApiResponse::success_with_message(
            json!({
                "totalTemplates": template_count,
                "totalCategories": category_count,
                "totalLanguages": language_count,
                "totalFiles": total_files,
            }),
            "OK",
        ))
        .unwrap_or_default(),
    ))
}

/// 分类分布：GROUP BY category_id 真实聚合
pub async fn get_category_distribution(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let rows = state
        .template_service
        .get_category_distribution()
        .await
        .unwrap_or_default();
    let grand_total: i64 = rows.iter().map(|(_, c)| *c).sum();

    let items: Vec<Value> = rows
        .iter()
        .map(|(name, count)| {
            let pct = if grand_total > 0 {
                (*count as f64 * 100.0 / grand_total as f64).round() as i32
            } else {
                0
            };
            json!({
                "categoryName": name,
                "templateCount": count,
                "percentage": pct
            })
        })
        .collect();

    Ok(Json(
        serde_json::to_value(ApiResponse::success_with_message(
            json!({ "items": items }),
            "OK",
        ))
        .unwrap_or_default(),
    ))
}

/// 语言热度：template_languages JOIN languages 真实聚合
pub async fn get_language_popularity(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let rows = state
        .template_service
        .get_language_popularity()
        .await
        .unwrap_or_default();
    let grand_total: i64 = rows.iter().map(|(_, c)| *c).sum();

    let items: Vec<Value> = rows
        .iter()
        .map(|(name, count)| {
            let pct = if grand_total > 0 {
                (*count as f64 * 100.0 / grand_total as f64).round() as i32
            } else {
                0
            };
            json!({
                "languageName": name,
                "templateCount": count,
                "percentage": pct
            })
        })
        .collect();

    Ok(Json(
        serde_json::to_value(ApiResponse::success_with_message(
            json!({ "items": items }),
            "OK",
        ))
        .unwrap_or_default(),
    ))
}

/// 模板复杂度：按模板类型与变量定义数量真实分档
///
/// - 简单/中等/复杂：basic / scaffold / 其他（datadriven 等）
/// - 无变量 / 少变量 / 多变量：解析各模板 .meta/variables/variables.json 的字段数（0 / 1-10 / >10）
pub async fn get_template_complexity(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let stats = state
        .template_service
        .get_template_complexity_stats()
        .await
        .unwrap_or_default();

    Ok(Json(
        serde_json::to_value(ApiResponse::success_with_message(
            json!({
                "simpleTemplates": stats.simple,
                "mediumTemplates": stats.medium,
                "complexTemplates": stats.complex,
                "noVariableTemplates": stats.no_variable,
                "fewVariableTemplates": stats.few_variable,
                "manyVariableTemplates": stats.many_variable,
            }),
            "OK",
        ))
        .unwrap_or_default(),
    ))
}

/// 使用趋势：按模板创建日期真实聚合最近 N 天
pub async fn get_usage_trends(
    State(state): State<AppState>,
    Query(query): Query<UsageTrendsQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let days = query.days.unwrap_or(30).clamp(1, 365);
    let rows = state
        .template_service
        .get_daily_created_counts(days)
        .await
        .unwrap_or_default();
    let by_day: std::collections::HashMap<String, i64> = rows.into_iter().collect();

    // 补齐无创建记录的日期为 0，保持时间轴连续
    let items: Vec<Value> = (0..days)
        .rev()
        .map(|i| {
            let date = chrono::Utc::now() - chrono::Duration::days(i64::from(i));
            let key = date.format("%Y-%m-%d").to_string();
            json!({
                "date": key,
                "templateCreated": by_day.get(&key).copied().unwrap_or(0)
            })
        })
        .collect();

    Ok(Json(
        serde_json::to_value(ApiResponse::success_with_message(
            json!({ "items": items }),
            "OK",
        ))
        .unwrap_or_default(),
    ))
}
