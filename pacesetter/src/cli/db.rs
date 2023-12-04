use crate::cli::ui::{log, log_per_env, LogType};
use crate::cli::util::parse_env;
use crate::config::DatabaseConfig;
use crate::Environment;
use clap::{arg, value_parser, Command};
use sqlx::postgres::{PgConnectOptions, PgConnection};
use sqlx::{
    migrate::{Migrate, Migrator},
    ConnectOptions, Connection, Executor,
};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use url::Url;

fn commands() -> Command {
    Command::new("db")
        .about("A CLI tool to manage the project's database.")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("drop")
                .about("Drop the database")
                .arg(arg!(env: -e <ENV>).value_parser(value_parser!(String))),
        )
        .subcommand(
            Command::new("create")
                .about("Create the database")
                .arg(arg!(env: -e <ENV>).value_parser(value_parser!(String))),
        )
        .subcommand(
            Command::new("migrate")
                .about("Migrate the database")
                .arg(arg!(env: -e <ENV>).value_parser(value_parser!(String))),
        )
        .subcommand(
            Command::new("reset")
                .about("Reset the database (drop, re-create, migrate)")
                .arg(arg!(env: -e <ENV>).value_parser(value_parser!(String))),
        )
        .subcommand(
            Command::new("seed")
                .about("Seed the database")
                .arg(arg!(env: -e <ENV>).value_parser(value_parser!(String))),
        )
}

pub async fn cli<F>(load_config: F)
where
    F: Fn(&Environment) -> DatabaseConfig,
{
    let matches = commands().get_matches();

    match matches.subcommand() {
        Some(("drop", sub_matches)) => {
            let env = parse_env(sub_matches);
            let config = load_config(&env);
            drop(&env, &config).await;
        }
        Some(("create", sub_matches)) => {
            let env = parse_env(sub_matches);
            let config = load_config(&env);
            create(&env, &config).await;
        }
        Some(("migrate", sub_matches)) => {
            let env = parse_env(sub_matches);
            let config = load_config(&env);
            migrate(&env, &config).await;
        }
        Some(("reset", sub_matches)) => {
            let env = parse_env(sub_matches);
            let config = load_config(&env);
            drop(&env, &config).await;
            create(&env, &config).await;
            migrate(&env, &config).await;
        }
        Some(("seed", sub_matches)) => {
            let env = parse_env(sub_matches);
            let config = load_config(&env);
            seed(&env, &config).await;
        }
        _ => unreachable!(),
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
