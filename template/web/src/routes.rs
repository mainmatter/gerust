use crate::controllers::example::hello;
use crate::state::AppState;
use axum::{routing::get, Router};

pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .route("/example", get(hello))
        .with_state(app_state)
}
