use crate::cli::ui::{log, log_per_env, LogType};
use crate::cli::util::parse_env;
use crate::config::DatabaseConfig;
use crate::Environment;
use clap::{Parser, Subcommand};
use sqlx::postgres::{PgConnectOptions, PgConnection};
use sqlx::{
    migrate::{Migrate, Migrator},
    ConnectOptions, Connection, Executor,
};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use url::Url;

#[derive(Parser)]
#[command(author, version, about = "A CLI tool to manage the project's database.", long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true, help = "Choose the environment (development, test, production).", value_parser = parse_env, default_value = "development")]
    env: Environment,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Drop the database")]
    Drop,
    #[command(about = "Create the database")]
    Create,
    #[command(about = "Migrate the database")]
    Migrate,
    #[command(about = "Reset (drop, create, migrate) the database")]
    Reset,
    #[command(about = "Seed the database")]
    Seed,
}

pub async fn cli<F>(load_config: F)
where
    F: Fn(&Environment) -> DatabaseConfig,
{
    let cli = Cli::parse();
    let config = load_config(&cli.env);
    match cli.command {
        Commands::Drop => drop(&cli.env, &config).await,
        Commands::Create => create(&cli.env, &config).await,
        Commands::Migrate => migrate(&cli.env, &config).await,
        Commands::Seed => seed(&cli.env, &config).await,
        Commands::Reset => {
            drop(&cli.env, &config).await;
            create(&cli.env, &config).await;
            migrate(&cli.env, &config).await;
        }
    }
}

async fn drop(env: &Environment, config: &DatabaseConfig) {
    log_per_env(
        env,
        LogType::Info,
        "Dropping development database…",
        "Dropping test database…",
        "Dropping production database…",
    );

    let db_config = get_db_config(config);
    let db_name = db_config.get_database().unwrap();
    let mut root_connection = get_root_db_client(config).await;

    let query = format!("DROP DATABASE IF EXISTS {}", db_name);
    let result = root_connection.execute(query.as_str()).await;

    match result {
        Ok(_) => log(
            LogType::Success,
            format!("Database {} dropped successfully.", &db_name).as_str(),
        ),
        Err(_) => log(
            LogType::Error,
            format!("Dropping database {} failed!", &db_name).as_str(),
        ),
    }
}

async fn create(env: &Environment, config: &DatabaseConfig) {
    log_per_env(
        env,
        LogType::Info,
        "Creating development database…",
        "Creating test database…",
        "Creating production database…",
    );

    let db_config = get_db_config(config);
    let db_name = db_config.get_database().unwrap();
    let mut root_connection = get_root_db_client(config).await;

    let query = format!("CREATE DATABASE {}", db_name);
    let result = root_connection.execute(query.as_str()).await;

    match result {
        Ok(_) => log(
            LogType::Success,
            format!("Database {} created successfully.", &db_name).as_str(),
        ),
        Err(_) => log(
            LogType::Error,
            format!("Creating database {} failed!", &db_name).as_str(),
        ),
    }
}

async fn migrate(env: &Environment, config: &DatabaseConfig) {
    log_per_env(
        env,
        LogType::Info,
        "Migrating development database…",
        "Migrating test database…",
        "Migrating production database…",
    );

    let db_config = get_db_config(config);
    let migrator = Migrator::new(Path::new("db/migrations")).await.unwrap();
    let mut connection = db_config.connect().await.unwrap();

    connection.ensure_migrations_table().await.unwrap();

    let applied_migrations: HashMap<_, _> = connection
        .list_applied_migrations()
        .await
        .unwrap()
        .into_iter()
        .map(|m| (m.version, m))
        .collect();

    let mut applied = 0;
    for migration in migrator.iter() {
        if applied_migrations.get(&migration.version).is_none() {
            match connection.apply(migration).await {
                Ok(_) => log(
                    LogType::Info,
                    format!("Migration {} applied.", migration.version).as_str(),
                ),
                Err(_) => {
                    log(
                        LogType::Error,
                        format!("Coulnd't apply migration {}!", migration.version).as_str(),
                    );
                    return;
                }
            }
            applied += 1;
        }
    }

    log(
        LogType::Success,
        format!(
            "Migrated database successfully ({} migrations applied).",
            applied
        )
        .as_str(),
    );
}

async fn seed(env: &Environment, config: &DatabaseConfig) {
    log_per_env(
        env,
        LogType::Info,
        "Seeding development database…",
        "Seeding test database…",
        "Seeding production database…",
    );

    let mut connection = get_db_client(config).await;

    let statements = fs::read_to_string("./db/seeds.sql")
        .expect("Could not read seeds – make sure db/seeds.sql exists!");

    let mut transaction = connection.begin().await.unwrap();
    let result = transaction.execute(statements.as_str()).await;

    match result {
        Ok(_) => {
            let _ = transaction
                .commit()
                .await
                .map_err(|_| log(LogType::Error, "Seeding database failed!"));
            log(LogType::Info, "Seeded database.");
        }
        Err(_) => log(LogType::Error, "Seeding database failed!"),
    }
}

fn get_db_config(config: &DatabaseConfig) -> PgConnectOptions {
    let db_url = Url::parse(&config.url).expect("Invalid DATABASE_URL!");
    ConnectOptions::from_url(&db_url).expect("Invalid DATABASE_URL!")
}

async fn get_db_client(config: &DatabaseConfig) -> PgConnection {
    let db_config = get_db_config(config);
    let connection: PgConnection = Connection::connect_with(&db_config).await.unwrap();

    connection
}

async fn get_root_db_client(config: &DatabaseConfig) -> PgConnection {
    let db_config = get_db_config(config);
    let root_db_config = db_config.clone().database("postgres");
    let connection: PgConnection = Connection::connect_with(&root_db_config).await.unwrap();

    connection
}
