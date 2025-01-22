---
sidebar_position: 2
---

# Adding Endpoints for Reading

In order to expose notes via a REST interface, we need a controller.

## Generating the Controller

Let's generate a controller that comes with example implementations for the <abbr>CRUD</abbr> (<dfn id="CRUD"><u>C</u>reate, <u>R</u>ead, <u>U</u>pdate, <u>D</u>elete</dfn>) methods:

```
cargo generate crud-controller notes
```

That generates a new controller in `web/src/controllers/notes.rs` with functions for creating, reading, updating, and deleting notes. For now, we only care about reading notes so we can uncomment the example code in the `read_all` and `read_one` functions and remove the `todo!`s:

```rust
use crate::{error::Error, state::SharedAppState};
use axum::{extract::Path, extract::State, http::StatusCode, Json};
use my_app_db::entities;
use tracing::info;
use uuid::Uuid;

…

#[axum::debug_handler]
pub async fn read_all(
    State(app_state): State<SharedAppState>,
) -> Result<Json<Vec<entities::notes::Note>>, Error> {
    let notes = entities::notes::load_all(&app_state.db_pool)
        .await?;

    info!("responding with {:?}", notes);

    Ok(Json(notes))
}

#[axum::debug_handler]
pub async fn read_one(
    State(app_state): State<SharedAppState>,
    Path(id): Path<Uuid>,
) -> Result<()Json<entities::notes::Note>, Error> {
    let note = entities::notes::load(id, &app_state.db_pool).await?;
    Ok(Json(note))
}

…
```

The auto-generated code is already what we need: it uses the data access functions defined in the `entities::notes` module of the [`db` crate](../architecture/the-db-crate) to load all notes or one particular note defined by its ID from the database. Since the `Note` entity is `Serializable`, we can simply JSON-ify it in both functions for the reponse.

The database connection (or specifically, a connection pool) is passed to the data access functions from the application state. When the project was generated, Gerust automatically defined the `AppState` struct in `web/src/state.rs` along with a function to initialize it on startup:

```rust
use my_app_config::Config;
use my_app_db::{connect_pool, DbPool};
use std::sync::Arc;

/// The application's state that is available in [`crate::controllers`] and [`crate::middlewares`].
pub struct AppState {
    /// The database pool that's used to get a connection to the application's database (see [`my_app_db::DbPool`]).
    pub db_pool: DbPool,
}

/// The application's state as it is shared across the application, e.g. in controllers and middlewares.
///
/// This is the [`AppState`] struct wrappend in an [`std::sync::Arc`].
pub type SharedAppState = Arc<AppState>;

/// Initializes the application state.
///
/// This function creates an [`AppState`] based on the current [`my_app_config::Config`].
pub async fn init_app_state(config: Config) -> AppState {
    let db_pool = connect_pool(config.database)
        .await
        .expect("Could not connect to database!");

    AppState { db_pool }
}
```

## Routing the Endpoints

In order to expose the controller's `read_all` and `read_one` endpoints, they need to be routed in `web/src/routes.rs`:

```rust
// diff-add
+use crate::controllers::notes;
use crate::state::AppState;
// diff-remove
-use axum::Router;
// diff-add
+use axum::{routing::get, Router};

use std::sync::Arc;

/// Initializes the application's routes.
///
/// This function maps paths (e.g. "/greet") and HTTP methods (e.g. "GET") to functions in [`crate::controllers`] as well as includes middlewares defined in [`crate::middlewares`] into the routing layer (see [`axum::Router`]).
pub fn init_routes(app_state: AppState) -> Router {
    let shared_app_state = Arc::new(app_state);
// diff-remove
-    Router::new().with_state(shared_app_state)
// diff-add
+    Router::new()
// diff-add
+        .route("/notes", get(notes::read_all))
// diff-add
+        .route("/notes/{id}", get(notes::read_one))
// diff-add
+        .with_state(shared_app_state)

}
```

That's all that needs to be done to be able to read notes that are stored in the database via the `web` application's interface. Of course there are no notes in the database yet so we receive an empty response for now:

```
 » curl -i http://127.0.0.1:3000/notes
HTTP/1.1 200 OK
content-type: application/json
content-length: 2
date: Wed, 22 Jan 2025 14:01:54 GMT

[]%
```

## Testing

