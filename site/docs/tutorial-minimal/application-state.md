---
sidebar_position: 1
---

# Application State

We'll add a simple visitor counter so that we can keep track of how many visitors access the application's endpoints.

## 1. Extending the Application State

The application state is defined in `web/src/state.rs`. That's where we'll add our new `counter` property to the `AppState` struct:

```rust
use my_app_config::Config;
 
/// The application's state that is available in [`crate::controllers`] and [`crate::middlewares`].
#[derive(Clone)]
// diff-remove
-pub struct AppState {}
// diff-add
+pub struct AppState {
// diff-add
+    pub counter: u16,
// diff-add
+}
 
/// Initializes the application state.
///
/// This function creates an [`AppState`] based on the current [`my_app_config::Config`].
pub async fn init_app_state(_config: Config) -> AppState {
// diff-remove
-    AppState {}
// diff-add
+    AppState {
// diff-add
+        counter: 0
// diff-add
+    }
 }
```

We can then modify the `web/src/controllers/greeting.rs` controller share with the visitors how many other people have accessed the endpoint before them:



counter
test
