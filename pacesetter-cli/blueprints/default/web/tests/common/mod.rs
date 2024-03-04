use {{crate_name}}_config::Config;
use {{crate_name}}_db::connect_pool;
use {{crate_name}}_web::routes::routes;
use {{crate_name}}_web::state::AppState;
use pacesetter::{
    load_config,
    test::helpers::{build_db_test_context, prepare_db, DbTestContext},
    Environment,
};
use std::cell::OnceCell;

pub async fn setup_with_db() -> DbTestContext {
    let init_config: OnceCell<Config> = OnceCell::new();
    let config = init_config.get_or_init(|| load_config(&Environment::Test).unwrap());

    let test_db_config = prepare_db(&config.database).await;
    let db_pool = connect_pool(test_db_config)
        .await
        .expect("Could not connect to database!");

    let app = routes(AppState {
        db_pool: db_pool.clone(),
    });

    build_db_test_context(app, db_pool)
}
