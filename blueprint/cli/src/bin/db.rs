use anyhow::{anyhow, Context};
use clap::{Parser, Subcommand};
use {{crate_name}}_cli::util::ui::UI;
use {{crate_name}}_config::DatabaseConfig;
use {{crate_name}}_config::{load_config, parse_env, Config, Environment};
use guppy::{Version, VersionReq};
use sqlx::postgres::{PgConnectOptions, PgConnection};
use sqlx::{
    migrate::{AppliedMigration, Migrate, Migration, MigrationType, Migrator},
    ConnectOptions, Connection, Executor,
};
use tokio::io::{stdin, AsyncBufReadExt};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{ExitCode, Stdio};
use url::Url;

/// The version of sqlx-cli required
const SQLX_CLI_VERSION: &str = "0.8";

#[tokio::main]
async fn main() -> ExitCode {
    let mut stdout = std::io::stdout();
    let mut stderr = std::io::stderr();

    let args = Cli::parse();
    let mut ui = UI::new(&mut stdout, &mut stderr, !args.no_color, !args.quiet);

    match cli(&mut ui, args).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            ui.error(e.to_string().as_str(), &e);
            ExitCode::FAILURE
        }
    }
}

#[derive(Parser)]
#[command(author, version, about = "A CLI tool to manage the project's database.", long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true, help = "Choose the environment (development, test, production).", value_parser = parse_env, default_value = "development")]
    env: Environment,

    #[arg(long, global = true, help = "Disable colored output.")]
    no_color: bool,

    #[arg(long, global = true, help = "Disable debug output.")]
    quiet: bool,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Drop the database")]
    Drop,
    #[command(about = "Create the database")]
    Create,
    #[command(about = "Migrate the database")]
    Migrate,
    #[command(about = "Rollback database migrations")]
    Rollback {
        #[arg(short, long, default_value = "1", help = "Number of migrations to roll back.", conflicts_with = "to")]
        steps: u32,

        #[arg(short, long, help = "Roll back up to (but not including) the migration with this name.")]
        to: Option<String>,
    },
    #[command(about = "Reset (drop, create, migrate) the database")]
    Reset,
    #[command(about = "Seed the database")]
    Seed,
    #[command(about = "Generate query metadata to support offline compile-time verification")]
    Prepare,
}

#[allow(missing_docs)]
async fn cli(ui: &mut UI<'_>, cli: Cli) -> Result<(), anyhow::Error> {
    let config: Result<Config, anyhow::Error> = load_config(&cli.env);
    match config {
        Ok(config) => {
            let migrations_path = db_package_root()?.join("migrations");
            match cli.command {
                Commands::Drop => {
                    ui.info(&format!("Dropping {} database…", &cli.env));
                    let db_name = drop(&config.database)
                        .await
                        .context("Could not drop database!")?;
                    ui.success(&format!("Dropped database {db_name} successfully."));
                    Ok(())
                }
                Commands::Create => {
                    ui.info(&format!("Creating {} database…", &cli.env));
                    let db_name = create(&config.database)
                        .await
                        .context("Could not create database!")?;
                    ui.success(&format!("Created database {db_name} successfully."));
                    Ok(())
                }
                Commands::Migrate => {
                    ui.info(&format!("Migrating {} database…", &cli.env));
                    ui.indent();
                    let migrations = migrate(ui, &config.database, &migrations_path)
                        .await
                        .context("Could not migrate database!");
                    ui.outdent();
                    let migrations = migrations?;
                    ui.success(&format!("{migrations} migrations applied."));
                    Ok(())
                }
                Commands::Rollback { steps, to } => {
                    if let Some(ref name) = to {
                        ui.info(&format!("Rolling back {} database to \"{name}\"…", &cli.env));
                    } else {
                        ui.info(&format!("Rolling back {} database ({steps} step(s))…", &cli.env));
                    }
                    ui.indent();
                    let result = rollback(ui, &config.database, &migrations_path, steps, to.as_deref())
                        .await
                        .context("Could not rollback database!");
                    ui.outdent();
                    let reverted = result?;
                    ui.success(&format!("{reverted} migration(s) reverted."));
                    Ok(())
                }
                Commands::Seed => {
                    ui.info(&format!("Seeding {} database…", &cli.env));
                    seed(&config.database)
                        .await
                        .context("Could not seed database!")?;
                    ui.success("Seeded database successfully.");
                    Ok(())
                }
                Commands::Reset => {
                    ui.info(&format!("Resetting {} database…", &cli.env));
                    ui.indent();
                    let result = reset(ui, &config.database, &migrations_path)
                        .await
                        .context("Could not reset the database!");
                    ui.outdent();
                    let db_name = result?;
                    ui.success(&format!("Reset database {db_name} successfully."));
                    Ok(())
                }
                Commands::Prepare => {
                    if let Err(e) = ensure_sqlx_cli_installed(ui).await {
                        return Err(e.context("Error ensuring sqlx-cli is installed!"));
                    }
                
                    let cargo = get_cargo_path().expect("Existence of CARGO env var is asserted by calling `ensure_sqlx_cli_installed`");
                
                    let mut sqlx_prepare_command = {
                        let mut cmd = tokio::process::Command::new(&cargo);
                
                        cmd.args(["sqlx", "prepare", "--", "--all-targets", "--all-features"]);
                
                        let cmd_cwd = db_package_root().context("Error finding the root of the db package!")?;
                        cmd.current_dir(cmd_cwd);
                
                        cmd.env("DATABASE_URL", &config.database.url);
                        cmd
                    };
                
                    let o = sqlx_prepare_command
                        .output()
                        .await
                        .context("Could not run {cargo} sqlx prepare!")?;
                    if !o.status.success() {
                        let error = anyhow!(String::from_utf8_lossy(&o.stdout).to_string()).context("Error generating query metadata. Are you sure the database is running and all migrations are applied?");
                        return Err(error);
                    }
                
                    ui.success("Query data written to db/.sqlx directory; please check this into version control.");
                    Ok(())
                }
            }
        }
        Err(e) => {
            Err(e.context("Could not load config!"))
        }
    }
}

