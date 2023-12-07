use axum::http::StatusCode;
use axum::serve;
use {{crate_name}}_config::Config;
use pacesetter::{get_env, load_config};
use tokio::net::TcpListener;
use tracing::info;

mod controllers;
mod middlewares;
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

pub fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
