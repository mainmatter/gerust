use crate::{state::AppState, internal_error};
use axum::{extract::Path, extract::State, http::StatusCode, Json};
use {{crate_name}}_db::{entities::tasks, transaction, Error};
use tracing::info;
use uuid::Uuid;

/// Creates a task in the database.
///
/// This function creates a task in the database (see [`{{crate_name}}_db::entities::tasks::create`]) based on a [`{{crate_name}}_db::entities::tasks::TaskChangeset`] (sent as JSON). If the task is created successfully, a 201 response is returned with the created [`{{crate_name}}_db::entities::tasks::Task`]'s JSON representation in the response body. If the changeset is invalid, a 422 response is returned.
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

/// Creates multiple tasks in the database.
///
/// This function creates multiple tasks in the database (see [`{{crate_name}}_db::entities::tasks::create`]) based on [`{{crate_name}}_db::entities::tasks::TaskChangeset`]s (sent as JSON). If all tasks are created successfully, a 201 response is returned with the created [`{{crate_name}}_db::entities::tasks::Task`]s' JSON representation in the response body. If any of the passed changesets is invalid, a 422 response is returned.
///
/// This function creates all tasks in a transaction so that either all are created successfully or none is.
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

/// Reads and responds with all the tasks currently present in the database.
///
/// This function reads all [`{{crate_name}}_db::entities::tasks::Task`]s from the database (see [`{{crate_name}}_db::entities::tasks::load_all`]) and responds with their JSON representations.
pub async fn read_all(
    State(app_state): State<AppState>,
) -> Result<Json<Vec<tasks::Task>>, StatusCode> {
    let tasks = tasks::load_all(&app_state.db_pool)
        .await
        .map_err(internal_error)?;

    info!("responding with {:?}", tasks);

    Ok(Json(tasks))
}

/// Reads and responds with a task identified by its ID.
///
/// This function reads one [`{{crate_name}}_db::entities::tasks::Task`] identified by its ID from the database (see [`{{crate_name}}_db::entities::tasks::load`]) and responds with its JSON representations. If no task is found for the ID, a 404 response is returned.
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

/// Updates a task in the database.
///
/// This function updates a task identified by its ID in the database (see [`{{crate_name}}_db::entities::tasks::update`]) with the data from the passed [`{{crate_name}}_db::entities::tasks::TaskChangeset`] (sent as JSON). If the task is updated successfully, a 200 response is returned with the created [`{{crate_name}}_db::entities::tasks::Task`]'s JSON representation in the response body. If the changeset is invalid, a 422 response is returned.
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

/// Deletes a task identified by its ID from the database.
///
/// This function deletes one [`{{crate_name}}_db::entities::tasks::Task`] identified by the entity's id from the database (see [`{{crate_name}}_db::entities::tasks::delete`]) and responds with a 204 status code and empty response body. If no task is found for the ID, a 404 response is returned.
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