async fn drop(config: &DatabaseConfig) -> Result<String, anyhow::Error> {
    let db_config = get_db_config(config);
    let db_name = db_config
        .get_database()
        .context("Failed to get database name!")?;
    let mut root_connection = get_root_db_client(config).await;

    let query = format!("DROP DATABASE {db_name}");
    root_connection
        .execute(query.as_str())
        .await
        .context("Failed to drop database!")?;

    Ok(String::from(db_name))
}

async fn create(config: &DatabaseConfig) -> Result<String, anyhow::Error> {
    let db_config = get_db_config(config);
    let db_name = db_config
        .get_database()
        .context("Failed to get database name!")?;
    let mut root_connection = get_root_db_client(config).await;

    let query = format!("CREATE DATABASE {db_name}");
    root_connection
        .execute(query.as_str())
        .await
        .context("Failed to create database!")?;

    Ok(String::from(db_name))
}

struct SqlxMigrator;

impl SqlxMigrator {
    /// Build a `Migrator` for all forward (up) migrations: both reversible
    /// `{version}__{name}/up.sql` directories and simple `{version}_{name}.sql` files.
    fn up_migrator(migrations_path: &Path) -> Result<Migrator, anyhow::Error> {
        let migrations = Self::read_all_up_migrations(migrations_path)?;
        Self::build_migrator(migrations)
    }

    /// Build a `Migrator` for reversible down migrations from `{version}__{name}/down.sql` directories.
    fn down_migrator(migrations_path: &Path) -> Result<Migrator, anyhow::Error> {
        let mut migrations =
            Self::read_dir_migrations(migrations_path, "down.sql", MigrationType::ReversibleDown)?;
        migrations.sort_by_key(|m| m.version);
        Self::build_migrator(migrations)
    }

    /// Wrap a `Vec<Migration>` into a `Migrator` with default settings.
    fn build_migrator(migrations: Vec<Migration>) -> Result<Migrator, anyhow::Error> {
        Ok(Migrator {
            migrations: migrations.into(),
            ..Migrator::DEFAULT
        })
    }

