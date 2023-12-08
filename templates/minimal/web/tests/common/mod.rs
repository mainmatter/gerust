use {{crate_name}}_config::Config;
use {{crate_name}}_web::routes::routes;
use {{crate_name}}_web::state::AppState;
use pacesetter::{
    load_config,
    test::helpers::{build_test_context, TestContext},
    Environment,
};
use std::cell::OnceCell;

pub async fn setup() -> TestContext {
    let init_config: OnceCell<Config> = OnceCell::new();
    let _config = init_config.get_or_init(|| load_config(&Environment::Test));

    let app = routes(AppState {});

    build_test_context(app)
}
