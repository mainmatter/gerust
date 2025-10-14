---
sidebar_position: 4
---

# Adding a Middleware

Finally, to introduce middlewares and make the example more realistic perhaps, we introduce a middleware to require a user to be authenticated for creating, updating, and deleting notes.

:::warning

The middleware introduced in this chapter is not actually a proper way to implement authentication and just a simplified example!

:::

## Generating the Entity

First, we need to introduce the concept of a user as such. We can do that by generating a `User` entity the same way we created the `Note` entity [in the first step](./the-entity):

```sh
» cargo generate entity user name:string
```

In this case, we don't keep most of the generated standard blueprint code since we want to treat users very differently than notes: Users cannot be created as part of the app's normal execution flow so we don't need a `UserChangeset` or the `create`, `update`, or `delete` functions. Also, users can only be loaded by their secret token (which is going to be included in incoming requests to identify the users) but not by their ID and we also don't need to be able to load all users at once. Plus, the token should not be leaked obviously and does never actually need to be read during the program's execution. Thus, the `User` entity and its related functionality can be shortened to this:

```rust
use serde::Serialize;
use sqlx::Postgres;
use uuid::Uuid;

#[derive(Serialize, Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub name: String,
}

pub async fn load_with_token(
    token: &str,
    executor: impl sqlx::Executor<'_, Database = Postgres>,
) -> Result<Option<User>, anyhow::Error> {
    Ok(
        sqlx::query_as!(User, "SELECT id, name FROM users WHERE token = $1", token)
            .fetch_optional(executor)
            .await?,
    )
}
```

:::info

Note that the `User` entity needs to implement the `Clone` trait so that we can use it properly in the middleware later.

:::

The `User` entity has an `id` and `name` only – we did not include the token in the field list when generating ent entity because we don't want it to be exposed as a property at all so there's no risk of leaking it by e.g. responding with a `User` entity that's serialized to JSON. The only way to load a user from the database is via the user's secret token and the `load_with_token` function.

Let's generate the corresponding migration next:

```sh
» cargo generate migration create-users
```

which generates the migration file in `/db/migrations/1743085345__create-users.sql` (timestamp prefix will vary). Use the following SQL to create the `users` table:

```sql
CREATE TABLE users (
    id uuid PRIMARY KEY default gen_random_uuid(),
    name varchar(255) NOT NULL,
    token varchar(100) NOT NULL
);
```

The next step is again to migrate the database:

```sh
» cargo db migrate
```

## The Middleware

Now that the entity is done, let's create the middleware:

```sh
» cargo generate middleware auth
```

That creates the middleware in `web/src/middlewares/auth.rs` with the basic scaffolding:

```rust
use crate::state::SharedAppState;
use axum::body::Body;
use axum::{
    extract::State,
    http::{self, Request, StatusCode},
    middleware::Next,
    response::Response,
};

#[tracing::instrument(skip_all, fields(rejection_reason = tracing::field::Empty))]
pub async fn auth(
    State(app_state): State<SharedAppState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    todo!("Implement this (return `next.run(req).await` to continue processing the request or Err(StatusCode) to error out).")
}
```

The idea of the authentication mechanism we're building is that the user will send their secret token in the `Authorization` header. If a user is found for that token, the request is authenticated with that user, otherwise it's not. Let's implement the logic for that:

```rust
…
// diff-add
+use my_app_db::entities::users;
// diff-add
+
#[tracing::instrument(skip_all, fields(rejection_reason = tracing::field::Empty))]
pub async fn auth(
    State(app_state): State<SharedAppState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
// diff-remove
-   todo!("Implement this (return `next.run(req).await` to continue processing the request or Err(StatusCode) to error out).")
// diff-add
+   let auth_header = req
// diff-add
+       .headers()
// diff-add
+       .get(http::header::AUTHORIZATION)
// diff-add
+       .and_then(|header| header.to_str().ok());
// diff-add
+
// diff-add
+   let auth_header = if let Some(auth_header) = auth_header {
// diff-add
+       auth_header
// diff-add
+   } else {
// diff-add
+       return Err(StatusCode::UNAUTHORIZED);
// diff-add
+   };
// diff-add
+
// diff-add
+   match users::load_with_token(auth_header, &app_state.db_pool).await {
// diff-add
+       Ok(Some(current_user)) => {
// diff-add
+           req.extensions_mut().insert(current_user);
// diff-add
+           Ok(next.run(req).await)
// diff-add
+       }
// diff-add
+       Ok(None) => {
// diff-add
// diff-add
+           return Err(StatusCode::UNAUTHORIZED);
// diff-add
+       }
// diff-add
+       Err(_) => {
// diff-add
// diff-add
+           Err(StatusCode::INTERNAL_SERVER_ERROR)
// diff-add
+       }
// diff-add
+   }
}
```

If the `Authorization` header is present and a user is found for the passed token, the middleware calls `next.run(req).await` to continue processing of the request, otherwise (if no header is present or no user is found), it returns `Err(StatusCode::UNAUTHORIZED)`.

That middleware can now be added to the router to require authentication for the endpoints for creating, updating, and deleting notes:

```rust
use crate::controllers::notes;
// diff-add
+use crate::middlewares::auth::auth;
use crate::state::AppState;
use axum::{
// diff-add
+    middleware,
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

pub fn init_routes(app_state: AppState) -> Router {
    let shared_app_state = Arc::new(app_state);
    Router::new()
        .route("/notes", post(notes::create))
// diff-remove
-        .route("/notes", get(notes::read_all))
// diff-remove
-        .route("/notes/{id}", get(notes::read_one))
        .route("/notes/{id}", put(notes::update))
        .route("/notes/{id}", delete(notes::delete))
// diff-add
+       // the middleware will be applied to all requests to routes preceding this line
// diff-add
+        .route_layer(middleware::from_fn_with_state(shared_app_state.clone(), auth))
// diff-add
+        .route("/notes", get(notes::read_all))
// diff-add
+        .route("/notes/{id}", get(notes::read_one))
        .with_state(shared_app_state)
}
```

To verify it works, call the note creation endpoint without passing the `Authorization` header:

```
» curl -i -X POST 127.0.0.1:3000/notes -H 'Authorization: 2c1b1ca9b5cf201368cc68f81ab75a5155091edf5aac5a2ada5633d617363c9dd363a0f2b10633d3cca5958fb2053e16c922' -H 'Content-Type: application/json' -d '{"text": "do something"}'
HTTP/1.1 401 Unauthorized
content-length: 0
date: Thu, 27 Mar 2025 16:04:31 GMT
```

…which responds with a 401 status code as expected. To verify it works correctly when authentication credentials are provided, first create a new user:

```sh
» psql -Atx "postgresql://my_app:my_app@localhost:5432/my_app" -c "INSERT INTO users (name, token) VALUES ('admin', '2c1b1ca9b5cf201368cc68f81ab75a5155091edf5aac5a2ada5633d617363c9dd363a0f2b10633d3cca5958fb2053e16c922')"
```

…and invoke the same endpoint again, this time passing the token in the `Authorization` header:

```
» curl -i -X POST 127.0.0.1:3000/notes -H 'Authorization: 2c1b1ca9b5cf201368cc68f81ab75a5155091edf5aac5a2ada5633d617363c9dd363a0f2b10633d3cca5958fb2053e16c922' -H 'Content-Type: application/json' -d '{"text": "do something"}'
HTTP/1.1 201 Created
content-type: application/json
content-length: 67
date: Thu, 27 Mar 2025 16:12:37 GMT

{"id":"ddb15cf7-587b-4221-aca8-7f889673d1fe","text":"do something"}
```

## Testing

Now that the endpoint for creating, updating, and deleting notes require a user to be authenticated, the respective tests fail since the tests have not been updated yet to provide an authenticated user:

```
» cargo test
   Compiling my-app-web v0.0.1 (/Users/marcoow/Code/gerust/my-app/web)

…

failures:
notes_test::test_create_invalid
notes_test::test_create_success
notes_test::test_delete_nonexistent
notes_test::test_delete_success
notes_test::test_update_invalid
notes_test::test_update_nonexistent
notes_test::test_update_success

test result: FAILED. 3 passed; 7 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s

error: test failed, to rerun pass `--test api`
```

Let's adapt the tests so they create a user and pass the secret token when invoking the above endpoints. However, when we [created the `User` entity above](#generating-the-entity), we only created the `load_with_token` function and no `UserChangeset` since creation of users is not actually supported in the application. We don't necessarily want to create functionality for creating, updating, or deleting users that would become part of the `db` crate's public interface only to satisfy the `web` crate's tests.

That's exactly what _"Entity test helpers"_ are for in Gerust. They encapsulate code that's required for tests but should not be part of the `db` crate's public interface or be included in a release build of the application. Like the fake data configuration (see the [creation of the `Note` entity](the-entity#generating-the-entity)), entity test helpers are hidden behind the `test-helpers` feature flag which is only enabled when tests are run.

Let's create an entity test helper for the `User` entity:

```sh
» cargo generate entity-test-helper user
```

That creates a test helper for the `User` entity in `db/src/test_helpers/users.rs`. That can be adapted to match the `User` entity created before:

```rust
use crate::entities::users::User;
use fake::{faker::name::en::*, Dummy};
use sqlx::postgres::PgPool;

#[derive(Debug, Clone, Dummy)]
pub struct UserChangeset {
    #[dummy(faker = "Name()")]
    pub name: String,
    // The user's auth token, fake data will be a 100 characters long number
    #[dummy(faker = "100..101")]
    pub token: String,
}

pub async fn create(user: UserChangeset, db: &PgPool) -> Result<User, anyhow::Error> {
    let record = sqlx::query!(
        "INSERT INTO users (name, token) VALUES ($1, $2) RETURNING id",
        user.name,
        user.token
    )
    .fetch_one(db)
    .await?;

    Ok(User {
        id: record.id,
        name: user.name
    })
}
```