    /// Read all up migrations from `migrations_path` in a single directory scan.
    /// Collects both `{version}__{name}/up.sql` (reversible) and `{version}_{name}.sql` (simple) migrations.
    fn read_all_up_migrations(migrations_path: &Path) -> Result<Vec<Migration>, anyhow::Error> {
        let entries = fs::read_dir(migrations_path)
            .context("Failed to read migrations directory")?;

        let mut dirs = Vec::new();
        let mut files = Vec::new();

        for entry in entries {
            let path = entry?.path();
            if path.is_dir() {
                dirs.push(path);
            } else if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("sql") {
                files.push(path);
            }
        }

        dirs.sort();
        files.sort();

        let mut migrations = Vec::new();

        for dir in &dirs {
            let migration_file = dir.join("up.sql");
            if !migration_file.exists() {
                continue;
            }

            let version = Self::extract_dir_version(dir)?;
            let description = Self::extract_dir_description(dir)?;
            let contents = fs::read_to_string(&migration_file)
                .with_context(|| format!("Failed to read {}", migration_file.display()))?;

            migrations.push(Migration::new(
                version,
                description.into(),
                MigrationType::ReversibleUp,
                contents.into(),
                false,
            ));
        }

        for file in &files {
            let file_stem = file
                .file_stem()
                .and_then(|s| s.to_str())
                .context("Invalid migration file name")?;

            let (version_str, description) = file_stem.split_once('_').with_context(|| {
                format!("Migration file name must contain '_' separator: {file_stem}")
            })?;

            let version: i64 = version_str
                .parse()
                .with_context(|| format!("Invalid migration version: {version_str}"))?;

            let contents = fs::read_to_string(file)
                .with_context(|| format!("Failed to read {}", file.display()))?;

            migrations.push(Migration::new(
                version,
                description.to_string().into(),
                MigrationType::Simple,
                contents.into(),
                false,
            ));
        }

        migrations.sort_by_key(|m| m.version);
        Ok(migrations)
    }

    /// Read migrations from subdirectories matching `{version}__{name}/{file_name}`.
    fn read_dir_migrations(
        migrations_path: &Path,
        file_name: &str,
        migration_type: MigrationType,
    ) -> Result<Vec<Migration>, anyhow::Error> {
        let entries = fs::read_dir(migrations_path)
            .context("Failed to read migrations directory")?;

        let mut dirs: Vec<PathBuf> = entries
            .filter_map(|res| res.ok())
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        dirs.sort();

        let mut migrations = Vec::new();

        for dir in &dirs {
            let migration_file = dir.join(file_name);
            if !migration_file.exists() {
                continue;
            }

            let version = Self::extract_dir_version(dir)?;
            let description = Self::extract_dir_description(dir)?;
            let contents = fs::read_to_string(&migration_file)
                .with_context(|| format!("Failed to read {}", migration_file.display()))?;

            migrations.push(Migration::new(
                version,
                description.into(),
                migration_type,
                contents.into(),
                false,
            ));
        }

        Ok(migrations)
    }

    /// Extract the numeric version from a `{version}__{name}` directory name.
    fn extract_dir_version(dir: &Path) -> Result<i64, anyhow::Error> {
        let dir_name = dir
            .file_name()
            .and_then(|n| n.to_str())
            .context("Invalid migration directory name")?;

        let version_str = dir_name
            .split("__")
            .next()
            .context("Migration directory name must contain '__' separator")?;

        version_str
            .parse::<i64>()
            .with_context(|| format!("Invalid migration version: {version_str}"))
    }

    /// Extract the description from a `{version}__{name}` directory name.
    fn extract_dir_description(dir: &Path) -> Result<String, anyhow::Error> {
        let dir_name = dir
            .file_name()
            .and_then(|n| n.to_str())
            .context("Invalid migration directory name")?;

        let parts: Vec<&str> = dir_name.splitn(2, "__").collect();
        if parts.len() > 1 {
            Ok(parts[1].to_string())
        } else {
            Ok(dir_name.to_string())
        }
    }
}

async fn prepare_migrations(
    config: &DatabaseConfig,
) -> Result<(PgConnection, HashMap<i64, AppliedMigration>), anyhow::Error> {
    let db_config = get_db_config(config);
    let mut connection = db_config
        .connect()
        .await
        .context("Failed to connect to database!")?;

    connection
        .ensure_migrations_table()
        .await
        .context("Failed to ensure migrations table!")?;

    let applied_migrations: HashMap<_, _> = connection
        .list_applied_migrations()
        .await
        .context("Failed to list applied migrations!")?
        .into_iter()
        .map(|m| (m.version, m))
        .collect();

    Ok((connection, applied_migrations))
}

