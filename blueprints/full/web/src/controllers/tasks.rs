use crate::state::AppState;
use axum::{extract::Path, extract::State, http::StatusCode, Json};
use {{crate_name}}_db::{entities::tasks, transaction, Error};
use pacesetter::web::internal_error;
use tracing::info;
use uuid::Uuid;

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

pub async fn create_task(
    State(app_state): State<AppState>,
    Json(task): Json<tasks::TaskChangeset>,
) -> Result<Json<tasks::Task>, (StatusCode, String)> {
    match transaction(&app_state.db_pool).await {
        Ok(mut tx) => match tasks::create(task, &mut *tx).await {
            Ok(task) => match tx.commit().await {
                Ok(_) => Ok(Json(task)),
                Err(e) => Err((internal_error(e), "".into())),
            },
            Err(Error::ValidationError(e)) => {
                info!(err.msg = %e, err.details = ?e, "Validation failed");
                Err((StatusCode::UNPROCESSABLE_ENTITY, e.to_string()))
            }
            Err(e) => Err((internal_error(e), "".into())),
        },
        Err(e) => Err((internal_error(e), "".into())),
    }
}