When the `notes` controller was created, Gerust automatically created a test for it in `web/tests/api/notes_test.rs`. It comes with test cases for all of the scenarios we'll want to cover for a <abbr>CRUD</abbr> controller. Since we only care about the `read_all` and `read_one` endpoints for now, we can uncomment the auto-generated code for the respective test cases, replace the example `description` property with the `text` property that we're using, and remove the `#[ignore]`s:

```rust
use axum::{
    body::Body,
    http::{self, Method},
};
use fake::{Fake, Faker};
use googletest::prelude::*;
use hyper::StatusCode;
use my_app_db::{entities, transaction, Error};
use my_app_macros::db_test;
use my_app_web::test_helpers::{BodyExt, DbTestContext, RouterExt};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

#[ignore = "not yet implemented"]
#[db_test]
async fn test_create_invalid(context: &DbTestContext) {
    let payload = json!(entities::notes::NoteChangeset {
// diff-remove
-        name: String::from("")
// diff-add
+        text: String::from("")
    });

    let response = context
        .app
        .request("/notes")
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
    let changeset: entities::notes::NoteChangeset = Faker.fake();
    let payload = json!(changeset);

    let response = context
        .app
        .request("/notes")
        .method(Method::POST)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::CREATED));

    let notes = entities::notes::load_all(&context.db_pool).await.unwrap();
    assert_that!(notes, len(eq(1)));
// diff-remove
-   assert_that!(notes.first().unwrap().name, eq(&changeset.name));
// diff-add
+   assert_that!(notes.first().unwrap().text, eq(&changeset.text));
}

// diff-remove
-#[ignore = "not yet implemented"]
#[db_test]
 async fn test_read_all(context: &DbTestContext) {
     let changeset: entities::notes::NoteChangeset = Faker.fake();
     entities::notes::create(changeset.clone(), &context.db_pool)
         .await
         .unwrap();

       let response = context
           .app
           .request("/notes")
           .send()
           .await;

       assert_that!(response.status(), eq(StatusCode::OK));

       let notes: Vec<entities::notes::Note> = response.into_body().into_json::<Vec<entities::notes::Note>>().await;
       assert_that!(notes, len(eq(1)));
       assert_that!(
// diff-remove
-           notes.first().unwrap().name,
// diff-add
+           notes.first().unwrap().text,
           eq(&changeset.text)
       );
}

// diff-remove
-#[ignore = "not yet implemented"]
#[db_test]
 async fn test_read_one_nonexistent(context: &DbTestContext) {
     let response = context
         .app
         .request(&format!("/notes/{}", Uuid::new_v4()))
         .body(Body::from(payload.to_string()))
         .send()
         .await;

     assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

// diff-remove
-#[ignore = "not yet implemented"]
#[db_test]
async fn test_read_one_success(context: &DbTestContext) {
    let note_changeset: entities::notes::NoteChangeset = Faker.fake();
    let note = entities::notes::create(note_changeset.clone(), &context.db_pool)
        .await
        .unwrap();
    let note_id = note.id;

    let response = context
        .app
        .request(&format!("/notes/{}", note_id))
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::OK));

    let note: entities::notes::Note = response.into_body().into_json::<entities::notes::Note>().await;
    assert_that!(note.id, eq(note_id));
// diff-remove
-    assert_that!(note.name, eq(&note_changeset.name));
// diff-remove
+    assert_that!(note.text, eq(&note_changeset.text));
}

#[ignore = "not yet implemented"]
#[db_test]
async fn test_update_invalid(context: &DbTestContext) {
    let note_changeset: entities::notes::NoteChangeset = Faker.fake();
    let note = entities::notes::create(note_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let payload = json!(entities::notes::NoteChangeset {
// diff-remove
-        name: String::from("")
// diff-add
+        text: String::from("")
    });

    let response = context
        .app
        .request(&format!("/notes/{}", note.id))
        .method(Method::PUT)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::UNPROCESSABLE_ENTITY));

    let note_after = entities::notes::load(note.id, &context.db_pool)
        .await
        .unwrap();
// diff-remove
-   assert_that!(note_after.text, eq(&note.text));
// diff-add
+   assert_that!(note_after.text, eq(&note.text));
}

#[ignore = "not yet implemented"]
#[db_test]
async fn test_update_nonexistent(context: &DbTestContext) {
    let note_changeset: entities::notes::NoteChangeset = Faker.fake();
    let payload = json!(note_changeset);

    let response = context
        .app
        .request(&format!("/notes/{}", Uuid::new_v4()))
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
    let note_changeset: entities::notes::NoteChangeset = Faker.fake();
    let note = entities::notes::create(note_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let note_changeset: entities::notes::NoteChangeset = Faker.fake();
    let payload = json!(note_changeset);

    let response = context
        .app
        .request(&format!("/notes/{}", note.id))
        .method(Method::PUT)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::OK));

    let note: entities::notes::Note = response
        .into_body()
        .into_json::<entities::notes::Note>()
        .await;
// diff-remove
-   assert_that!(note.name, eq(&note_changeset.name.clone()));
// diff-add
+   assert_that!(note.text, eq(&note_changeset.text.clone()));

    let note = entities::notes::load(note.id, &context.db_pool)
        .await
        .unwrap();
// diff-remove
-   assert_that!(note.name, eq(&note_changeset.name));
// diff-remove
+   assert_that!(note.text, eq(&note_changeset.text));
}

#[ignore = "not yet implemented"]
#[db_test]
async fn test_delete_nonexistent(context: &DbTestContext) {
    let response = context
        .app
        .request(&format!("/notes/{}", Uuid::new_v4()))
        .method(Method::DELETE)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[ignore = "not yet implemented"]
#[db_test]
async fn test_delete_success(context: &DbTestContext) {
    let note_changeset: entities::notes::NoteChangeset = Faker.fake();
    let note = entities::notes::create(note_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let response = context
        .app
        .request(&format!("/notes/{}", Uuid::new_v4()))
        .method(Method::DELETE)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NO_CONTENT));

    let result = entities::notes::load(note.id, &context.db_pool).await;
    assert_that!(result, err(anything()));
}
```

