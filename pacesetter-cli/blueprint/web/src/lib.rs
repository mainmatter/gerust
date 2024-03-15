use anyhow::Context;
use axum::{serve, http::StatusCode};
use {{crate_name}}_config::{Config, load_config};
use pacesetter::get_env;
use std::fmt::{Debug, Display};
use tokio::net::TcpListener;
use tracing::info;

pub mod controllers;
pub mod middlewares;
pub mod routes;
pub mod state;

pub async fn run() -> anyhow::Result<()> {
    let env = get_env().context("Cannot get environment!")?;
    let config: Config = load_config(&env).context("Cannot load config!")?;

    let app_state = state::app_state(config.clone()).await;
    let app = routes::routes(app_state);

    let addr = config.server.addr();
    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on {}", &addr);
    serve(listener, app.into_make_service()).await?;

    Ok(())
}

/// Helper function to create an internal error response while
/// taking care to log the error itself.
///
/// This is useful to avoid duplication in web endpoints – in the case of unrecoverable errors,
/// there's only really two things to do anyway, which you'll want to do in every such case:
///
/// 1. create an error-level tracing event
/// 2. respond with an axum::http::StatusCode::INTERNAL_SERVER_ERROR status code
///
/// Example
/// ```rust
/// pub async fn read_all(
///     State(app_state): State<AppState>,
/// ) -> Result<Json<Vec<Task>>, StatusCode> {
///     let tasks = tasks::load_all(&app_state.db_pool)
///         .await
///         .map_err(internal_error)?;
///
///     Ok(Json(tasks))
/// }
/// ```
pub fn internal_error<E>(e: E) -> StatusCode
where
    // Some "error-like" types (e.g. `anyhow::Error`) don't implement the error trait, therefore
    // we "downgrade" to simply requiring `Debug` and `Display`, the traits
    // we actually need for logging purposes.
    E: Debug + Display,
{
    tracing::error!(err.msg = %e, err.details = ?e, "Internal server error");
    // We don't want to leak internal implementation details to the client
    // via the error response, so we just return an opaque internal server.
    StatusCode::INTERNAL_SERVER_ERROR
}

