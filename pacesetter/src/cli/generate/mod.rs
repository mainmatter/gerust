use anyhow::{anyhow, Context};
use clap::{Parser, Subcommand};
use cruet::{
    string::{pluralize::to_plural, singularize::to_singular},
    to_title_case,
};
use pacesetter_util::ui::UI;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::SystemTime;

static VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), " (", env!("VERGEN_GIT_SHA"), ")");

static BLUEPRINTS_DIR: include_dir::Dir =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/src/cli/generate/blueprints");

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
    let struct_name = to_title_case(&name);

    let tmp_directory = std::env::temp_dir().join(format!("pacesetter-blueprint-{}", VERSION));
    std::fs::create_dir_all(&tmp_directory)
        .context("Failed to create a temporary directory for Pacesetter's blueprints")
        .unwrap();
    BLUEPRINTS_DIR
        .extract(&tmp_directory)
        .context("Failed to extract Pacesetter's blueprints to a temporary directory")
        .unwrap();
    let blueprint_path = tmp_directory.join("entity").join("file.rs.liquid");
    let blueprint_path = blueprint_path
        .to_str()
        .unwrap_or("Failed to get full path to Pacesetter's blueprint");
    let template_source =
        fs::read_to_string(blueprint_path).expect("Should have been able to read the file");
    let template = liquid::ParserBuilder::with_stdlib()
        .build()
        .unwrap()
        .parse(&template_source)
        .unwrap();
    let variables = liquid::object!({
        "entity_struct_name": struct_name,
        "entity_singular_name": name,
        "entity_plural_name": name_plural,
    });
    let output = template.render(&variables).unwrap();

    let file_name = format!("{}.rs", name_plural);
    let full_file_name = format!("./db/src/entities/{}", file_name);
    let path = Path::new(&full_file_name);
    let mut file = File::create(path)?;
    if let Err(_) = file.write_all(output.as_bytes()) {
        return Err(anyhow!("File to write to file: {}!", full_file_name));
    }

    let full_file_name = "./db/src/entities/mod.rs";
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(full_file_name)?;

    if let Err(_) = file.write_all(output.as_bytes()) {
        Err(anyhow!("File to write to file: {}!", full_file_name))
    } else {
        Ok(file_name)
    }
}
