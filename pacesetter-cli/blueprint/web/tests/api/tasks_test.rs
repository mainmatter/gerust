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
use pacesetter::test::helpers::{request, response_body_json, DbTestContext};
use pacesetter_procs::db_test;
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

use crate::common;

type TasksList = Vec<Task>;

#[db_test]
async fn test_create_unauthorized(context: &DbTestContext) {
    let mut headers = HashMap::new();
    headers.insert(http::header::CONTENT_TYPE.as_str(), "application/json");

    let response = request(&context.app, "/tasks", headers, Body::empty(), Method::POST).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[db_test]
async fn test_create_invalid(context: &DbTestContext) {
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
async fn test_create_success(context: &DbTestContext) {
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

    assert_eq!(response.status(), StatusCode::CREATED);

    let tasks = load_tasks(&context.db_pool).await.unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(
        tasks.first().unwrap().description,
        task_changeset.description
    );
}

#[db_test]
async fn test_create_batch_unauthorized(context: &DbTestContext) {
    let mut headers = HashMap::new();
    headers.insert(http::header::CONTENT_TYPE.as_str(), "application/json");

    let response = request(&context.app, "/tasks", headers, Body::empty(), Method::PUT).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[db_test]
async fn test_create_batch_invalid(context: &DbTestContext) {
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
async fn test_create_batch_success(context: &DbTestContext) {
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

    assert_eq!(response.status(), StatusCode::CREATED);

    let tasks: Vec<Task> = response_body_json::<Vec<Task>>(response).await;
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
async fn test_read_all(context: &DbTestContext) {
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

    let tasks: TasksList = response_body_json::<TasksList>(response).await;
    assert_eq!(tasks.len(), 1);
    assert_eq!(
        tasks.first().unwrap().description,
        task_changeset.description
    );
}

#[db_test]
async fn test_read_one_nonexistent(context: &DbTestContext) {
    let response = request(
        &context.app,
        format!("/tasks/{}", Uuid::new_v4()).as_str(),
        HashMap::new(),
        Body::empty(),
        Method::GET,
    )
    .await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[db_test]
async fn test_read_one_success(context: &DbTestContext) {
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

    let task: Task = response_body_json::<Task>(response).await;
    assert_eq!(task.id, task_id);
    assert_eq!(task.description, task_changeset.description);
}

#[db_test]
async fn test_update_unauthorized(context: &DbTestContext) {
    let mut headers = HashMap::new();
    headers.insert(http::header::CONTENT_TYPE.as_str(), "application/json");

    let task_changeset: TaskChangeset = Faker.fake();
    let task = create_task(task_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let response = request(
        &context.app,
        &format!("/tasks/{}", task.id),
        headers,
        Body::empty(),
        Method::PUT,
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[db_test]
async fn test_update_invalid(context: &DbTestContext) {
    let user_changeset: UserChangeset = Faker.fake();
    create_user(user_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let task_changeset: TaskChangeset = Faker.fake();
    let task = create_task(task_changeset.clone(), &context.db_pool)
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
        &format!("/tasks/{}", task.id),
        headers,
        Body::from(payload.to_string()),
        Method::PUT,
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let task_after = load_task(task.id, &context.db_pool).await.unwrap();
    assert_eq!(task_after.description, task_changeset.description);
}

#[db_test]
async fn test_update_update_nonexistent(context: &DbTestContext) {
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
        &format!("/tasks/{}", Uuid::new_v4()),
        headers,
        Body::from(payload.to_string()),
        Method::PUT,
    )
    .await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[db_test]
async fn test_update_success(context: &DbTestContext) {
    let user_changeset: UserChangeset = Faker.fake();
    create_user(user_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let task_changeset: TaskChangeset = Faker.fake();
    let task = create_task(task_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let mut headers = HashMap::new();
    headers.insert(http::header::CONTENT_TYPE.as_str(), "application/json");
    headers.insert(http::header::AUTHORIZATION.as_str(), &user_changeset.token);

    let task_changeset: TaskChangeset = Faker.fake();
    let payload = json!(task_changeset);

    let response = request(
        &context.app,
        &format!("/tasks/{}", task.id),
        headers,
        Body::from(payload.to_string()),
        Method::PUT,
    )
    .await;

    let task: Task = response_body_json::<Task>(response).await;
    assert_eq!(task.description, task_changeset.description);

    let task = load_task(task.id, &context.db_pool).await.unwrap();
    assert_eq!(task.description, task_changeset.description);
}

#[db_test]
async fn test_delete_unauthorized(context: &DbTestContext) {
    let task_changeset: TaskChangeset = Faker.fake();
    let task = create_task(task_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let response = request(
        &context.app,
        &format!("/tasks/{}", task.id),
        HashMap::new(),
        Body::empty(),
        Method::DELETE,
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[db_test]
async fn test_delete_nonexistent(context: &DbTestContext) {
    let user_changeset: UserChangeset = Faker.fake();
    create_user(user_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let mut headers = HashMap::<&str, &str>::new();
    headers.insert(http::header::AUTHORIZATION.as_str(), &user_changeset.token);

    let response = request(
        &context.app,
        &format!("/tasks/{}", Uuid::new_v4()),
        headers,
        Body::empty(),
        Method::DELETE,
    )
    .await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[db_test]
async fn test_delete_success(context: &DbTestContext) {
    let user_changeset: UserChangeset = Faker.fake();
    create_user(user_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let mut headers = HashMap::<&str, &str>::new();
    headers.insert(http::header::AUTHORIZATION.as_str(), &user_changeset.token);

    let task_changeset: TaskChangeset = Faker.fake();
    let task = create_task(task_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let response = request(
        &context.app,
        &format!("/tasks/{}", task.id),
        headers,
        Body::empty(),
        Method::DELETE,
    )
    .await;

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let result = load_task(task.id, &context.db_pool).await;
    assert!(result.is_err());
}