async fn migrate(ui: &mut UI<'_>, config: &DatabaseConfig, migrations_path: &Path) -> Result<i32, anyhow::Error> {
    let migrator = SqlxMigrator::up_migrator(migrations_path)
        .context("Failed to build migrator!")?;
    let (mut connection, applied_migrations) = prepare_migrations(config).await?;

    let mut applied = 0;
    for migration in migrator.iter() {
        if !applied_migrations.contains_key(&migration.version) {
            connection
                .apply(migration)
                .await
                .context("Failed to apply migration!")?;
            ui.log(&format!("Applied migration {}.", migration.version));
            applied += 1;
        }
    }

    Ok(applied)
}

async fn rollback(ui: &mut UI<'_>, config: &DatabaseConfig, migrations_path: &Path, steps: u32, to: Option<&str>) -> Result<i32, anyhow::Error> {
    let migrator = SqlxMigrator::down_migrator(migrations_path)
        .context("Failed to build migrator!")?;
    let (mut connection, applied_migrations) = prepare_migrations(config).await?;

    let mut applied_versions: Vec<i64> = applied_migrations.keys().copied().collect();
    applied_versions.sort_unstable();
    applied_versions.reverse();

    let target_version = if let Some(name) = to {
        let version = migrator
            .iter()
            .find(|m| *m.description == *name)
            .map(|m| m.version)
            .with_context(|| format!("No migration found with name \"{name}\""))?;

        if !applied_migrations.contains_key(&version) {
            return Err(anyhow!("Migration \"{name}\" has not been applied."));
        }

        Some(version)
    } else {
        None
    };

    let mut reverted = 0;
    for version in applied_versions {
        if let Some(target) = target_version {
            if version <= target {
                break;
            }
        } else if reverted >= steps as i32 {
            break;
        }

        if let Some(migration) = migrator.iter().find(|m| m.version == version) {
            connection
                .revert(migration)
                .await
                .with_context(|| format!("Failed to revert migration {version}!"))?;
            ui.log(&format!("Reverted migration {version}."));
            reverted += 1;
        }
    }

    Ok(reverted)
}

async fn seed(config: &DatabaseConfig) -> Result<(), anyhow::Error> {
    let mut connection = get_db_client(config).await;

    let statements = fs::read_to_string("./db/seeds.sql")
        .expect("Could not read seeds – make sure db/seeds.sql exists!");

    let mut transaction = connection
        .begin()
        .await
        .context("Failed to start transaction!")?;
    transaction
        .execute(statements.as_str())
        .await
        .context("Failed to execute seeds!")?;
    transaction
        .commit()
        .await
        .context("Failed to commit transaction!")?;

    Ok(())
}

