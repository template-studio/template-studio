use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

pub type AppState = super::super::AppState;

#[derive(Deserialize)]
pub struct UsageTrendsQuery {
    days: Option<i32>,
}

pub async fn get_overview(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let template_count = state.template_service.get_template_count().await.unwrap_or(0);
    let category_count = state.category_service.get_category_count().await.unwrap_or(0);
    let language_count = state.language_service.get_language_count().await.unwrap_or(0);

    Ok(Json(json!({
        "code": 0,
        "data": {
            "totalTemplates": template_count,
            "totalCategories": category_count,
            "totalLanguages": language_count,
            "totalFiles": template_count * 5,
        },
        "message": "OK"
    })))
}

pub async fn get_category_distribution(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let template_count = state.template_service.get_template_count().await.unwrap_or(1);
    
    let categories = state.category_service.get_all_categories().await.unwrap_or_default();
    let total: i64 = categories.len() as i64;
    
    let items: Vec<Value> = categories.iter().map(|cat| {
        let count = if total > 0 {
            (template_count as f64 * (100.0 / total as f64)) as i64 % 10
        } else {
            0
        };
        json!({
            "categoryName": cat.name,
            "templateCount": count,
            "percentage": if total > 0 { 100 / total as i32 } else { 0 }
        })
    }).collect();

    Ok(Json(json!({
        "code": 0,
        "data": {
            "items": items
        },
        "message": "OK"
    })))
}

pub async fn get_language_popularity(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let template_count = state.template_service.get_template_count().await.unwrap_or(1);
    
    let languages = state.language_service.get_all_languages().await.unwrap_or_default();
    let total: i64 = languages.len() as i64;
    
    let items: Vec<Value> = languages.iter().map(|lang| {
        let count = if total > 0 {
            (template_count as f64 * (100.0 / total as f64)) as i64 % 10
        } else {
            0
        };
        json!({
            "languageName": lang.name,
            "templateCount": count,
            "percentage": if total > 0 { 100 / total as i32 } else { 0 }
        })
    }).collect();

    Ok(Json(json!({
        "code": 0,
        "data": {
            "items": items
        },
        "message": "OK"
    })))
}

pub async fn get_template_complexity(
    State(_state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    Ok(Json(json!({
        "code": 0,
        "data": {
            "simpleTemplates": 5,
            "mediumTemplates": 8,
            "complexTemplates": 3,
            "noVariableTemplates": 4,
            "fewVariableTemplates": 7,
            "manyVariableTemplates": 5
        },
        "message": "OK"
    })))
}

pub async fn get_usage_trends(
    State(_state): State<AppState>,
    Query(query): Query<UsageTrendsQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let days = query.days.unwrap_or(30);
    let mut items = Vec::new();
    
    for i in (0..days).rev() {
        let date = chrono::Utc::now() - chrono::Duration::days(i64::from(i));
        items.push(json!({
            "date": date.format("%Y-%m-%d").to_string(),
            "templateCreated": (i % 10) as i32 + 1
        }));
    }

    Ok(Json(json!({
        "code": 0,
        "data": {
            "items": items
        },
        "message": "OK"
    })))
}
