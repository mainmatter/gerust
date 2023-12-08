use {{crate_name}}_config::Config;

#[derive(Clone)]
pub struct AppState {
}

pub async fn app_state(_config: Config) -> AppState {
    AppState {}
}
