---
sidebar_position: 2
---

# Adding a new Endpoint

Instead of having our app always greet the entire world, let's make it so that visitors can specify their name to be personally greeted:

```
curl -H 'Content-Type: application/json' \
     -d '{ "name": "Ferris"} ' \
     -X POST \
     http://127.0.0.1:3000/greet_me
```

## Implementing the controller function

For now the above curl command result in an error since our app only exposes the `/greet` endpoint and nothing handles `/greet_me`. Let's add that endpoint to `web/src/controllers/greeting.rs`:

```rust
…
// diff-add
+#[axum::debug_handler]
// diff-add
+pub async fn hello_person(State(app_state): State<SharedAppState>) -> Json<Greeting> {
// diff-add
+    app_state.count_visit();
// diff-add
+    Json(Greeting {
// diff-add
+        hello: String::from("<user-name>"),
// diff-add
+        visit: app_state.get_visit_count(),
// diff-add
+    })
// diff-add
+}
…
```

We keep the counting of visits from the previous step. Now, instead of `<user-name>`, we want to use the user's name that they send in the `POST` data of course. In order to process the data, we need a struct that describes its shape:

```rust
// web/src/controllers/greeting.rs
…
// diff-add
+#[derive(Deserialize, Serialize)]
// diff-add
+ pub struct PersonalData {
// diff-add
+     pub name: String,
// diff-add
+ }
…
```

That struct can be consumed in the new `hellp_person` endpoint:

```rust
// web/src/controllers/greeting.rs
…
#[axum::debug_handler]
// diff-add
+pub async fn hello_person(
// diff-add
+    State(app_state): State<SharedAppState>,
// diff-add
+    Json(person): Json<PersonalData>,
// diff-add
+) -> Json<Greeting> {
// diff-remove
-pub async fn hello_person(State(app_state): State<SharedAppState>) -> Json<Greeting> {
    app_state.count_visit();
    Json(Greeting {
// diff-add
+       hello: person.name,
// diff-remove
-       hello: String::from("<user-name>"),
        visit: app_state.get_visit_count(),
    })
}
```

## Routing

The last step for making the endpoint work at `/greet_me` is to route it. All of the application's routes are defined in `web/src/routes.rs` so let's add it there:

```rust
use crate::controllers::greeting;
use crate::state::AppState;
// diff-add
+use axum::{routing::{get, post}, Router};
// diff-remove
-use axum::{routing::get, Router};
use std::sync::Arc;

/// Initializes the application's routes.
///
/// This function maps paths (e.g. "/greet") and HTTP methods (e.g. "GET") to functions in [`crate::controllers`] as well as includes middlewares defined in [`crate::middlewares`] into the routing layer (see [`axum::Router`]).
pub fn init_routes(app_state: AppState) -> Router {
    let shared_app_state = Arc::new(app_state);

    Router::new()
        .route("/greet", get(greeting::hello))
// diff-add
+        .route("/greet_me", post(greeting::hello_person))
        .with_state(shared_app_state)
}
```

We can now invoke the new endpoint and enjoy our personal greeting 🦀:

```
» curl -H 'Content-Type: application/json' -d '{ "name": "Ferris"} ' -X POST http://127.0.0.1:3000/greet_me
{"hello":"Ferris","visit":1}%
```

## Testing

Now that the endpoint is working as intended, let's add a test to make sure we catch any potential regressions that break it in the future. Since a test for the `greeting` controller exists already in `web/tests/api/greeting_test.rs`, we can add a new test case there:

```rust
// diff-add
+use axum::{
// diff-add
+    body::Body,
// diff-add
+    http::{self, Method},
// diff-add
+};
use googletest::prelude::*;
use my_app_macros::test;
// diff-add
+use my_app_web::controllers::greeting::{Greeting, PersonalData};
// diff-remove
-use my_app_web::controllers::greeting::Greeting;
use my_app_web::test_helpers::{BodyExt, RouterExt, TestContext};
// diff-add
+use serde_json::json;
…

// diff-add
+#[test]
// diff-add
+async fn test_personal_greeting(context: &TestContext) {
// diff-add
+    let payload = json!(PersonalData {
// diff-add
+        name: String::from("Ferris"),
// diff-add
+    });
// diff-add
+    let response = context
// diff-add
+        .app
// diff-add
+        .request("/greet_me")
// diff-add
+        .method(Method::POST)
// diff-add
+        .body(Body::from(payload.to_string()))
// diff-add
+        .header(http::header::CONTENT_TYPE, "application/json")
// diff-add
+        .send()
// diff-add
+        .await;
// diff-add
+
// diff-add
+    let greeting: Greeting = response.into_body().into_json().await;
// diff-add
+    assert_that!(greeting.hello, eq(&String::from("Ferris")));
// diff-add
+}
```

The request can be built up step-by-step using the testing convenience functions that are available on the application the test receives via the test context.

---

Finally, in the last step of the tutorial, let's add a middleware.
