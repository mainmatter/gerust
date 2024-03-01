use anyhow::Context;
use axum::serve;
use {{crate_name}}_config::Config;
use pacesetter::{get_env, load_config};
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

    let addr = config
        .server
        .get_bind_addr()
        .context("Cannot get bind address!")?;
    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on {}", addr);
    serve(listener, app.into_make_service()).await?;

    Ok(())
}
