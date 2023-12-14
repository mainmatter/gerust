pub async fn run() -> anyhow::Result<()> {
    let env = get_env();
    let config: Config = load_config(&env);

    let app_state = state::app_state(config.clone()).await;
    let app = routes::routes(app_state);

    let addr = config.server.get_bind_addr();
    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on {}", addr);
    run(redis/kafka, app.into_make_service()).await?;

    Ok(())
}