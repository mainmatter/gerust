use crate::config::DatabaseConfig;
use axum::{
    body::{Body, Bytes},
    http::{Method, Request},
    response::Response,
    Router,
};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use regex::{Captures, Regex};
use sqlx::postgres::{PgConnectOptions, PgConnection};
use sqlx::{Connection, Executor, PgPool};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tower::ServiceExt;

/// Context object for testing an application that provides access to the application instance that is being tested.
///
/// This is used together with the [`pacesetter-procs::test`] proc macro which will pass a `TestContext` into the test function as an argument:
///
/// ```rust
/// #[test]
/// async fn test_hello(context: &TestContext) {
///     let response = request(
///         &context.app,
///         "/greet",
///         HashMap::new(),
///         Body::empty(),
///         Method::GET,
///     )
///     .await;
/// 
///     let greeting: Greeting = response_body_json(response).await;
///     assert_eq!(greeting.hello, String::from("world"));
/// }
/// ```
pub struct TestContext {
    /// The application that is being tested.
    pub app: Router,
}

/// Context object for testing an application that provides access to the application instance that is being tested as well as an [`sqlx::PgPool`] that allows access to the test-specific database which is the same database that is also being used by the application.
///
/// This is used together with the [`pacesetter-procs::db_test`] proc macro which will pass a `DbTestContext` into the test function as an argument. The passed in database pool can be used to arrange the database before requesting the applicaton and also to assert the correct elements were created in the database:
///
/// ```rust
/// #[db_test]
/// async fn test_update_success(context: &DbTestContext) {
///     let task_changeset: TaskChangeset = Faker.fake();
///     let task = create_task(task_changeset.clone(), &context.db_pool)
///         .await
///         .unwrap(); // create a task in the database
/// 
///     let mut headers = HashMap::new();
///     headers.insert(http::header::CONTENT_TYPE.as_str(), "application/json");
/// 
///     let task_changeset: TaskChangeset = Faker.fake();
///     let payload = json!(task_changeset);
/// 
///     let response = request(
///         &context.app,
///         &format!("/tasks/{}", task.id),
///         headers,
///         Body::from(payload.to_string()),
///         Method::PUT,
///     )
///     .await; // update the task with new data
/// 
///     let task = load_task(task.id, &context.db_pool).await.unwrap();
///     assert_eq!(task.description, task_changeset.description); // assert the task was changed in the database
/// }
/// ```
///
/// In pacesetter, each test gets its own instance of the database so that all tests are isolated from each other. The database is created before the test starts as a copy of the main test database so that it will have all the migrations applied that the main test database has applied as well as all seed data loaded that was loaded into the main test database. The database is cleaned up automatically once the test completes.
pub struct DbTestContext {
    /// The application that is being tested.
    pub app: Router,
    /// A connection pool connected to the same database that the application that is being tested uses as well.
    pub db_pool: PgPool,
}

#[allow(missing_docs)]
pub fn build_test_context(router: Router) -> TestContext {
    TestContext { app: router }
}

#[allow(missing_docs)]
pub fn build_db_test_context(router: Router, db_pool: PgPool) -> DbTestContext {
    DbTestContext {
        app: router,
        db_pool,
    }
}

#[allow(missing_docs)]
// TODO: ideally, this should be serializing an updated PgConnectionOptions instead of using replace with regex
pub async fn prepare_db(config: &DatabaseConfig) -> DatabaseConfig {
    let db_config = parse_db_config(&config.url);
    let db_name = db_config.get_database().unwrap();

    let root_db_config = db_config.clone().database("postgres");
    let mut connection: PgConnection = Connection::connect_with(&root_db_config).await.unwrap();

    let test_db_name = build_test_db_name(db_name);

    let query = format!("CREATE DATABASE {} TEMPLATE {}", test_db_name, db_name);
    connection.execute(query.as_str()).await.unwrap();

    let regex = Regex::new(r"(.+)\/(.+$)").unwrap();
    let test_db_url = regex.replace(&config.url, |caps: &Captures| {
        format!("{}/{}", &caps[1], test_db_name)
    });

    DatabaseConfig {
        url: test_db_url.to_string(),
    }
}

#[allow(missing_docs)]
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

/// Helper function for making requests against the application that is being tested.
///
/// The URL, as well as headers, a request body, and request method can be specified.
///
/// Example
///
/// ```rust
///     let response = request(
///         &context.app,
///         "/greet",
///         HashMap::new(),
///         Body::empty(),
///         Method::GET,
///     )
///     .await;
/// 
///     let greeting = response_body(response).await;
///     assert_eq!(greeting, String::from("hi!"));
/// ```
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

/// Gets the response body from a JSON test response parsed into a struct T.
///
/// Example
///
/// ```rust
/// #[db_test]
/// async fn test_read_one_success(context: &DbTestContext) {
///     let task_changeset: TaskChangeset = Faker.fake();
///     let task = create_task(task_changeset.clone(), &context.db_pool)
///         .await
///         .unwrap(); // create a test task in the database
/// 
///     let response = request(
///         &context.app,
///         format!("/tasks/{}", task.id).as_str(),
///         HashMap::new(),
///         Body::empty(),
///         Method::GET,
///     )
///     .await; // load the task from the server
/// 
///     let task: Task = response_body_json::<Task>(response).await; // parse the task from the response
///     assert_eq!(task.description, task_changeset.description);
/// }
/// ```
pub async fn response_body_json<T>(response: Response<Body>) -> T
where
    T: serde::de::DeserializeOwned,
{
    let body = response_body(response).await;
    serde_json::from_slice::<T>(&body).expect("Failed to deserialize JSON body")
}

/// Gets the response body from a test response as [`axum::body::Bytes`].
///
/// Example
///
/// ```rust
///     let response = request(
///         &context.app,
///         "/greet",
///         HashMap::new(),
///         Body::empty(),
///         Method::GET,
///     )
///     .await;
/// 
///     let greeting = response_body(response).await;
///     assert_eq!(greeting, String::from("world"));
/// ```
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
