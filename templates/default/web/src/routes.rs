use crate::state::AppState;
use axum::Router;

pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .with_state(app_state)
}
