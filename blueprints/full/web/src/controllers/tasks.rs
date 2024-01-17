use crate::state::AppState;
use axum::{extract::Path, extract::State, http::StatusCode, Json};
use {{crate_name}}_db::{entities::tasks, transaction, Error};
use pacesetter::web::internal_error;
use tracing::info;
use uuid::Uuid;

pub async fn create(
    State(app_state): State<AppState>,
    Json(task): Json<tasks::TaskChangeset>,
) -> Result<(StatusCode, Json<tasks::Task>), (StatusCode, String)> {
    match tasks::create(task, &app_state.db_pool).await {
        Ok(task) => Ok((StatusCode::CREATED, Json(task))),
        Err(Error::ValidationError(e)) => {
            info!(err.msg = %e, err.details = ?e, "Validation failed");
            Err((StatusCode::UNPROCESSABLE_ENTITY, e.to_string()))
        }
        Err(e) => Err((internal_error(e), "".into())),
    }
}

pub async fn create_batch(
    State(app_state): State<AppState>,
    Json(tasks): Json<Vec<tasks::TaskChangeset>>,
) -> Result<(StatusCode, Json<Vec<tasks::Task>>), (StatusCode, String)> {
    match transaction(&app_state.db_pool).await {
        Ok(mut tx) => {
            let mut results: Vec<tasks::Task> = vec![];
            for task in tasks {
                match tasks::create(task, &mut *tx).await {
                    Ok(task) => results.push(task),
                    Err(Error::ValidationError(e)) => {
                        info!(err.msg = %e, err.details = ?e, "Validation failed");
                        return Err((StatusCode::UNPROCESSABLE_ENTITY, e.to_string()));
                    }
                    Err(e) => return Err((internal_error(e), "".into())),
                }
            }

            match tx.commit().await {
                Ok(_) => Ok((StatusCode::CREATED, Json(results))),
                Err(e) => Err((internal_error(e), "".into())),
            }
        }
        Err(e) => Err((internal_error(e), "".into())),
    }
}

pub async fn read_all(
    State(app_state): State<AppState>,
) -> Result<Json<Vec<tasks::Task>>, StatusCode> {
    let tasks = tasks::load_all(&app_state.db_pool)
        .await
        .map_err(internal_error)?;

    info!("responding with {:?}", tasks);

    Ok(Json(tasks))
}

pub async fn read_one(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<tasks::Task>, StatusCode> {
    match tasks::load(id, &app_state.db_pool).await {
        Ok(task) => Ok(Json(task)),
        Err(Error::NoRecordFound) => Err(StatusCode::NOT_FOUND),
        Err(e) => Err(internal_error(e)),
    }
}

pub async fn update(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(task): Json<tasks::TaskChangeset>,
) -> Result<Json<tasks::Task>, (StatusCode, String)> {
    match tasks::update(id, task, &app_state.db_pool).await {
        Ok(task) => Ok(Json(task)),
        Err(Error::NoRecordFound) => Err((StatusCode::NOT_FOUND, "".into())),
        Err(Error::ValidationError(e)) => {
            info!(err.msg = %e, err.details = ?e, "Validation failed");
            Err((StatusCode::UNPROCESSABLE_ENTITY, e.to_string()))
        }
        Err(e) => Err((internal_error(e), "".into())),
    }
}

pub async fn delete(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    match tasks::delete(id, &app_state.db_pool).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(Error::NoRecordFound) => Err((StatusCode::NOT_FOUND, "".into())),
        Err(e) => Err((internal_error(e), "".into())),
    }
}
