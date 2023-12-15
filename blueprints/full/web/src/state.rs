use {{crate_name}}_config::Config;
use {{crate_name}}_db::connect_pool;
use sqlx::postgres::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
}

pub async fn app_state(config: Config) -> AppState {
    let db_pool = connect_pool(config.database)
        .await
        .expect("Could not connect to database!");

    AppState { db_pool }
}
