use {{crate_name}}_config::Config;
use {{crate_name}}_web::routes::routes;
use {{crate_name}}_web::state::AppState;
use pacesetter::{
    load_config,
    test::helpers::{build_db_test_context, prepare_db, DbTestContext},
    Environment,
};
use sqlx::postgres::PgPoolOptions;
use std::cell::OnceCell;

pub async fn setup_with_db() -> DbTestContext {
    let init_config: OnceCell<Config> = OnceCell::new();
    let config = init_config.get_or_init(|| load_config(&Environment::Test));

    let db_config = prepare_db(&config.database).await;
    let db_pool = PgPoolOptions::new()
        .connect_with(db_config.clone())
        .await
        .expect("Could not connect to database!");

    let app = routes(AppState {
        db_pool: db_pool.clone(),
    });

    build_db_test_context(app, db_pool, db_config)
}
