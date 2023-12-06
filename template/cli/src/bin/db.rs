use pacesetter::{cli::db::cli, load_config};
use {{crate_name}}_config::Config;

#[tokio::main]
async fn main() {
    cli(|env| {
        let config: Config = load_config(&env);
        config.database
    })
    .await;
}
