use crate::config::DatabaseConfig;
use axum::{
    body::Body,
    http::{Method, Request},
    response::Response,
    Router,
};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sqlx::postgres::{PgConnectOptions, PgConnection};
use sqlx::{ConnectOptions, Connection, Executor, PgPool};
use std::collections::HashMap;
use tower::ServiceExt;
use url::Url;

pub struct TestContext {
    pub app: Router,
    pub db_pool: PgPool,
    db_config: PgConnectOptions,
}

pub fn build_test_context(
    router: Router,
    db_pool: PgPool,
    test_db_config: PgConnectOptions,
) -> TestContext {
    TestContext {
        app: router,
        db_pool,
        db_config: test_db_config,
    }
}

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

pub async fn teardown(context: TestContext) {
    drop(context.app);
    drop(context.db_pool);

    let root_db_config = context.db_config.clone().database("postgres");
    let mut connection: PgConnection = Connection::connect_with(&root_db_config).await.unwrap();

    let test_db_name = context.db_config.get_database().unwrap();

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

fn build_test_db_name(base_name: &str) -> String {
    let test_db_suffix: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();
    format!("{}_{}", base_name, test_db_suffix).to_lowercase()
}

fn parse_db_config(url: &str) -> PgConnectOptions {
    let db_url = Url::parse(url).expect("Invalid DATABASE_URL!");
    ConnectOptions::from_url(&db_url).expect("Invalid DATABASE_URL!")
}
