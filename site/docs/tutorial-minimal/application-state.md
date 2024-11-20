---
sidebar_position: 1
---

# Application State

We'll add a simple visitor counter so that we can keep track of how many visitors access the application's endpoints.

## Extending the Application State

The application state is defined in `web/src/state.rs`. That's where we'll add our new `counter` property to the `AppState` struct:

```rust
use my_app_config::Config;
// diff-add
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

 /// The application's state that is available in [`crate::controllers`] and [`crate::middlewares`].
// diff-remove
-pub struct AppState {}
// diff-add
+pub struct AppState {
// diff-add
+    counter: AtomicUsize,
// diff-add
+}
// diff-add
+
// diff-add
+impl AppState {
// diff-add
+    pub fn get_visit_count(&self) -> usize {
// diff-add
+        self.counter.load(Ordering::Relaxed)
// diff-add
+    }
// diff-add
+
// diff-add
+    pub fn count_visit(&self) {
// diff-add
+        self.counter.fetch_add(1, Ordering::Relaxed);
// diff-add
+    }
// diff-add
+}

/// The application's state as it is shared across the application, e.g. in controllers and middlewares.
///
/// This function creates an [`AppState`] based on the current [`my_app_config::Config`].
pub async fn init_app_state(_config: Config) -> AppState {
// diff-remove
-    AppState {}
// diff-add
+    AppState {
// diff-add
+        counter: AtomicUsize::new(0),
// diff-add
+    }
}

```

We can then modify the `web/src/controllers/greeting.rs` controller to count each visit and share with the visitors how many visits to the endpoint have been made:

```rust
// diff-remove
-use axum::response::Json;
// diff-add
+use crate::state::SharedAppState;
// diff-add
+use axum::{extract::State, response::Json};
use serde::{Deserialize, Serialize};

/// A greeting to respond with to the requesting client
pub struct Greeting {
    /// Who do we say hello to?
    pub hello: String,
// diff-add
+    /// Let them know this is the nth visit
// diff-add
+    pub visit: usize,
}

/// Responds with a [`Greeting`], encoded as JSON.
#[axum::debug_handler]
// diff-remove
-pub async fn hello() -> Json<Greeting> {
// diff-add
+pub async fn hello(State(app_state): State<SharedAppState>) -> Json<Greeting> {
// diff-add
+    app_state.count_visit();
   Json(Greeting {
        hello: String::from("world"),
// diff-add
+        visit: app_state.get_visit_count(),
    })
}
```

## Testing

Let's now add a test for that as well. There is a test for the greeting controller already that Rust generated out-of-the-box when generating the project. However, when we run that, we see it no longer works:

```
» cargo test
   Compiling my-app-web v0.0.1 (/Users/mainmatter/Code/gerust/my-app/web)
error: cannot construct `AppState` with struct literal syntax due to private fields
   --> web/src/test_helpers/mod.rs:193:27
    |
193 |     let app = init_routes(AppState {});
    |                           ^^^^^^^^
    |
    = note: private field `counter` that was not provided

error: could not compile `my-app-web` (lib) due to 1 previous error
```

The problem is we added a new private field to our `AppState` that we're no specifying when creating the `AppState` for the app under test. Also, we cannot specify the value of the field when creating the `AppState` for the test anyway, since it's private. The easiest solution is to derive `std::default::Default` for the `AppState` since `AtomicUsize` implements that as well and will simply default to 0:

```rust
// web/src/state.rs
use my_app_config::Config;
// diff-add
+use std::default::Default;
…

// diff-add
+#[derive(Default)]
pub struct AppState {
    counter: AtomicUsize,
}
```

```rust
// web/src/test-helpers/mod.rs
use crate::routes::init_routes;
use crate::state::AppState;
use axum::{
    body::{Body, Bytes},
    http::{Method, Request},
    response::Response,
    Router,
};
// diff-add
+use std::default::Default;
…

pub async fn setup() -> TestContext {
    let init_config: OnceCell<Config> = OnceCell::new();
    let _config = init_config.get_or_init(|| load_config(&Environment::Test).unwrap());

// diff-add
+    let app_state: AppState = Default::default();
// diff-add
+    let app = init_routes(app_state);
// diff-remove
-    let app = init_routes(AppState {});

    TestContext { app }
}
```

That fixes the tests:

```
» cargo test
   Compiling my-app-web v0.0.1 (/Users/mainmatter/Code/gerust/my-app/web)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.75s
     Running unittests src/lib.rs (target/debug/deps/my_app_web-425f75e35e5cfe7b)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/main.rs (target/debug/deps/my_app_web-73f82937d51059a9)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/api/main.rs (target/debug/deps/api-2ba7a788d4d16867)

running 1 test
test greeting_test::test_hello ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

We can now add a test for the `visit` value we introduced in the response of the `/greet` endpoint to the existing test in `web/tests/api/greeting_test.rs`:

```rust
use googletest::prelude::*;
use my_app_macros::test;
use my_app_web::controllers::greeting::Greeting;
use my_app_web::test_helpers::{BodyExt, RouterExt, TestContext};

#[test]
async fn test_hello(context: &TestContext) {
    let response = context.app.request("/greet").send().await;

    let greeting: Greeting = response.into_body().into_json().await;
    assert_that!(greeting.hello, eq(&String::from("world")));
}
// diff-add
+
// diff-add
+#[test]
// diff-add
+async fn test_visit_count(context: &TestContext) {
// diff-add
+    let response = context.app.request("/greet").send().await;
// diff-add
+
// diff-add
+    let greeting: Greeting = response.into_body().into_json().await;
// diff-add
+    assert_that!(greeting.visit, eq(1));
// diff-add
+
// diff-add
+    let response = context.app.request("/greet").send().await;
// diff-add
+
// diff-add
+    let greeting: Greeting = response.into_body().into_json().await;
// diff-add
+    assert_that!(greeting.visit, eq(2));
// diff-add
+}
```

Since every test receives its own instance of the application under test which has its own application state which initializes the visit counter with `0`, we can simply request the `/greet` endpoint twice and assert the first visit is indeed reported as the first visit, and the second visit as the second. This test can run in parallel with other tests without the visit count getting messed up because of that isolation.

---

Let's now add a new endpoint to the application.
