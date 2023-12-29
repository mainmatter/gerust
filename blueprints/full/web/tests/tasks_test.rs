use axum::body::Bytes;
use axum::response::Response;
use axum::{
    body::Body,
    http::{self, Method},
};
use hyper::StatusCode;
use {{crate_name}}_db::entities::tasks::{
    create as create_task, load as load_task, load_all as load_tasks, Task, TaskChangeset,
};
use {{crate_name}}_db::test_helpers::users::create as create_user;
use pacesetter::test::helpers::{request, DbTestContext};
use pacesetter_procs::db_test;
use serde_json::json;
use std::collections::HashMap;

mod common;

type TasksList = Vec<Task>;

#[db_test]
async fn test_get_tasks(context: &DbTestContext) {
    let task_changeset = TaskChangeset {
        description: String::from("Test Task"),
    };
    create_task(task_changeset, &context.db_pool).await.unwrap();

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
    assert_eq!(tasks.first().unwrap().description, "Test Task");
}

#[db_test]
async fn test_create_task_unauthorized(context: &DbTestContext) {
    let mut headers = HashMap::new();
    headers.insert(http::header::CONTENT_TYPE.as_str(), "application/json");

    let response = request(&context.app, "/tasks", headers, Body::empty(), Method::POST).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[db_test]
async fn test_create_task_invalid(context: &DbTestContext) {
    create_user(
        String::from("Test User"),
        String::from("s3kuR t0k3n!"),
        &context.db_pool,
    )
    .await
    .unwrap();

    let mut headers = HashMap::new();
    headers.insert(http::header::CONTENT_TYPE.as_str(), "application/json");
    headers.insert(http::header::AUTHORIZATION.as_str(), "s3kuR t0k3n!");

    let payload = json!(TaskChangeset {
        description: String::from("")
    });

    let response = request(
        &context.app,
        "/tasks",
        headers,
        Body::from(payload.to_string()),
        Method::POST,
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[db_test]
async fn test_create_task_authorized(context: &DbTestContext) {
    create_user(
        String::from("Test User"),
        String::from("s3kuR t0k3n!"),
        &context.db_pool,
    )
    .await
    .unwrap();

    let mut headers = HashMap::new();
    headers.insert(http::header::CONTENT_TYPE.as_str(), "application/json");
    headers.insert(http::header::AUTHORIZATION.as_str(), "s3kuR t0k3n!");

    let payload = json!(TaskChangeset {
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

    let task = load_task(task.id, &context.db_pool).await.unwrap();
    assert_eq!(task.description, "my task");
}

#[db_test]
async fn test_create_tasks_unauthorized(context: &DbTestContext) {
    let mut headers = HashMap::new();
    headers.insert(http::header::CONTENT_TYPE.as_str(), "application/json");

    let response = request(&context.app, "/tasks", headers, Body::empty(), Method::PUT).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[db_test]
async fn test_create_tasks_invalid(context: &DbTestContext) {
    create_user(
        String::from("Test User"),
        String::from("s3kuR t0k3n!"),
        &context.db_pool,
    )
    .await
    .unwrap();

    let mut headers = HashMap::new();
    headers.insert(http::header::CONTENT_TYPE.as_str(), "application/json");
    headers.insert(http::header::AUTHORIZATION.as_str(), "s3kuR t0k3n!");

    let payload = json!(vec![
        TaskChangeset {
            description: String::from("")
        },
        TaskChangeset {
            description: String::from("do something")
        }
    ]);

    let response = request(
        &context.app,
        "/tasks",
        headers,
        Body::from(payload.to_string()),
        Method::PUT,
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let tasks = load_tasks(&context.db_pool).await.unwrap();
    assert_eq!(tasks.len(), 0);
}

#[db_test]
async fn test_create_tasks_authorized(context: &DbTestContext) {
    create_user(
        String::from("Test User"),
        String::from("s3kuR t0k3n!"),
        &context.db_pool,
    )
    .await
    .unwrap();

    let mut headers = HashMap::new();
    headers.insert(http::header::CONTENT_TYPE.as_str(), "application/json");
    headers.insert(http::header::AUTHORIZATION.as_str(), "s3kuR t0k3n!");

    let payload = json!(vec![
        TaskChangeset {
            description: String::from("my task")
        },
        TaskChangeset {
            description: String::from("my other task")
        }
    ]);

    let response = request(
        &context.app,
        "/tasks",
        headers,
        Body::from(payload.to_string()),
        Method::PUT,
    )
    .await;

    let tasks: Vec<Task> = json_body::<Vec<Task>>(response).await;
    assert_eq!(tasks.first().unwrap().description, "my task");
    assert_eq!(tasks.get(1).unwrap().description, "my other task");

    let tasks = load_tasks(&context.db_pool).await.unwrap();
    assert_eq!(tasks.len(), 2);
}

#[db_test]
async fn test_get_task(context: &DbTestContext) {
    let task_changeset = TaskChangeset {
        description: String::from("Test Task"),
    };
    let task = create_task(task_changeset, &context.db_pool).await.unwrap();
    let task_id = task.id;

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
