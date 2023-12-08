use {{crate_name}}_config::Config;
use pacesetter::{cli::db::cli, load_config};

#[tokio::main]
async fn main() {
    cli(|env| {
        let config: Config = load_config(&env);
        config.database
    })
    .await;
}
