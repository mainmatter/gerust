use crate::controllers::greeting::hello;
use crate::state::AppState;
use axum::{Router, routing::get};

pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .route("/greet", get(hello))
        .with_state(app_state)
}
