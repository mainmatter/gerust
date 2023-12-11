use crate::state::AppState;
use axum::{extract::Path, extract::State, http::StatusCode, Json};
use {{crate_name}}_db::entities::tasks;
use pacesetter::web::internal_error;
use serde::Deserialize;
#[cfg(test)]
use serde::Serialize;
use tracing::info;
use uuid::Uuid;
use validator::Validate;

pub async fn get_tasks(
    State(app_state): State<AppState>,
) -> Result<Json<Vec<tasks::Task>>, StatusCode> {
    let tasks = tasks::load_all(&app_state.db_pool)
        .await
        .map_err(internal_error)?;

    info!("responding with {:?}", tasks);

    Ok(Json(tasks))
}

pub async fn get_task(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<tasks::Task>, StatusCode> {
    let task = tasks::load(id, &app_state.db_pool)
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
) -> Result<Json<tasks::Task>, (StatusCode, String)> {
    if let Err(e) = payload.validate() {
        info!(err.msg = %e, err.details = ?e, "Validation failed");
        return Err((StatusCode::UNPROCESSABLE_ENTITY, e.to_string()));
    }

    let task = tasks::create(payload.description, &app_state.db_pool)
        .await
        .map_err(|e| (internal_error(e), "".into()))?;

    Ok(Json(task))
}
