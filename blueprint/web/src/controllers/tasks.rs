use crate::{error::Error, state::AppState};
use axum::{extract::Path, extract::State, http::StatusCode, Json};
use {{crate_name}}_db::{entities::tasks, transaction};
use payloads::*;
use tracing::info;
use uuid::Uuid;

/// Creates a task in the database.
///
/// This function creates a task in the database (see [`{{crate_name}}_db::entities::tasks::create`]) based on a [`{{crate_name}}_db::entities::tasks::TaskChangeset`] (sent as JSON). If the task is created successfully, a 201 response is returned with the created [`{{crate_name}}_db::entities::tasks::Task`]'s JSON representation in the response body. If the changeset is invalid, a 422 response is returned.
#[axum::debug_handler]
pub async fn create(
    State(app_state): State<AppState>,
    Json(payload): Json<CreateRequestPayload>,
) -> Result<(StatusCode, Json<CreateResponsePayload>), Error> {
    Ok(tasks::create(payload.into(), &app_state.db_pool)
        .await
        .map(|task| (StatusCode::CREATED, Json(task.into())))?)
}

/// Creates multiple tasks in the database.
///
/// This function creates multiple tasks in the database (see [`getest_db::entities::tasks::create`]) based on [`getest_db::entities::tasks::TaskChangeset`]s (sent as JSON). If all tasks are created successfully, a 201 response is returned with the created [`getest_db::entities::tasks::Task`]s' JSON representation in the response body. If any of the passed changesets is invalid, a 422 response is returned.
///
/// This function creates all tasks in a transaction so that either all are created successfully or none is.
#[axum::debug_handler]
pub async fn create_batch(
    State(app_state): State<AppState>,
    Json(payload): Json<CreateBatchRequestPayload>,
) -> Result<(StatusCode, Json<CreateBatchResponsePayload>), Error> {
    let mut tx = transaction(&app_state.db_pool).await?;

    let mut results: Vec<tasks::Task> = vec![];
    for task in Vec::<_>::from(payload) {
        let task = tasks::create(task, &mut *tx).await?;
        results.push(task);
    }

    tx.commit().await.map_err(anyhow::Error::from)?;

    Ok((StatusCode::CREATED, Json(results.into())))
}

/// Reads and responds with all the tasks currently present in the database.
///
/// This function reads all [`{{crate_name}}_db::entities::tasks::Task`]s from the database (see [`{{crate_name}}_db::entities::tasks::load_all`]) and responds with their JSON representations.
#[axum::debug_handler]
pub async fn read_all(State(app_state): State<AppState>) -> Result<Json<Vec<tasks::Task>>, Error> {
    let tasks = tasks::load_all(&app_state.db_pool).await?;

    info!("responding with {:?}", tasks);

    Ok(Json(tasks))
}

/// Reads and responds with a task identified by its ID.
///
/// This function reads one [`{{crate_name}}_db::entities::tasks::Task`] identified by its ID from the database (see [`{{crate_name}}_db::entities::tasks::load`]) and responds with its JSON representations. If no task is found for the ID, a 404 response is returned.
#[axum::debug_handler]
pub async fn read_one(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<tasks::Task>, Error> {
    let task = tasks::load(id, &app_state.db_pool).await?;
    Ok(Json(task))
}

/// Updates a task in the database.
///
/// This function updates a task identified by its ID in the database (see [`{{crate_name}}_db::entities::tasks::update`]) with the data from the passed [`{{crate_name}}_db::entities::tasks::TaskChangeset`] (sent as JSON). If the task is updated successfully, a 200 response is returned with the created [`{{crate_name}}_db::entities::tasks::Task`]'s JSON representation in the response body. If the changeset is invalid, a 422 response is returned.
#[axum::debug_handler]
pub async fn update(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateRequestPayload>,
) -> Result<Json<UpdateResponsePayload>, Error> {
    let task = tasks::update(id, payload.into(), &app_state.db_pool).await?;
    Ok(Json(task.into()))
}

/// Deletes a task identified by its ID from the database.
///
/// This function deletes one [`{{crate_name}}_db::entities::tasks::Task`] identified by the entity's id from the database (see [`{{crate_name}}_db::entities::tasks::delete`]) and responds with a 204 status code and empty response body. If no task is found for the ID, a 404 response is returned.
#[axum::debug_handler]
pub async fn delete(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, Error> {
    tasks::delete(id, &app_state.db_pool).await?;
    Ok(StatusCode::NO_CONTENT)
}

mod payloads {
    use {{crate_name}}_db::entities::tasks::{Task, TaskChangeset};
    use {{crate_name}}_macros::{batch_request_payload, request_payload, response_payload};

    #[derive(Debug)]
    #[request_payload]
    pub struct CreateRequestPayload(TaskChangeset);

    #[derive(Debug)]
    #[response_payload]
    pub struct CreateResponsePayload(Task);

    #[derive(Debug)]
    #[batch_request_payload]
    pub struct CreateBatchRequestPayload(Vec<TaskChangeset>);

    #[derive(Debug)]
    #[response_payload]
    pub struct CreateBatchResponsePayload(Vec<Task>);

    #[derive(Debug)]
    #[request_payload]
    pub struct UpdateRequestPayload(TaskChangeset);

    #[derive(Debug)]
    #[response_payload]
    pub struct UpdateResponsePayload(Task);
}
