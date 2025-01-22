---
sidebar_position: 3
---

# Working With Middleware

Despite the fact that our application doesn't really do anything and we could possible serve thousands of users running this on a Raspberry Pi, let's assume we really need to rate-limit how often people can invoke the greeting endpoints.

## Adding the middleware

Let's start with creating a simple rate limiting middleware. We can generate the file with Gerust's [CLI](../architecture/the-cli-crate):

```
» cargo generate middleware rate-limiter
```

That will create the basic structure for the new middleware in `web/src/middlewares/rate_limiter.rs`:

```rust
use crate::state::SharedAppState;
use axum::body::Body;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};

#[tracing::instrument(skip_all, fields(rejection_reason = tracing::field::Empty))]
pub async fn rate_limiter(
    State(app_state): State<SharedAppState>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    todo!("Implement this (return `next.run(req).await` to continue processing the request or Err(StatusCode) to error out).")
}
```

A naive rate limiter middleware could look something like this (this is based on https://www.shuttle.dev/blog/2024/02/22/api-rate-limiting-rust):

:::warning

This is not **a real rate-limiter middleware** and for the purpose of this tutorial only. Do not use this in production!

:::

We need the [`chrono` crate](https://crates.io/crates/chrono) so we add that to `web/Cargo.toml`:

```toml
…
[dependencies]
anyhow = "1.0"
axum = { version = "0.7", features = ["macros"] }
chrono = "0.4.38"
…
```

In the next step, we add a [`HashMap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html) to the `AppState` defined in `web/src/state.rs` so we can keep track of each request within a given period per Uri:

```rust
// diff-add
+use axum::http::uri::Uri;
// diff-add
+use chrono::{DateTime, Utc};
use my_app_config::Config;
// diff-add
+use std::collections::HashMap;
use std::default::Default;
// diff-add
use std::sync::atomic::{AtomicUsize, Ordering};
// diff-remove
-use std::sync::Arc;
// diff-add
+use std::sync::{Arc, Mutex};

/// The application's state that is available in [`crate::controllers`] and [`crate::middlewares`].
#[derive(Default)]
pub struct AppState {
    counter: AtomicUsize,
// diff-add
+    // keep track of number of request timestamps per uri
// diff-add
+    requests: Mutex<HashMap<Uri, Vec<DateTime<Utc>>>>,
}

impl AppState {
    pub fn count_visit(&self) {
        self.counter.fetch_add(1, Ordering::Relaxed);
    }

// diff-add
+    /// Allow a request if the limit hasn't been reached in the specified period (in seconds)
// diff-add
+    pub fn allow_request(&self, uri: Uri, period: u64, limit: usize) -> bool {
// diff-add
+        let throttle_time_limit = Utc::now() - std::time::Duration::from_secs(period);
// diff-add
+        let mut requests = self.requests.lock().unwrap();
// diff-add
+        let requests_for_uri = requests.entry(uri).or_insert(Vec::new());
// diff-add
+
// diff-add
+        requests_for_uri.retain(|x| x.to_utc() > throttle_time_limit);
// diff-add
+        requests_for_uri.push(Utc::now());
// diff-add
+
// diff-add
+        requests_for_uri.len() <= limit
// diff-add
+    }
}

/// The application's state as it is shared across the application, e.g. in controllers and middlewares.
pub async fn init_app_state(_config: Config) -> AppState {
    AppState {
        counter: AtomicUsize::new(0),
// diff-add
+        requests: Mutex::new(HashMap::new()),
    }
}
```

Finally, we can implement the the middleware in `web/middlewares/rate_limiter.rs`:

```rust
use crate::state::SharedAppState;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
+use std::net::SocketAddr;

#[tracing::instrument(skip_all, fields(rejection_reason = tracing::field::Empty))]
pub async fn rate_limiter(
// diff-add
    State(app_state): State<SharedAppState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
// diff-add
+    // only allow two requests per minute
// diff-add
+    if app_state.allow_request(req.uri().clone(), 60, 2) {
// diff-add
+        Ok(next.run(req).await)
// diff-add
+    } else {
// diff-add
+        Err(StatusCode::TOO_MANY_REQUESTS)
// diff-add
+    }
// diff-remove
-    todo!("Implement this (return `next.run(req).await` to continue processing the request or Err(StatusCode) to error out).")
}
```

and route it in `web/src/routes.rs`:

```rust
use crate::controllers::greeting;
// diff-add
+use crate::middlewares::rate_limiter::rate_limiter;
…
Router::new()
.route("/greet", get(greeting::hello))
.route("/greet_me", post(greeting::hello_person))
// diff-add
+.route_layer(middleware::from_fn_with_state(
// diff-add
+    shared_app_state.clone(),
// diff-add
+    rate_limiter,
// diff-add
+))
.with_state(shared_app_state)
…
```

Invoking the `/greet` endpoint three times now results in a 429 for the third invocation as expected:

```
» curl -i http://127.0.0.1:3000/greet
HTTP/1.1 200 OK
content-type: application/json
content-length: 27
date: Wed, 20 Nov 2024 12:48:46 GMT

{"hello":"world","visit":1}%

» curl -i http://127.0.0.1:3000/greet
HTTP/1.1 200 OK
content-type: application/json
content-length: 27
date: Wed, 20 Nov 2024 12:48:46 GMT

{"hello":"world","visit":2}%

» curl -i http://127.0.0.1:3000/greet
HTTP/1.1 429 Too Many Requests
content-length: 0
date: Wed, 20 Nov 2024 12:48:47 GMT
```

## Testing the middleware

Finally, let's add a test that ensures our users cannot request excessive greetings from the API. We can just add a new test case to `web/tests/api/greeting_test.rs`:

```rust
…
// diff-add
+#[test]
// diff-add
+async fn test_rate_limit(context: &TestContext) {
// diff-add
+    let response = context.app.request("/greet").send().await;
// diff-add
+    assert_that!(response.status(), eq(200));
// diff-add
+
// diff-add
+    let response = context.app.request("/greet").send().await;
// diff-add
+    assert_that!(response.status(), eq(200));
// diff-add
+
// diff-add
+    // only 2 requests are allowed per minute – this is expected to fail
// diff-add
+    let response = context.app.request("/greet").send().await;
// diff-add
+    assert_that!(response.status(), eq(429));
// diff-add
+}
```

The first two requests complete successfully while the third one fails as the rate limit kicks in. Since all test cases are isolated from each other, the `HashMap` that keeps track of the invocations is a different one for each test case and the tests cannot influence each other.

---

This concludes the tutorial on creating and working with a minimal Gerust project. We've seen the [project structure](../tutorial-minimal), how to work with and extend the [application state](./application-state), as well as working with middlewares. Continue to the tutorial on working with a [complete Gerust project](../tutorial-standard) to learn more about working with data.
