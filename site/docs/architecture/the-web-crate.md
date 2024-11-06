---
sidebar_position: 1
---

# The `web` crate

The `web` crate contains the main [axum](https://crates.io/crates/axum) application, that implements the actual web interface. It contains the controllers with the implementations of the exposed endpoints, as well as any middlewares. The `web` crate also contains the application's main executable, which when starting up, will determine the environment the application runs in, load the configuration, initialize the app state, set up tracing and error handling, and bind the server to the configured interface.

The crate uses a simple folder structure:

```
web
├── controllers // Controllers implement request handlers for the exposed endpoints
├── middlewares // Tower middlewares for pre-processing requests before they are passed to the request handlers
├── lib.rs      // Code for starting up the server
├── main.rs     // Main entrypoint of the application
├── routes.rs   // Mapping of request handlers to routes
├── state.rs    // Definition and construction of the application state
└── tests       // Application tests
```

The `web` crate is a standard axum application that comes with a predefined file system layout and module organization. There is nothing that's specific to Gerust really – refer to the [axum](https://docs.rs/axum/latest/axum/) and [tower-http docs](https://docs.rs/tower-http/latest/tower_http/) for more detailed documentation on how to write controllers and middlewares.

## Testing

Testing Gerust applications is done via application tests that test the entire stack of the application, including middlewares, controller, as well as database access (if the project uses a database). Those tests live in the `web` crate.

Testing backends is typically straight forward: invoke a particular endpoint with a particular method and potentially query string and/or request body and assert the response is what you expect. However, things become more complicated when the server under test uses a database. The tests then need to seed the database with test data to establish a well-defined state for the test so assertions can be made. The database also needs to be cleaned up afterwards or better, isolated databases are used for the different tests so those can't interfere with each other. There are several mechanisms for ensuring that like transactions, cleanup scripts, etc.

Gerust uses an approach for test isolation that allows parallel execution of tests without adding a ton of complexity: every test runs in its own database. These test-specific databases are automatically created as copies of the main test database and destroyed after the test has completed. All that is made easily available via the `[db_test]` macro (see the [docs on the `macros` crate](#the-macros-crate) below) which passes a test context to each test which allows access both to the applicatoin under test as well as the database that this application is configured to connect to:

```rust
pub struct DbTestContext {
    /// The axum application that is being tested.
    pub app: Router,
    /// A connection pool connected to the test-specific database; the app is set up to use this database automatically
    pub db_pool: DbPool,
}

#[db_test]
async fn test_read_all(context: &DbTestContext) {
    let task_changeset: TaskChangeset = Faker.fake();
    create_task(task_changeset.clone(), &context.db_pool) // create a task in the database
        .await
        .unwrap();

    let response = context
        .app
        .request("/tasks")
        .method(Method::GET)
        .send()
        .await;                                           // load all tasks

    assert_that!(response.status(), eq(StatusCode::OK));

    let tasks: TasksList = response.into_body().into_json::<TasksList>().await;
    assert_that!(tasks, len(eq(1)));
    assert_that!(                                         // assert the task created above is returned (as the application uses the same database)
        tasks.first().unwrap().description,
        eq(task_changeset.description)
    );
}
```

The concept of changesets as well as the database access utilities like `create_task`, are explained in the [docs on the `db` crate](./the-db-crate).

### Test Helpers

The `web` crate has a feature `test-helpers` that is off by default bun on when running tests. Behind the feature flag is the `test_helpers` module that contains a number of extensions for e.g. `axum::Router` that allow for a simple way of making requests to the app in tests, e.g.:

```rust
let response = context
    .app
    .request("/tasks")
    .method(Method::GET)
    .send()
    .await;
```