async fn reset(ui: &mut UI<'_>, config: &DatabaseConfig, migrations_path: &Path) -> Result<String, anyhow::Error> {
    ui.log("Dropping database…");
    drop(config).await?;
    ui.log("Recreating database…");
    let db_name = create(config).await?;
    ui.log("Migrating database…");
    ui.indent();
    let migration_result = migrate(ui, config, migrations_path).await;
    ui.outdent();

    match migration_result {
        Ok(_) => Ok(db_name),
        Err(e) => Err(e),
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

fn get_cargo_path() -> Result<String, anyhow::Error> {
    std::env::var("CARGO")
        .map_err(|_| anyhow!("Please invoke me using Cargo, e.g.: `cargo db <ARGS>`"))
}

/// Ensure that the correct version of sqlx-cli is installed,
/// and install it if it isn't.
async fn ensure_sqlx_cli_installed(ui: &mut UI<'_>) -> Result<(), anyhow::Error> {
    let sqlx_version_req = VersionReq::parse(SQLX_CLI_VERSION)
        .expect("SQLX_CLI_VERSION value is not a valid semver version requirement.");

    let cargo = get_cargo_path()?;

    let current_version = installed_sqlx_cli_version(&cargo).await?;
    if let Some(version) = &current_version {
        if sqlx_version_req.matches(version) {
            // sqlx-cli is already installed and of the correct version, nothing to do
            return Ok(());
        }
    }

    let curr_vers_msg = current_version.map_or_else(
        || "sqlx-cli is currently not installed.".to_string(),
        |v| format!("The currently installed version is {v}.")
    );
    ui.info(&format!(
        "This command requires a version of sqlx-cli that is \
        compatible with version {SQLX_CLI_VERSION}, which is not installed yet. \
        {curr_vers_msg} \
        Would you like to install the latest compatible version now? [Y/n]"
    ));

    // Read user answer
    {
        let mut buf = String::new();
        let mut reader = tokio::io::BufReader::new(stdin());
        loop {
            reader.read_line(&mut buf).await?;
            let line = buf.to_ascii_lowercase();
            let line = line.trim_end();
            if matches!(line, "" | "y" | "yes") {
                ui.info("Starting installation of sqlx-cli...");
                break;
            } else if matches!(line, "n" | "no") {
                return Err(anyhow!("Installation of sqlx-cli canceled."));
            }
            ui.info("Please enter y or n");
            buf.clear();
        }
    }

    let mut cargo_install_command = {
        let mut cmd = tokio::process::Command::new(&cargo);
        cmd.args([
            "install",
            "sqlx-cli",
            "--version",
            &format!("^{SQLX_CLI_VERSION}"),
            "--locked",
            // Install unoptimized version,
            // making the process much faster.
            // sqlx-cli doesn't really need to be
            // performant anyway for our purposes
            "--debug",
        ]);
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());
        cmd
    };

    let mut child = cargo_install_command.spawn()?;

    let status = child.wait().await?;
    if !status.success() {
        return Err(anyhow!(
            "Something went wrong when installing sqlx-cli. Please check output"
        ));
    }

    match installed_sqlx_cli_version(&cargo).await {
        Ok(Some(v)) if sqlx_version_req.matches(&v) => {
            ui.success(&format!("Successfully installed sqlx-cli {v}"));
            Ok(())
        }
        Ok(Some(v)) => Err(anyhow!("Could not update sqlx cli. Current version: {v}")),
        Ok(None) => Err(anyhow!("sqlx-cli was not detected after installation")),
        Err(e) => Err(e),
    }
}

/// Get the version of the current sqlx-cli installation, if any.
async fn installed_sqlx_cli_version(cargo: &str) -> Result<Option<Version>, anyhow::Error> {
    /// The expected prefix of the version output of sqlx-cli >= 0.8
    const SQLX_CLI_VERSION_STRING_PREFIX: &str = "sqlx-cli-sqlx";
    /// The expected prefix of the version output of sqlx-cli < 0.8
    const SQLX_CLI_VERSION_STRING_PREFIX_OLD: &str = "cargo-sqlx";

    fn error_parsing_version() -> anyhow::Error {
        anyhow!(
                "Error parsing sqlx-cli version. Please install the \
                correct version manually using `cargo install sqlx-cli \
                --version ^{SQLX_CLI_VERSION} --locked`"
            )
    }

    let mut cargo_sqlx_command = {
        let mut cmd = tokio::process::Command::new(cargo);
        cmd.args(["sqlx", "--version"]);
        cmd
    };

    let out = cargo_sqlx_command.output().await?;
    if !out.status.success() {
        // Failed to run the command for some reason,
        // we conclude that sqlx-cli is not installed.
        return Ok(None);
    }

    let Ok(stdout) = String::from_utf8(out.stdout) else {
        return Err(error_parsing_version());
    };

    let Some(version) = stdout
        .strip_prefix(SQLX_CLI_VERSION_STRING_PREFIX)
        .or_else(|| stdout.strip_prefix(SQLX_CLI_VERSION_STRING_PREFIX_OLD))
        .map(str::trim)
    else {
        return Err(error_parsing_version());
    };

    let Ok(version) = Version::parse(version) else {
        return Err(error_parsing_version());
    };

    Ok(Some(version))
}

/// Find the root of the db package in the gerust workspace.
fn db_package_root() -> Result<PathBuf, anyhow::Error> {
    Ok(PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR")
            .map_err(|e| anyhow!(e).context("This command needs to be invoked using cargo"))?,
    )
    .join("..")
    .join("db")
    .canonicalize()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Skip the test if DATABASE_URL is not set instead of failing.
    macro_rules! require_db {
        () => {
            match std::env::var("DATABASE_URL") {
                Ok(url) => url,
                Err(_) => {
                    eprintln!("Skipping test: DATABASE_URL not set");
                    return;
                }
            }
        };
    }

    /// Create a simple (non-reversible) flat `.sql` migration file.
    fn create_simple_migration(base: &Path, version: &str, name: &str, sql: &str) {
        let file = base.join(format!("{version}_{name}.sql"));
        fs::write(file, sql).unwrap();
    }

    /// Create a reversible (up + down) migration directory.
    fn create_reversible_migration(
        base: &Path,
        version: &str,
        name: &str,
        up_sql: &str,
        down_sql: &str,
    ) {
        let dir = base.join(format!("{version}__{name}"));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("up.sql"), up_sql).unwrap();
        fs::write(dir.join("down.sql"), down_sql).unwrap();
    }

    async fn setup_test_db(suffix: &str) -> (PgConnectOptions, String) {
        let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let base_config: PgConnectOptions = db_url.parse().expect("Invalid DATABASE_URL!");
        let root_config = base_config.clone().database("postgres");
        let mut conn: PgConnection = Connection::connect_with(&root_config).await.unwrap();

        let db_name = format!("gerust_test_migrations_{suffix}");
        conn.execute(format!("DROP DATABASE IF EXISTS {db_name}").as_str())
            .await
            .unwrap();
        conn.execute(format!("CREATE DATABASE {db_name}").as_str())
            .await
            .unwrap();

        let test_config = base_config.database(&db_name);
        (test_config, db_name)
    }

    async fn teardown_test_db(db_name: &str) {
        let db_url = std::env::var("DATABASE_URL").unwrap();
        let base_config: PgConnectOptions = db_url.parse().expect("Invalid DATABASE_URL!");
        let root_config = base_config.database("postgres");
        let mut conn: PgConnection = Connection::connect_with(&root_config).await.unwrap();
        conn.execute(format!("DROP DATABASE IF EXISTS {db_name}").as_str())
            .await
            .unwrap();
    }

    async fn table_exists(conn: &mut PgConnection, table_name: &str) -> bool {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = $1)",
        )
        .bind(table_name)
        .fetch_one(conn)
        .await
        .unwrap();
        exists
    }

    #[tokio::test]
    async fn test_migrate_simple_up_only() {
        let _db_url = require_db!();
        let (db_config, db_name) = setup_test_db("simple").await;

        let tmp = TempDir::new().unwrap();
        create_simple_migration(
            tmp.path(),
            "1000",
            "create_foo",
            "CREATE TABLE foo (id integer PRIMARY KEY);",
        );
        create_simple_migration(
            tmp.path(),
            "2000",
            "create_bar",
            "CREATE TABLE bar (id integer PRIMARY KEY);",
        );

        let migrator = SqlxMigrator::up_migrator(tmp.path()).unwrap();
        let mut conn = db_config.connect().await.unwrap();

        conn.ensure_migrations_table().await.unwrap();

        for migration in migrator.iter() {
            assert!(
                !migration.migration_type.is_down_migration(),
                "up_migrator should only produce up migrations"
            );
            conn.apply(migration).await.unwrap();
        }

        assert!(table_exists(&mut conn, "foo").await);
        assert!(table_exists(&mut conn, "bar").await);

        let applied = conn.list_applied_migrations().await.unwrap();
        assert_eq!(applied.len(), 2);

        // down_migrator should produce no migrations for simple (up-only) dirs
        let down_migrator = SqlxMigrator::down_migrator(tmp.path()).unwrap();
        assert_eq!(
            down_migrator.iter().count(),
            0,
            "simple migrations should have no down migrations"
        );

        std::mem::drop(conn);
        teardown_test_db(&db_name).await;
    }

    #[tokio::test]
    async fn test_migrate_and_rollback_reversible() {
        let _db_url = require_db!();
        let (db_config, db_name) = setup_test_db("reversible").await;

        let tmp = TempDir::new().unwrap();
        create_reversible_migration(
            tmp.path(),
            "1000",
            "create_foo",
            "CREATE TABLE foo (id integer PRIMARY KEY);",
            "DROP TABLE IF EXISTS foo;",
        );
        create_reversible_migration(
            tmp.path(),
            "2000",
            "create_bar",
            "CREATE TABLE bar (id integer PRIMARY KEY);",
            "DROP TABLE IF EXISTS bar;",
        );

        // Apply up migrations
        let up_migrator = SqlxMigrator::up_migrator(tmp.path()).unwrap();
        let mut conn = db_config.connect().await.unwrap();
        conn.ensure_migrations_table().await.unwrap();

        for migration in up_migrator.iter() {
            conn.apply(migration).await.unwrap();
        }

        assert!(table_exists(&mut conn, "foo").await);
        assert!(table_exists(&mut conn, "bar").await);
        assert_eq!(conn.list_applied_migrations().await.unwrap().len(), 2);

        // Rollback the latest migration
        let down_migrator = SqlxMigrator::down_migrator(tmp.path()).unwrap();
        let bar_down = down_migrator
            .iter()
            .find(|m| m.version == 2000)
            .expect("down migration for version 2000 should exist");
        conn.revert(bar_down).await.unwrap();

        assert!(
            table_exists(&mut conn, "foo").await,
            "foo should still exist after rolling back bar"
        );
        assert!(
            !table_exists(&mut conn, "bar").await,
            "bar should be gone after rollback"
        );
        assert_eq!(conn.list_applied_migrations().await.unwrap().len(), 1);

        // Rollback the remaining migration
        let foo_down = down_migrator
            .iter()
            .find(|m| m.version == 1000)
            .expect("down migration for version 1000 should exist");
        conn.revert(foo_down).await.unwrap();

        assert!(
            !table_exists(&mut conn, "foo").await,
            "foo should be gone after rollback"
        );
        assert_eq!(conn.list_applied_migrations().await.unwrap().len(), 0);

        std::mem::drop(conn);
        teardown_test_db(&db_name).await;
    }

    #[tokio::test]
    async fn test_migrate_mixed_simple_and_reversible() {
        let _db_url = require_db!();
        let (db_config, db_name) = setup_test_db("mixed").await;

        let tmp = TempDir::new().unwrap();
        // First migration: simple (up-only, not rollbackable)
        create_simple_migration(
            tmp.path(),
            "1000",
            "create_foo",
            "CREATE TABLE foo (id integer PRIMARY KEY);",
        );
        // Second migration: reversible (rollbackable)
        create_reversible_migration(
            tmp.path(),
            "2000",
            "create_bar",
            "CREATE TABLE bar (id integer PRIMARY KEY);",
            "DROP TABLE IF EXISTS bar;",
        );

        // Apply all up migrations (both simple and reversible)
        let up_migrator = SqlxMigrator::up_migrator(tmp.path()).unwrap();
        let mut conn = db_config.connect().await.unwrap();
        conn.ensure_migrations_table().await.unwrap();

        for migration in up_migrator.iter() {
            conn.apply(migration).await.unwrap();
        }

        assert!(table_exists(&mut conn, "foo").await);
        assert!(table_exists(&mut conn, "bar").await);
        assert_eq!(conn.list_applied_migrations().await.unwrap().len(), 2);

        // down_migrator should only include the reversible migration
        let down_migrator = SqlxMigrator::down_migrator(tmp.path()).unwrap();
        let down_migrations: Vec<_> = down_migrator.iter().collect();
        assert_eq!(
            down_migrations.len(),
            1,
            "only the reversible migration should have a down migration"
        );
        assert_eq!(down_migrations[0].version, 2000);

        // Rollback the reversible migration
        conn.revert(&down_migrations[0]).await.unwrap();

        assert!(
            table_exists(&mut conn, "foo").await,
            "simple migration table should still exist"
        );
        assert!(
            !table_exists(&mut conn, "bar").await,
            "reversible migration table should be gone"
        );

        let applied = conn.list_applied_migrations().await.unwrap();
        assert_eq!(applied.len(), 1);
        assert_eq!(applied[0].version, 1000);

        std::mem::drop(conn);
        teardown_test_db(&db_name).await;
    }
}
