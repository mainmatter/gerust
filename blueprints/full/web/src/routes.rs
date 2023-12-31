use crate::controllers::tasks::{
    create_task, create_tasks, delete_task, get_task, get_tasks, update_task,
};
use crate::middlewares::auth::auth;
use crate::state::AppState;
use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};

pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .route("/tasks", post(create_task))
        .route("/tasks", put(create_tasks))
        .route("/tasks/:id", delete(delete_task))
        .route("/tasks/:id", put(update_task))
        .route_layer(middleware::from_fn_with_state(app_state.clone(), auth))
        .route("/tasks", get(get_tasks))
        .route("/tasks/:id", get(get_task))
        .with_state(app_state)
}
