use crate::config::DatabaseConfig;
use axum::{
    body::{Body, Bytes},
    http::{Method, Request},
    response::Response,
    Router,
};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sqlx::postgres::{PgConnectOptions, PgConnection};
use sqlx::{Connection, Executor, PgPool};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tower::ServiceExt;

pub struct TestContext {
    pub app: Router,
}

pub struct DbTestContext {
    pub app: Router,
    pub db_pool: PgPool,
}

pub fn build_test_context(router: Router) -> TestContext {
    TestContext { app: router }
}

pub fn build_db_test_context(router: Router, db_pool: PgPool) -> DbTestContext {
    DbTestContext {
        app: router,
        db_pool,
    }
}

// TODO: this should be returning a DatabaseConfig so that in the web crate's tests/common/mod.rs, we can use the db crate's connect_pool function to establish a connection (and drop the sqlx dependency from the web crate)
pub async fn prepare_db(config: &DatabaseConfig) -> PgConnectOptions {
    let db_config = parse_db_config(&config.url);
    let db_name = db_config.get_database().unwrap();

    let root_db_config = db_config.clone().database("postgres");
    let mut connection: PgConnection = Connection::connect_with(&root_db_config).await.unwrap();

    let test_db_name = build_test_db_name(db_name);

    let query = format!("CREATE DATABASE {} TEMPLATE {}", test_db_name, db_name);
    connection.execute(query.as_str()).await.unwrap();

    let test_db_config = db_config.clone();
    test_db_config.database(&test_db_name)
}

pub async fn teardown(context: DbTestContext) {
    drop(context.app);

    let mut connect_options = context.db_pool.connect_options();
    let db_config = Arc::make_mut(&mut connect_options);

    drop(context.db_pool);

    let root_db_config = db_config.clone().database("postgres");
    let mut connection: PgConnection = Connection::connect_with(&root_db_config).await.unwrap();

    let test_db_name = db_config.get_database().unwrap();

    let query = format!("DROP DATABASE IF EXISTS {}", test_db_name);
    connection.execute(query.as_str()).await.unwrap();
}

pub async fn request(
    app: &Router,
    uri: &str,
    headers: HashMap<&str, &str>,
    body: Body,
    method: Method,
) -> Response {
    let mut request_builder = Request::builder().uri(uri);

    for (key, value) in headers {
        request_builder = request_builder.header(key, value);
    }

    request_builder = request_builder.method(method);

    let request = request_builder.body(body);

    app.clone().oneshot(request.unwrap()).await.unwrap()
}

pub async fn response_body_json<T>(response: Response<Body>) -> T
where
    T: serde::de::DeserializeOwned,
{
    let body = response_body(response).await;
    serde_json::from_slice::<T>(&body).expect("Failed to deserialize JSON body")
}

pub async fn response_body(response: Response<Body>) -> Bytes {
    // We don't care about the size limit in tests.
    axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body")
}

fn build_test_db_name(base_name: &str) -> String {
    let test_db_suffix: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();
    format!("{}_{}", base_name, test_db_suffix).to_lowercase()
}

fn parse_db_config(url: &str) -> PgConnectOptions {
    PgConnectOptions::from_str(url).expect("Invalid DATABASE_URL!")
}
