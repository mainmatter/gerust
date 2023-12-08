use crate::state::AppState;
use axum::{extract::Path, extract::State, http::StatusCode, Json};
use {{crate_name}}_db::entities::Task;
use pacesetter::web::internal_error;
use serde::Deserialize;
#[cfg(test)]
use serde::Serialize;
use tracing::info;
use uuid::Uuid;
use validator::Validate;

pub async fn get_tasks(State(app_state): State<AppState>) -> Result<Json<Vec<Task>>, StatusCode> {
    let tasks = sqlx::query_as!(Task, "SELECT id, description FROM tasks")
        .fetch_all(&app_state.db_pool)
        .await
        .map_err(internal_error)?;

    info!("responding with {:?}", tasks);

    Ok(Json(tasks))
}

pub async fn get_task(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Task>, StatusCode> {
    let task = sqlx::query_as!(Task, "SELECT id, description FROM tasks WHERE id = $1", id)
        .fetch_one(&app_state.db_pool)
        .await
        .map_err(internal_error)?;

    info!("responding with {:?}", task);

    Ok(Json(task))
}

#[derive(Deserialize, Validate)]
#[cfg_attr(test, derive(Serialize))]
pub struct CreateTask {
    #[validate(length(min = 1))]
    description: String,
}

pub async fn create_task(
    State(app_state): State<AppState>,
    Json(payload): Json<CreateTask>,
) -> Result<Json<Task>, (StatusCode, String)> {
    if let Err(e) = payload.validate() {
        info!(err.msg = %e, err.details = ?e, "Validation failed");
        return Err((StatusCode::UNPROCESSABLE_ENTITY, e.to_string()));
    }

    let description = payload.description;

    let record = sqlx::query!(
        "INSERT INTO tasks (description) VALUES ($1) RETURNING id",
        description
    )
    .fetch_one(&app_state.db_pool)
    .await
    .map_err(|e| (internal_error(e), "".into()))?;

    let id = record.id;

    let task = Task { id, description };

    Ok(Json(task))
}