All of these tests use Gerust's `#[db_test]` macro (defined in the [`macros` crate](../architecture/the-macros-crate/)) instead of Rust's (or really tokio's) `#[test]` macro. Gerust creates a fresh database for each test case that's specific for the test (and created using the main test database as a template). By using the `#[db_test]` macro, the test function will be called with a `DbTestContext` argument which provides access to the test's dedicated database as well as the application instance being tested (which is configured to use the test's database as well). The test-specific database is automatically deleted once the test completes so that no unused databases are left behind.

Each test case using its own dedicated database means each test can set up the database state it needs without interfering with any other tests – see the [`web` crate's documentation](../architecture/the-web-crate#testing) for more details.

The test functions for the `notes` controller populate the test database with fake data (using the [fake data generators configured for the changeset](./the-entity#generating-the-entity)), e.g.:

```rust
let changeset: entities::notes::NoteChangeset = Faker.fake();
entities::notes::create(changeset.clone(), &context.db_pool)
    .await
    .unwrap();
```

The tests call the respective endpoint of the application using the helper functions defined on `context.app`, e.g.:

```rust
let response = context
    .app
    .request(&format!("/notes/{}", note_id))
    .send()
    .await;
```

…and assert on the response based on a well-defined state of the database that's isolated from all other test cases.

In order to run the tests, we need to migrate the test database first (when the database was migrated before, the **development** database was migrated but the tests will use the **test** database configured in `.env.test`):

```
» cargo db migrate -e test
```

Once the database is migrated, we can run the tests:

```
» cargo test
   Compiling my-app-web v0.0.1 (/Users/marcoow/Code/gerust/my-app/web)

…

running 10 tests
test notes_test::test_create_invalid ... ignored, not yet implemented
test notes_test::test_create_success ... ignored, not yet implemented
test notes_test::test_delete_nonexistent ... ignored, not yet implemented
test notes_test::test_delete_success ... ignored, not yet implemented
test notes_test::test_update_invalid ... ignored, not yet implemented
test notes_test::test_update_nonexistent ... ignored, not yet implemented
test notes_test::test_update_success ... ignored, not yet implemented
test notes_test::test_read_one_nonexistent ... ok
test notes_test::test_read_all ... ok
test notes_test::test_read_one_success ... ok

test result: ok. 3 passed; 0 failed; 7 ignored; 0 measured; 0 filtered out; finished in 0.22s
```

All tests for reading notes via the system's CRUD interface complete successfully. The tests for the writing endpoints are ignored for now – we'll get to them in the next step.

---

Now that the endpoints for reading notes are functional, let's move on to endpoints for writing (creating, updating and deleting) notes next.
