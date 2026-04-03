use axum::{
    extract::{State, Path, Multipart},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde_json::{json, Value};
use template_studio_shared::models::user::{LoginRequest, ChangePasswordRequest, RegisterRequest, UpdateProfileRequest};
use template_studio_shared::models::auth::AuthUser;
use template_studio_shared::models::pat::CreatePatRequest;
use validator::Validate;

pub type AppState = super::super::AppState;

/// 用户登录（公开接口，不需要认证）
pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if let Err(e) = request.validate() {
        return error_response(StatusCode::BAD_REQUEST, &e.to_string());
    }

    match state.auth_service.login(&request).await {
        Ok(resp) => Ok(Json(json!({
            "code": 200,
            "message": "登录成功",
            "result": resp
        }))),
        Err(e) => error_response(StatusCode::UNAUTHORIZED, &e.to_string()),
    }
}

/// 用户注册（公开接口，不需要认证）
pub async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if let Err(e) = request.validate() {
        return error_response(StatusCode::BAD_REQUEST, &e.to_string());
    }

    match state.auth_service.register(&request).await {
        Ok(resp) => Ok(Json(json!({
            "code": 200,
            "message": "注册成功",
            "result": resp
        }))),
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e.to_string()),
    }
}

/// 获取当前用户信息（需认证）
pub async fn get_info(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.auth_service.get_user_info(auth_user.user_id).await {
        Ok(info) => Ok(Json(json!({
            "code": 200,
            "result": info
        }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 修改密码
pub async fn change_password(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<ChangePasswordRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if let Err(e) = request.validate() {
        return error_response(StatusCode::BAD_REQUEST, &e.to_string());
    }

    match state.user_service.change_password(auth_user.user_id, &request).await {
        Ok(_) => Ok(Json(json!({
            "code": 200,
            "message": "密码修改成功"
        }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 创建 PAT 令牌
pub async fn create_pat(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<CreatePatRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.pat_service.create(auth_user.user_id, &request).await {
        Ok(resp) => Ok(Json(json!({
            "code": 200,
            "message": "令牌创建成功",
            "result": resp
        }))),
        Err(e) => error_response(StatusCode::BAD_REQUEST, &e.to_string()),
    }
}

/// 列出当前用户的 PAT 令牌
pub async fn list_pats(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.pat_service.list(auth_user.user_id).await {
        Ok(list) => Ok(Json(json!({
            "code": 200,
            "result": list
        }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 删除 PAT 令牌
pub async fn delete_pat(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<i64>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match state.pat_service.delete(id, auth_user.user_id).await {
        Ok(true) => Ok(Json(json!({
            "code": 200,
            "message": "令牌已删除"
        }))),
        Ok(false) => error_response(StatusCode::NOT_FOUND, "令牌不存在"),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

fn error_response(status: StatusCode, message: &str) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    Err((status, Json(json!({
        "code": status.as_u16(),
        "message": message
    }))))
}

/// 更新个人资料（bio）
pub async fn update_profile(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<UpdateProfileRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if let Err(e) = request.validate() {
        return error_response(StatusCode::BAD_REQUEST, &e.to_string());
    }
    match state.user_repository.update_profile(
        auth_user.user_id,
        request.bio.as_deref(),
        request.avatar.as_deref(),
    ).await {
        Ok(_) => Ok(Json(json!({ "code": 0, "message": "更新成功" }))),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

/// 上传头像
pub async fn upload_avatar(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    mut multipart: Multipart,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        (StatusCode::BAD_REQUEST, Json(json!({ "code": 400, "message": format!("上传失败: {}", e) })))
    })? {
        let filename = field.file_name().unwrap_or("avatar.png").to_string();
        let data = field.bytes().await.map_err(|e| {
            (StatusCode::BAD_REQUEST, Json(json!({ "code": 400, "message": format!("读取失败: {}", e) })))
        })?;

        if data.len() > 2 * 1024 * 1024 {
            return error_response(StatusCode::BAD_REQUEST, "头像文件不能超过2MB");
        }

        let ext = std::path::Path::new(&filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png");
        let avatar_filename = format!("{}_{}.{}", auth_user.user_id, chrono::Utc::now().timestamp(), ext);

        let avatar_dir = std::path::Path::new("data/avatars");
        std::fs::create_dir_all(avatar_dir).ok();
        let avatar_path = avatar_dir.join(&avatar_filename);
        std::fs::write(&avatar_path, &data).map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "code": 500, "message": format!("保存失败: {}", e) })))
        })?;

        let avatar_url = format!("/avatars/{}", avatar_filename);
        state.user_repository.update_profile(
            auth_user.user_id,
            None,
            Some(&avatar_url),
        ).await.map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "code": 500, "message": e.to_string() })))
        })?;

        return Ok(Json(json!({ "code": 0, "message": "上传成功", "data": { "avatar": avatar_url } })));
    }
    error_response(StatusCode::BAD_REQUEST, "未找到上传文件")
}

/// 获取公开用户主页
pub async fn public_profile(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let user = state.user_repository.find_public_by_username(&username).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "code": 500, "message": e.to_string() }))))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json(json!({ "code": 404, "message": "用户不存在" }))))?;

    // 获取该用户的公开模板
    let templates = state.template_repository.list_public_templates_by_owner(user.id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "code": 500, "message": e.to_string() }))))?;

    let mut templates_list = Vec::new();
    for tmpl in templates {
        let langs = state.template_repository.get_template_languages(tmpl.id).await.unwrap_or_default();
        templates_list.push(json!({
            "id": tmpl.id,
            "name": tmpl.name,
            "description": tmpl.description,
            "introduction": tmpl.introduction,
            "categoryId": tmpl.category_id,
            "isFeatured": tmpl.is_featured,
            "templateType": tmpl.template_type,
            "typeConfig": tmpl.type_config,
            "visibility": tmpl.visibility,
            "ownerName": tmpl.owner_name,
            "ownerAvatar": tmpl.owner_avatar,
            "downloadCount": tmpl.download_count,
            "createdAt": tmpl.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            "updatedAt": tmpl.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            "languages": langs,
        }));
    }

    Ok(Json(json!({
        "code": 0,
        "data": {
            "username": user.username,
            "avatar": user.avatar,
            "bio": user.bio,
            "createdAt": user.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            "templates": templates_list
        }
    })))
}
