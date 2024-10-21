//! The {{crate_name}}_web crate contains the application's web interface which mainly are controllers implementing HTTP endpoints. It also includes the application tests that are black-box tests, interfacing with the application like any other HTTP client.

use anyhow::Context;
use axum::serve;
use {{crate_name}}_config::{Config, get_env, load_config};
use tokio::net::TcpListener;
use tracing::info;
use tracing_panic::panic_hook;
use tracing_subscriber::{filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// The application's controllers that implement request handlers.
pub mod controllers;
/// Middlewares that incoming requests are passed through before being passed to [`controllers`].
pub mod middlewares;
/// Contains the application's route definitions.
pub mod routes;
/// Contains the application state definition and functionality to initialize it.
pub mod state;
/// Contains the application's error type and related conversion implementation.
pub mod error;

/// Runs the application.
///
/// This function does all the work to initiatilize and run the application:
///
/// 1. Determine the environment the application is running in (see [`{{crate_name}}_config::get_env`])
/// 2. Load the configuration (see [`{{crate_name}}_config::load_config`])
/// 3. Initialize the application state (see [`state::init_app_state`])
/// 4. Initialize the application's router (see [`routes::init_routes`])
/// 5. Boot the application and start listening for requests on the configured interface and port
pub async fn run() -> anyhow::Result<()> {
    let env = get_env().context("Cannot get environment!")?;
    let config: Config = load_config(&env).context("Cannot load config!")?;

    let app_state = state::init_app_state(config.clone()).await;
    let app = routes::init_routes(app_state);

    let addr = config.server.addr();
    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on {}", &addr);
    serve(listener, app.into_make_service()).await?;

    Ok(())
}

/// Initializes tracing.
///
/// This function
///
/// * registers a [`tracing_subscriber::fmt::Subscriber`]
/// * registers a [`tracing_panic::panic_hook`]
///
/// The function respects the `RUST_LOG` if set or defaults to filtering spans and events with level [`tracing_subscriber::filter::LevelFilter::INFO`] and higher.
pub fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();

    std::panic::set_hook(Box::new(panic_hook));
}

/// Helpers that simplify writing application tests.
#[cfg(feature = "test-helpers")]
pub mod test_helpers;
