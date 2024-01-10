use anyhow::Context;
use clap::{Parser, Subcommand};
use cruet::{
    string::{pluralize::to_plural, singularize::to_singular},
    to_title_case,
};
use liquid::Template;
use pacesetter_util::ui::UI;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::time::SystemTime;

static _VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), " (", env!("VERGEN_GIT_SHA"), ")");

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
                Ok(struct_name) => ui.success(&format!("Generated entity {}.", &struct_name)),
                Err(e) => ui.error("Could not generate entity!", e),
            }
        }
    }
}

async fn generate_migration(name: String) -> Result<String, anyhow::Error> {
    let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
    let file_name = format!("V{}__{}.sql", timestamp.as_secs(), name);
    let path = format!("./db/migrations/{}", file_name);
    create_project_file(&path, "".as_bytes())?;

    Ok(path)
}

async fn generate_entity(name: String) -> Result<String, anyhow::Error> {
    let name = to_singular(&name).to_lowercase();
    let name_plural = to_plural(&name);
    let struct_name = to_title_case(&name);

    let template = get_liquid_template("entity/file.rs.liquid")?;
    let variables = liquid::object!({
        "entity_struct_name": struct_name,
        "entity_singular_name": name,
        "entity_plural_name": name_plural,
    });
    let output = template
        .render(&variables)
        .context("Failed to render Liquid template")?;

    create_project_file(
        &format!("./db/src/entities/{}.rs", name_plural),
        output.as_bytes(),
    )?;
    append_to_project_file(
        "./db/src/entities/mod.rs",
        &format!("pub mod {};", name_plural),
    )?;

    Ok(struct_name)
}

fn get_liquid_template(path: &str) -> Result<Template, anyhow::Error> {
    let blueprint = BLUEPRINTS_DIR
        .get_file(path)
        .context(format!("Failed to get blueprint {}!", path))?;
    let template_source = blueprint
        .contents_utf8()
        .context(format!("Failed to read blueprint {}!", path))?;
    let template = liquid::ParserBuilder::with_stdlib()
        .build()
        .unwrap()
        .parse(template_source)
        .context("Failed to parse blueprint as Liquid template")?;

    Ok(template)
}

fn create_project_file(path: &str, contents: &[u8]) -> Result<(), anyhow::Error> {
    let mut file = File::create(path).context(format!(r#"Could not create file "{}""#, path))?;
    file.write_all(contents)
        .context(format!(r#"Could not write file "{}""#, path))?;

    Ok(())
}

fn append_to_project_file(path: &str, contents: &str) -> Result<(), anyhow::Error> {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(path)
        .context(format!(r#"Could not open file "{}"!"#, path))?;

    writeln!(file, "{}", contents).context(format!(r#"Failed to append to file "{}"!"#, path))?;

    Ok(())
}
