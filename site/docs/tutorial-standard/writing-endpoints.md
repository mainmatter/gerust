---
sidebar_position: 3
---

# Endpoints for Creation, Update, and Deletion

For a full CRUD interface, we are missing the endpoints to create, update, and delete notes.

## Routing the writing endpoints

The `create`, `update`, and `delete` endpoints were created as part of the standard blueprint when we generated the CRUD controller in the previous step. To make them usable, we can go ahead, remove the `todo!`s and uncomment the generated standard code:

```rust
#[axum::debug_handler]
pub async fn create(
    State(app_state): State<SharedAppState>,
// diff-remove
-    Json(note): Json<() /* e.g.entities::notes::NoteChangeset */>,
// diff-remove
-) -> Result<() /* e.g. (StatusCode, Json<entities::notes::Note>) */, Error> {
// diff-remove
-    todo!("create resource via my_app_db's APIs, trace, and respond!")
// diff-remove
-
// diff-remove
-    /* Example:
// diff-add
+   Json(note): Json<entities::notes::NoteChangeset>,
// diff-add
+) -> Result<(StatusCode, Json<entities::notes::Note>), Error> {
    let note = entities::notes::create(note, &app_state.db_pool).await?;
    Ok((StatusCode::CREATED, Json(note)))
// diff-remove
-   */
}

…

#[axum::debug_handler]
pub async fn update(
    State(app_state): State<SharedAppState>,
    Path(id): Path<Uuid>,
// diff-remove
-    Json(note): Json<() /* e.g. entities::notes::NoteChangeset */>,
// diff-remove
-) -> Result<() /* e.g. Json<entities::notes::Note> */, Error> {
// diff-remove
-    todo!("update resource via my_app_db's APIs, trace, and respond!")
// diff-remove
-
// diff-remove
-    /* Example:
// diff-add
+   Json(note): Json<entities::notes::NoteChangeset>,
// diff-add
+) -> Result<Json<entities::notes::Note>, Error> {
    let note = entities::notes::update(id, note, &app_state.db_pool).await?;
    Ok(Json(note))
// diff-remove
-   */
}

#[axum::debug_handler]
pub async fn delete(
    State(app_state): State<SharedAppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, Error> {
// diff-remove
-    todo!("delete resource via my_app_db's APIs, trace, and respond!")
// diff-remove
-
// diff-remove
-    /* Example:
    entities::notes::delete(id, &app_state.db_pool).await?;
    Ok(StatusCode::NO_CONTENT)
// diff-remove
-   */
}

```

No additional changes are necessary in the controller – the generated standard code is what we need. In the case of the `create` and `update` functions, we deserialize a `NoteChangeset` from the request body and pass it to the `entities::notes::create` and `entities::notes::update` functions respectively. For the `delete` function, the note id taken from the request path is passed.

The next step is to route those functions to the respective CRUD endpoints in `src/routes.rs`:

```rust
use crate::controllers::notes;
use crate::state::AppState;
// diff-remove
-use axum::{routing::get, Router};
// diff-add
+use axum::{
// diff-add
+    routing::{delete, get, post, put},
// diff-add
+    Router,
// diff-add
+};

…

    Router::new()
// diff-add
+        .route("/notes", post(notes::create))
        .route("/notes", get(notes::read_all))
        .route("/notes/{id}", get(notes::read_one))
// diff-add
+        .route("/notes/{id}", put(notes::update))
// diff-add
+        .route("/notes/{id}", delete(notes::delete))

```

We can confirm these endpoints work correctly from the command line:

```
» curl -X POST 127.0.0.1:3000/notes -H 'Content-Type: application/json' -d '{"text": "do something"}'
{"id":"8dd74edb-6187-4588-b976-5529590ea667","text":"do something"}%

» curl -i http://127.0.0.1:3000/notes
HTTP/1.1 200 OK
content-type: application/json
content-length: 69
date: Wed, 26 Mar 2025 16:29:04 GMT

[{"id":"8dd74edb-6187-4588-b976-5529590ea667","text":"do something"}]%

» curl -X PUT 127.0.0.1:3000/notes/8dd74edb-6187-4588-b976-5529590ea667 -H 'Content-Type: application/json' -d '{"text": "do something else"}'
{"id":"8dd74edb-6187-4588-b976-5529590ea667","text":"do something else"}%

» curl -X DELETE 127.0.0.1:3000/notes/8dd74edb-6187-4588-b976-5529590ea667

» curl -i http://127.0.0.1:3000/notes
HTTP/1.1 200 OK
content-type: application/json
content-length: 2
date: Wed, 26 Mar 2025 16:33:08 GMT

[]%
```

## Testing

Being able to confirm correctness on the command line is nice but let's enable the tests for the `create`, `update` and `delete` endpoints again for proper test coverage. We can simply remove the remaining `#[ignore]`s on those tests in `web/tests/notes/test.rs` and the tests will pass already:

```
» cargo test
   Compiling my-app-web v0.0.1 (/Users/marcoow/Code/gerust/my-app/web)

…

running 10 tests
test notes_test::test_create_invalid ... ok
test notes_test::test_create_success ... ok
test notes_test::test_delete_nonexistent ... ok
test notes_test::test_delete_success ... ok
test notes_test::test_update_invalid ... ok
test notes_test::test_update_nonexistent ... ok
test notes_test::test_update_success ... ok
test notes_test::test_read_one_nonexistent ... ok
test notes_test::test_read_all ... ok
test notes_test::test_read_one_success ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
```

---

In the last step, we will add a middleware that requires authentication for any of these writing endpoints.
