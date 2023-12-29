use axum::body::Bytes;
use axum::response::Response;
use axum::{
    body::Body,
    http::{self, Method},
};
use fake::{Fake, Faker};
use hyper::StatusCode;
use {{crate_name}}_db::entities::tasks::{
    create as create_task, load as load_task, load_all as load_tasks, Task, TaskChangeset,
};
use {{crate_name}}_db::test_helpers::users::{create as create_user, UserChangeset};
use pacesetter::test::helpers::{request, DbTestContext};
use pacesetter_procs::db_test;
use serde_json::json;
use std::collections::HashMap;

mod common;

type TasksList = Vec<Task>;

#[db_test]
async fn test_get_tasks(context: &DbTestContext) {
    let task_changeset: TaskChangeset = Faker.fake();
    create_task(task_changeset.clone(), &context.db_pool)
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
    assert_eq!(
        tasks.first().unwrap().description,
        task_changeset.description
    );
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
    let user_changeset: UserChangeset = Faker.fake();
    create_user(user_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let mut headers = HashMap::new();
    headers.insert(http::header::CONTENT_TYPE.as_str(), "application/json");
    headers.insert(http::header::AUTHORIZATION.as_str(), &user_changeset.token);

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
    let user_changeset: UserChangeset = Faker.fake();
    create_user(user_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let mut headers = HashMap::new();
    headers.insert(http::header::CONTENT_TYPE.as_str(), "application/json");
    headers.insert(http::header::AUTHORIZATION.as_str(), &user_changeset.token);

    let task_changeset: TaskChangeset = Faker.fake();
    let payload = json!(task_changeset);

    let response = request(
        &context.app,
        "/tasks",
        headers,
        Body::from(payload.to_string()),
        Method::POST,
    )
    .await;

    let task: Task = json_body::<Task>(response).await;
    assert_eq!(task.description, task_changeset.description);

    let task = load_task(task.id, &context.db_pool).await.unwrap();
    assert_eq!(task.description, task_changeset.description);
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
    let user_changeset: UserChangeset = Faker.fake();
    create_user(user_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let mut headers = HashMap::new();
    headers.insert(http::header::CONTENT_TYPE.as_str(), "application/json");
    headers.insert(http::header::AUTHORIZATION.as_str(), &user_changeset.token);

    let task_changeset: TaskChangeset = Faker.fake();
    let payload = json!(vec![
        TaskChangeset {
            description: String::from("")
        },
        task_changeset
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
    let user_changeset: UserChangeset = Faker.fake();
    create_user(user_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let mut headers = HashMap::new();
    headers.insert(http::header::CONTENT_TYPE.as_str(), "application/json");
    headers.insert(http::header::AUTHORIZATION.as_str(), &user_changeset.token);

    let task_changeset1: TaskChangeset = Faker.fake();
    let task_changeset2: TaskChangeset = Faker.fake();
    let payload = json!(vec![task_changeset1.clone(), task_changeset2.clone()]);

    let response = request(
        &context.app,
        "/tasks",
        headers,
        Body::from(payload.to_string()),
        Method::PUT,
    )
    .await;

    let tasks: Vec<Task> = json_body::<Vec<Task>>(response).await;
    assert_eq!(
        tasks.first().unwrap().description,
        task_changeset1.description
    );
    assert_eq!(
        tasks.get(1).unwrap().description,
        task_changeset2.description
    );

    let tasks = load_tasks(&context.db_pool).await.unwrap();
    assert_eq!(tasks.len(), 2);
}

#[db_test]
async fn test_get_task(context: &DbTestContext) {
    let task_changeset: TaskChangeset = Faker.fake();
    let task = create_task(task_changeset.clone(), &context.db_pool)
        .await
        .unwrap();
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
    assert_eq!(task.description, task_changeset.description);
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