The entity test helper for the `User` entity defines a `UserChangeset` as well as a `create` function. Those work exactly the same as the `NoteChangeset` and the `create` function for the `Note` entity [we created in the first step](./the-entity#generating-the-entity) with the only difference that this functionality is only available when running tests.

Now, we can adapt the tests for note creation, update, and deletion to create a user and pass the user's token in the `Authorization` header:

```rust
…
// diff-add
-use my_app_db::{entities, transaction, Error};
// diff-add
+use my_app_db::{entities, test_helpers::users, transaction, Error};
use my_app_macros::db_test;
use my_app_web::test_helpers::{BodyExt, DbTestContext, RouterExt};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

#[db_test]
async fn test_create_invalid(context: &DbTestContext) {
// diff-add
+   let user_changeset: users::UserChangeset = Faker.fake();
// diff-add
+   users::create(user_changeset.clone(), &context.db_pool)
// diff-add
+       .await
// diff-add
+       .unwrap();
    let payload = json!(entities::notes::NoteChangeset {
        text: String::from("")
    });

    let response = context
        .app
        .request("/notes")
        .method(Method::POST)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
// diff-add
+       .header(http::header::AUTHORIZATION, &user_changeset.token)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::UNPROCESSABLE_ENTITY));
}

#[db_test]
async fn test_create_success(context: &DbTestContext) {
// diff-add
+   let user_changeset: users::UserChangeset = Faker.fake();
// diff-add
+   users::create(user_changeset.clone(), &context.db_pool)
// diff-add
+       .await
// diff-add
+       .unwrap();
    let changeset: entities::notes::NoteChangeset = Faker.fake();
    let payload = json!(changeset);

    let response = context
        .app
        .request("/notes")
        .method(Method::POST)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
// diff-add
+       .header(http::header::AUTHORIZATION, &user_changeset.token)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::CREATED));

    let notes = entities::notes::load_all(&context.db_pool).await.unwrap();
    assert_that!(notes, len(eq(1)));
    assert_that!(notes.first().unwrap().text, eq(&changeset.text));
}

…

#[db_test]
async fn test_update_invalid(context: &DbTestContext) {
// diff-add
+   let user_changeset: users::UserChangeset = Faker.fake();
// diff-add
+   users::create(user_changeset.clone(), &context.db_pool)
// diff-add
+       .await
// diff-add
+       .unwrap();
    let note_changeset: entities::notes::NoteChangeset = Faker.fake();
    let note = entities::notes::create(note_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let payload = json!(entities::notes::NoteChangeset {
        text: String::from("")
    });

    let response = context
        .app
        .request(&format!("/notes/{}", note.id))
        .method(Method::PUT)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
// diff-add
+       .header(http::header::AUTHORIZATION, &user_changeset.token)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::UNPROCESSABLE_ENTITY));

    let note_after = entities::notes::load(note.id, &context.db_pool)
        .await
        .unwrap();
    assert_that!(note_after.text, eq(&note.text));
}

#[db_test]
async fn test_update_nonexistent(context: &DbTestContext) {
// diff-add
+   let user_changeset: users::UserChangeset = Faker.fake();
// diff-add
+   users::create(user_changeset.clone(), &context.db_pool)
// diff-add
+       .await
// diff-add
+       .unwrap();
    let note_changeset: entities::notes::NoteChangeset = Faker.fake();
    let payload = json!(note_changeset);

    let response = context
        .app
        .request(&format!("/notes/{}", Uuid::new_v4()))
        .method(Method::PUT)
        .body(Body::from(payload.to_string()))
        .header(http::header::CONTENT_TYPE, "application/json")
// diff-add
+       .header(http::header::AUTHORIZATION, &user_changeset.token)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[db_test]
async fn test_update_success(context: &DbTestContext) {
// diff-add
+   let user_changeset: users::UserChangeset = Faker.fake();
// diff-add
+   users::create(user_changeset.clone(), &context.db_pool)
// diff-add
+       .await
// diff-add
+       .unwrap();
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
// diff-add
+       .header(http::header::AUTHORIZATION, &user_changeset.token)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::OK));

    let note: entities::notes::Note = response
        .into_body()
        .into_json::<entities::notes::Note>()
        .await;
    assert_that!(note.text, eq(&note_changeset.text.clone()));

    let note = entities::notes::load(note.id, &context.db_pool)
        .await
        .unwrap();
    assert_that!(note.text, eq(&note_changeset.text));
}

#[db_test]
async fn test_delete_nonexistent(context: &DbTestContext) {
// diff-add
+   let user_changeset: users::UserChangeset = Faker.fake();
// diff-add
+   users::create(user_changeset.clone(), &context.db_pool)
// diff-add
+       .await
// diff-add
+       .unwrap();
    let response = context
        .app
        .request(&format!("/notes/{}", Uuid::new_v4()))
// diff-add
+       .header(http::header::AUTHORIZATION, &user_changeset.token)
        .method(Method::DELETE)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NOT_FOUND));
}

#[db_test]
async fn test_delete_success(context: &DbTestContext) {
// diff-add
+   let user_changeset: users::UserChangeset = Faker.fake();
// diff-add
+   users::create(user_changeset.clone(), &context.db_pool)
// diff-add
+       .await
// diff-add
+       .unwrap();
    let note_changeset: entities::notes::NoteChangeset = Faker.fake();
    let note = entities::notes::create(note_changeset.clone(), &context.db_pool)
        .await
        .unwrap();

    let response = context
        .app
        .request(&format!("/notes/{}", note.id))
// diff-add
+       .header(http::header::AUTHORIZATION, &user_changeset.token)
        .method(Method::DELETE)
        .send()
        .await;

    assert_that!(response.status(), eq(StatusCode::NO_CONTENT));

    let result = entities::notes::load(note.id, &context.db_pool).await;
    assert_that!(result, err(anything()));
}
```

…which, after migrating the test database:

```sh
» cargo db migrate -e test
```

…fixes the tests

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

🎉
