use anyhow::{anyhow, Context};
use clap::{Parser, Subcommand};
use cruet::string::{pluralize::to_plural, singularize::to_singular};
use pacesetter_util::ui::UI;
use std::fs::File;
use std::path::Path;
use std::time::SystemTime;

#[derive(Parser)]
#[command(author, version, about = "A CLI tool to generate project files.", long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, global = true, help = "Disable colored output.")]
    no_color: bool,

    #[arg(long, global = true, help = "Enable debug output.")]
    debug: bool,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Generate a migration")]
    Migration {
        #[arg(help = "The name of the migration.")]
        name: String,
    },
    #[command(about = "Generate an entity")]
    Entity {
        #[arg(help = "The name of the entity.")]
        name: String,
    },
}

pub async fn cli() {
    let cli = Cli::parse();
    let ui = UI::new(!cli.no_color, cli.debug);

    match cli.command {
        Commands::Migration { name } => {
            ui.info("Generating migration…");
            match generate_migration(name).await {
                Ok(file_name) => ui.success(&format!("Generated migration {}.", &file_name)),
                Err(e) => ui.error("Could not generate migration!", e),
            }
        }
        Commands::Entity { name } => {
            ui.info("Generating entity…");
            match generate_entity(name).await {
                Ok(file_name) => ui.success(&format!("Generated entity {}.", &file_name)),
                Err(e) => ui.error("Could not generate entity!", e),
            }
        }
    }
}

async fn generate_migration(name: String) -> Result<String, anyhow::Error> {
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .context("Failed to get timestamp!")?;
    let file_name = format!("V{}__{}.sql", timestamp.as_secs(), name);
    let full_file_name = format!("./db/migrations/{}", file_name);
    let path = Path::new(&full_file_name);

    if Path::new(path).exists() {
        Err(anyhow!("File already exists: {}", full_file_name))
    } else {
        File::create(path).context("Failed to create file!")?;

        Ok(file_name)
    }
}

async fn generate_entity(name: String) -> Result<String, anyhow::Error> {
    let name = to_singular(&name).to_lowercase();
    let name_plural = to_plural(&name);
    let file_name = format!("{}.rs", name_plural);
    let full_file_name = format!("./db/src/entities/{}", file_name);
    let path = Path::new(&full_file_name);
    File::create(path)?;

    Ok(file_name)
}
