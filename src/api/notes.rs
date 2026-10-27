use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{core::error::AppError, state::AppState};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Note {
    pub id: String,
    pub owner_id: String,
    pub title: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateNoteRequest {
    pub title: String,
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNoteRequest {
    pub title: Option<String>,
    pub body: Option<String>,
}

fn extract_owner_id(headers: &HeaderMap) -> String {
    headers
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "test-owner".to_string())
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

pub async fn list_notes(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<Note>>, AppError> {
    let owner_id = extract_owner_id(&headers);
    let notes = sqlx::query_as::<_, Note>(
        "SELECT id, owner_id, title, body, created_at, updated_at FROM notes WHERE owner_id = $1 ORDER BY created_at DESC",
    )
    .bind(owner_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    Ok(Json(notes))
}

pub async fn get_note(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Note>, AppError> {
    let owner_id = extract_owner_id(&headers);
    let note = sqlx::query_as::<_, Note>(
        "SELECT id, owner_id, title, body, created_at, updated_at FROM notes WHERE id = $1 AND owner_id = $2",
    )
    .bind(id)
    .bind(owner_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?
    .ok_or_else(|| AppError::NotFound("Note not found".to_string()))?;

    Ok(Json(note))
}

pub async fn create_note(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateNoteRequest>,
) -> Result<(StatusCode, Json<Note>), AppError> {
    let owner_id = extract_owner_id(&headers);
    validate_title(&payload.title)?;
    validate_body(&payload.body)?;

    let id = Uuid::new_v4().to_string();
    let note = sqlx::query_as::<_, Note>(
        "INSERT INTO notes (id, owner_id, title, body, created_at, updated_at) VALUES ($1, $2, $3, $4, NOW(), NOW()) RETURNING id, owner_id, title, body, created_at, updated_at",
    )
    .bind(id)
    .bind(owner_id)
    .bind(payload.title.trim())
    .bind(payload.body.trim())
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?;

    Ok((StatusCode::CREATED, Json(note)))
}

pub async fn update_note(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(payload): Json<UpdateNoteRequest>,
) -> Result<Json<Note>, AppError> {
    let owner_id = extract_owner_id(&headers);

    if let Some(ref title) = payload.title {
        validate_title(title)?;
    }
    if let Some(ref body) = payload.body {
        validate_body(body)?;
    }

    let note = sqlx::query_as::<_, Note>(
        "UPDATE notes SET title = COALESCE($1, title), body = COALESCE($2, body), updated_at = NOW() WHERE id = $3 AND owner_id = $4 RETURNING id, owner_id, title, body, created_at, updated_at",
    )
    .bind(payload.title.as_deref().map(|s| s.trim()))
    .bind(payload.body.as_deref().map(|s| s.trim()))
    .bind(id)
    .bind(owner_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Internal(e.into()))?
    .ok_or_else(|| AppError::NotFound("Note not found".to_string()))?;

    Ok(Json(note))
}

pub async fn delete_note(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let owner_id = extract_owner_id(&headers);
    let result = sqlx::query("DELETE FROM notes WHERE id = $1 AND owner_id = $2")
        .bind(id)
        .bind(owner_id)
        .execute(&state.db)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Note not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}
