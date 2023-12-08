use axum::body::Bytes;
use axum::response::Response;
use axum::{
    body::Body,
    http::{self, Method},
};
use hyper::StatusCode;
use {{crate_name}}_db::entities::Task;
use pacesetter::test::helpers::{request, DbTestContext};
use pacesetter_procs::db_test;
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

mod common;

type TasksList = Vec<Task>;

#[db_test]
async fn test_get_tasks(context: &DbTestContext) {
    sqlx::query!(
        "INSERT INTO tasks (description) VALUES ($1) RETURNING id",
        "Test Task",
    )
    .fetch_one(&context.db_pool)
    .await
    .unwrap();

    let response = request(
        &context.app,
        "/tasks",
        HashMap::new(),
        Body::empty(),
        Method::GET,
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);

    let tasks: TasksList = json_body::<TasksList>(response).await;
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks.get(0).unwrap().description, "Test Task");
}

#[db_test]
async fn test_create_tasks_unauthorized(context: &DbTestContext) {
    let mut headers = HashMap::new();
    headers.insert(http::header::CONTENT_TYPE.as_str(), "application/json");

    let response = request(&context.app, "/tasks", headers, Body::empty(), Method::POST).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[db_test]
async fn test_create_tasks_authorized(context: &DbTestContext) {
    sqlx::query!(
        "INSERT INTO users (name, token) VALUES ($1, $2) RETURNING id",
        "Test User",
        "s3kuR t0k3n!",
    )
    .fetch_one(&context.db_pool)
    .await
    .unwrap();

    let mut headers = HashMap::new();
    headers.insert(http::header::CONTENT_TYPE.as_str(), "application/json");
    headers.insert(http::header::AUTHORIZATION.as_str(), "s3kuR t0k3n!");

    #[derive(Serialize)]
    struct CreateTask {
        description: String,
    }

    let payload = json!(CreateTask {
        description: String::from("my task")
    });

    let response = request(
        &context.app,
        "/tasks",
        headers,
        Body::from(payload.to_string()),
        Method::POST,
    )
    .await;

    let task: Task = json_body::<Task>(response).await;
    assert_eq!(task.description, "my task");
}

#[db_test]
async fn test_get_task(context: &DbTestContext) {
    let record = sqlx::query!(
        "INSERT INTO tasks (description) VALUES ($1) RETURNING id",
        "Test Task",
    )
    .fetch_one(&context.db_pool)
    .await
    .unwrap();
    let task_id: Uuid = record.id;

    let response = request(
        &context.app,
        format!("/tasks/{}", task_id).as_str(),
        HashMap::new(),
        Body::empty(),
        Method::GET,
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);

    let task: Task = json_body::<Task>(response).await;
    assert_eq!(task.id, task_id);
    assert_eq!(task.description, "Test Task");
}

async fn json_body<T>(response: Response<Body>) -> T
where
    T: serde::de::DeserializeOwned,
{
    let body = response_body(response).await;
    serde_json::from_slice::<T>(&body).expect("Failed to deserialize JSON body")
}

async fn response_body(response: Response<Body>) -> Bytes {
    // We don't care about the size limit in tests.
    axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body")
}
