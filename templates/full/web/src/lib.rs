use axum::http::StatusCode;
use axum::serve;
use {{crate_name}}_config::Config;
use pacesetter::{get_env, load_config};
use std::fmt::{Debug, Display};
use tokio::net::TcpListener;
use tracing::info;

pub mod controllers;
pub mod middlewares;
pub mod routes;
pub mod state;

pub async fn run() -> anyhow::Result<()> {
    let env = get_env();
    let config: Config = load_config(&env);

    let app_state = state::app_state(config.clone()).await;
    let app = routes::routes(app_state);

    let addr = config.server.get_bind_addr();
    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on {}", addr);
    serve(listener, app.into_make_service()).await?;

    Ok(())
}

/// Helper function to create an internal error response while
/// taking care to log the error itself.
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
