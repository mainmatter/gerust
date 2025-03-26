use axum::{
    body::Body,
    http::{self, Method},
};
use fake::{Fake, Faker};
use googletest::prelude::*;
use hyper::StatusCode;
use {{db_crate_name}}::{entities, transaction, Error};
use {{macros_crate_name}}::db_test;
use {{web_crate_name}}::test_helpers::{BodyExt, DbTestContext, RouterExt};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

#[ignore = "not yet implemented"]
#[db_test]
async fn test_create_invalid(context: &DbTestContext) {
    let payload = json!(entities::{{entity_plural_name}}::{{entity_struct_name}}Changeset {
        name: String::from("")
    });

    let response = context
        .app
        .request("/{{entity_plural_name}}")
        .method(Method::POST)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::UNPROCESSABLE_ENTITY));
}

#[ignore = "not yet implemented"]
#[db_test]
async fn test_create_success(context: &DbTestContext) {
    let changeset: entities::{{entity_plural_name}}::{{entity_struct_name}}Changeset = Faker.fake();
    let payload = json!(changeset);

    let response = context
        .app
        .request("/{{entity_plural_name}}")
        .method(Method::POST)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::CREATED));

    let {{entity_plural_name}} = entities::{{entity_plural_name}}::load_all(&context.db_pool).await.unwrap();
    assert_that!({{entity_plural_name}}, len(eq(1)));
    assert_that!({{entity_plural_name}}.first().unwrap().name, eq(&changeset.name));
}

#[ignore = "not yet implemented"]
#[db_test]
async fn test_read_all(context: &DbTestContext) {
    let changeset: entities::{{entity_plural_name}}::{{entity_struct_name}}Changeset = Faker.fake();
    entities::{{entity_plural_name}}::create(changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let response = context.app.request("/{{entity_plural_name}}").send().await;

    assert_that!(response.status(), eq(StatusCode::OK));

    let {{entity_plural_name}}: Vec<entities::{{entity_plural_name}}::{{entity_struct_name}}> = response
        .into_body()
        .into_json::<Vec<entities::{{entity_plural_name}}::{{entity_struct_name}}>>()
        .await;
    assert_that!({{entity_plural_name}}, len(eq(1)));
    assert_that!({{entity_plural_name}}.first().unwrap().name, eq(&changeset.name));
}

#[ignore = "not yet implemented"]
#[db_test]
async fn test_read_one_nonexistent(context: &DbTestContext) {
    let response = context
        .app
        .request(&format!("/{{entity_plural_name}}/{}", Uuid::new_v4()))
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[ignore = "not yet implemented"]
#[db_test]
async fn test_read_one_success(context: &DbTestContext) {
    let {{entity_singular_name}}_changeset: entities::{{entity_plural_name}}::{{entity_struct_name}}Changeset = Faker.fake();
    let {{entity_singular_name}} = entities::{{entity_plural_name}}::create({{entity_singular_name}}_changeset.clone(), &context.db_pool)
        .await
        .unwrap();
    let {{entity_singular_name}}_id = {{entity_singular_name}}.id;

    let response = context
        .app
        .request(&format!("/{{entity_plural_name}}/{}", {{entity_singular_name}}_id))
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::OK));

    let {{entity_singular_name}}: entities::{{entity_plural_name}}::{{entity_struct_name}} = response
        .into_body()
        .into_json::<entities::{{entity_plural_name}}::{{entity_struct_name}}>()
        .await;
    assert_that!({{entity_singular_name}}.id, eq({{entity_singular_name}}_id));
    assert_that!({{entity_singular_name}}.name, eq(&{{entity_singular_name}}_changeset.name));
}

#[ignore = "not yet implemented"]
#[db_test]
async fn test_update_invalid(context: &DbTestContext) {
    let {{entity_singular_name}}_changeset: entities::{{entity_plural_name}}::{{entity_struct_name}}Changeset = Faker.fake();
    let {{entity_singular_name}} = entities::{{entity_plural_name}}::create({{entity_singular_name}}_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let payload = json!(entities::{{entity_plural_name}}::{{entity_struct_name}}Changeset {
        name: String::from("")
    });

    let response = context
        .app
        .request(&format!("/{{entity_plural_name}}/{}", {{entity_singular_name}}.id))
        .method(Method::PUT)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::UNPROCESSABLE_ENTITY));

    let {{entity_singular_name}}_after = entities::{{entity_plural_name}}::load({{entity_singular_name}}.id, &context.db_pool)
        .await
        .unwrap();
    assert_that!({{entity_singular_name}}_after.name, eq(&{{entity_singular_name}}.name));
}

#[ignore = "not yet implemented"]
#[db_test]
async fn test_update_nonexistent(context: &DbTestContext) {
    let {{entity_singular_name}}_changeset: entities::{{entity_plural_name}}::{{entity_struct_name}}Changeset = Faker.fake();
    let payload = json!({{entity_singular_name}}_changeset);

    let response = context
        .app
        .request(&format!("/{{entity_plural_name}}/{}", Uuid::new_v4()))
        .method(Method::PUT)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[ignore = "not yet implemented"]
#[db_test]
async fn test_update_success(context: &DbTestContext) {
    let {{entity_singular_name}}_changeset: entities::{{entity_plural_name}}::{{entity_struct_name}}Changeset = Faker.fake();
    let {{entity_singular_name}} = entities::{{entity_plural_name}}::create({{entity_singular_name}}_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let {{entity_singular_name}}_changeset: entities::{{entity_plural_name}}::{{entity_struct_name}}Changeset = Faker.fake();
    let payload = json!({{entity_singular_name}}_changeset);

    let response = context
        .app
        .request(&format!("/{{entity_plural_name}}/{}", {{entity_singular_name}}.id))
        .method(Method::PUT)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::OK));

    let {{entity_singular_name}}: entities::{{entity_plural_name}}::{{entity_struct_name}} = response
        .into_body()
        .into_json::<entities::{{entity_plural_name}}::{{entity_struct_name}}>()
        .await;
    assert_that!({{entity_singular_name}}.name, eq(&{{entity_singular_name}}_changeset.name.clone()));

    let {{entity_singular_name}} = entities::{{entity_plural_name}}::load({{entity_singular_name}}.id, &context.db_pool)
        .await
        .unwrap();
    assert_that!({{entity_singular_name}}.name, eq(&{{entity_singular_name}}_changeset.name));
}

#[ignore = "not yet implemented"]
#[db_test]
async fn test_delete_nonexistent(context: &DbTestContext) {
    let response = context
        .app
        .request(&format!("/{{entity_plural_name}}/{}", Uuid::new_v4()))
        .method(Method::DELETE)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[ignore = "not yet implemented"]
#[db_test]
async fn test_delete_success(context: &DbTestContext) {
    let {{entity_singular_name}}_changeset: entities::{{entity_plural_name}}::{{entity_struct_name}}Changeset = Faker.fake();
    let {{entity_singular_name}} = entities::{{entity_plural_name}}::create({{entity_singular_name}}_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let response = context
        .app
        .request(&format!("/{{entity_plural_name}}/{}", Uuid::new_v4()))
        .method(Method::DELETE)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NO_CONTENT));

    let result = entities::{{entity_plural_name}}::load({{entity_singular_name}}.id, &context.db_pool).await;
    assert_that!(result, err(anything()));
}
