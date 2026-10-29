use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    api::extract::{ApiJson, OwnerId},
    core::error::{AppError, ErrorEnvelope},
    state::AppState,
};

/// Column list shared by every query so `Note` and the SQL cannot drift apart.
macro_rules! note_columns {
    () => {
        "id, owner_id, title, body, created_at, updated_at"
    };
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct Note {
    pub id: String,
    pub owner_id: String,
    pub title: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateNoteRequest {
    pub title: String,
    pub body: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateNoteRequest {
    pub title: Option<String>,
    pub body: Option<String>,
}

fn note_not_found() -> AppError {
    AppError::NotFound("Note not found".to_string())
}

fn validate_title(title: &str) -> Result<(), AppError> {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return Err(AppError::BadRequest("Title cannot be empty".to_string()));
    }
    if trimmed.len() > 200 {
        return Err(AppError::BadRequest(
            "Title must not exceed 200 characters".to_string(),
        ));
    }
    Ok(())
}

fn validate_body(body: &str) -> Result<(), AppError> {
    if body.trim().is_empty() {
        return Err(AppError::BadRequest("Body cannot be empty".to_string()));
    }
    Ok(())
}

#[utoipa::path(
    get,
    path = "/notes",
    tag = "notes",
    security(("bearer" = []), ("identity_header" = [])),
    responses(
        (status = 200, description = "Notes owned by the caller", body = Vec<Note>),
        (status = 401, description = "Missing or invalid credentials", body = ErrorEnvelope),
    )
)]
pub async fn list_notes(
    State(state): State<AppState>,
    OwnerId(owner_id): OwnerId,
) -> Result<Json<Vec<Note>>, AppError> {
    let notes = sqlx::query_as::<_, Note>(concat!(
        "SELECT ",
        note_columns!(),
        " FROM notes WHERE owner_id = $1 ORDER BY created_at DESC"
    ))
    .bind(owner_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    Ok(Json(notes))
}

#[utoipa::path(
    get,
    path = "/notes/{id}",
    tag = "notes",
    params(("id" = String, Path, description = "Note identifier")),
    security(("bearer" = []), ("identity_header" = [])),
    responses(
        (status = 200, description = "The note", body = Note),
        (status = 401, description = "Missing or invalid credentials", body = ErrorEnvelope),
        (status = 404, description = "No note with this id for the caller", body = ErrorEnvelope),
    )
)]
pub async fn get_note(
    State(state): State<AppState>,
    OwnerId(owner_id): OwnerId,
    Path(id): Path<String>,
) -> Result<Json<Note>, AppError> {
    let note = sqlx::query_as::<_, Note>(concat!(
        "SELECT ",
        note_columns!(),
        " FROM notes WHERE id = $1 AND owner_id = $2"
    ))
    .bind(id)
    .bind(owner_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(note_not_found)?;

    Ok(Json(note))
}

#[utoipa::path(
    post,
    path = "/notes",
    tag = "notes",
    request_body = CreateNoteRequest,
    security(("bearer" = []), ("identity_header" = [])),
    responses(
        (status = 201, description = "The created note", body = Note),
        (status = 400, description = "Title or body failed validation", body = ErrorEnvelope),
        (status = 401, description = "Missing or invalid credentials", body = ErrorEnvelope),
    )
)]
pub async fn create_note(
    State(state): State<AppState>,
    OwnerId(owner_id): OwnerId,
    ApiJson(payload): ApiJson<CreateNoteRequest>,
) -> Result<(StatusCode, Json<Note>), AppError> {
    validate_title(&payload.title)?;
    validate_body(&payload.body)?;

    let id = Uuid::new_v4().to_string();
    let note = sqlx::query_as::<_, Note>(concat!(
        "INSERT INTO notes (",
        note_columns!(),
        ") VALUES ($1, $2, $3, $4, NOW(), NOW()) RETURNING ",
        note_columns!()
    ))
    .bind(id)
    .bind(owner_id)
    .bind(payload.title.trim())
    .bind(payload.body.trim())
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    Ok((StatusCode::CREATED, Json(note)))
}

#[utoipa::path(
    put,
    path = "/notes/{id}",
    tag = "notes",
    params(("id" = String, Path, description = "Note identifier")),
    request_body = UpdateNoteRequest,
    security(("bearer" = []), ("identity_header" = [])),
    responses(
        (status = 200, description = "The updated note", body = Note),
        (status = 400, description = "Title or body failed validation", body = ErrorEnvelope),
        (status = 401, description = "Missing or invalid credentials", body = ErrorEnvelope),
        (status = 404, description = "No note with this id for the caller", body = ErrorEnvelope),
    )
)]
pub async fn update_note(
    State(state): State<AppState>,
    OwnerId(owner_id): OwnerId,
    Path(id): Path<String>,
    ApiJson(payload): ApiJson<UpdateNoteRequest>,
) -> Result<Json<Note>, AppError> {
    if let Some(ref title) = payload.title {
        validate_title(title)?;
    }
    if let Some(ref body) = payload.body {
        validate_body(body)?;
    }

    let note = sqlx::query_as::<_, Note>(
        concat!("UPDATE notes SET title = COALESCE($1, title), body = COALESCE($2, body), updated_at = NOW() WHERE id = $3 AND owner_id = $4 RETURNING ", note_columns!()),
    )
    .bind(payload.title.as_deref().map(|s| s.trim()))
    .bind(payload.body.as_deref().map(|s| s.trim()))
    .bind(id)
    .bind(owner_id)
    .fetch_optional(&state.db)
    .await
    ?
    .ok_or_else(note_not_found)?;

    Ok(Json(note))
}

#[utoipa::path(
    delete,
    path = "/notes/{id}",
    tag = "notes",
    params(("id" = String, Path, description = "Note identifier")),
    security(("bearer" = []), ("identity_header" = [])),
    responses(
        (status = 204, description = "The note was deleted"),
        (status = 401, description = "Missing or invalid credentials", body = ErrorEnvelope),
        (status = 404, description = "No note with this id for the caller", body = ErrorEnvelope),
    )
)]
pub async fn delete_note(
    State(state): State<AppState>,
    OwnerId(owner_id): OwnerId,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let result = sqlx::query("DELETE FROM notes WHERE id = $1 AND owner_id = $2")
        .bind(id)
        .bind(owner_id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(note_not_found());
    }

    Ok(StatusCode::NO_CONTENT)
}
