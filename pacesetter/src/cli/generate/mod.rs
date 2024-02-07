use anyhow::{anyhow, Context};
use clap::{Parser, Subcommand};
use cruet::{
    case::{snake::to_snake_case, title::to_title_case},
    string::{pluralize::to_plural, singularize::to_singular},
};
use guppy::{graph::PackageGraph, MetadataCommand};
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
    #[command(about = "Generate an entity test helper")]
    EntityTestHelper {
        #[arg(help = "The name of the entity the test helper is for.")]
        name: String,
    },
    #[command(about = "Generate a middleware")]
    Middleware {
        #[arg(help = "The name of the middleware.")]
        name: String,
    },
    #[command(about = "Generate an example CRUD controller")]
    CrudController {
        #[arg(help = "The name of the entity the controller is for.")]
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
        Commands::EntityTestHelper { name } => {
            ui.info("Generating entity test helper…");
            match generate_entity_test_helper(name).await {
                Ok(struct_name) => ui.success(&format!(
                    "Generated test helper for entity {}.",
                    &struct_name
                )),
                Err(e) => ui.error("Could not generate entity test helper!", e),
            }
        }
        Commands::Middleware { name } => {
            ui.info("Generating middleware…");
            match generate_middleware(name).await {
                Ok(file_name) => ui.success(&format!("Generated middleware {}.", &file_name)),
                Err(e) => ui.error("Could not generate middleware!", e),
            }
        }
        Commands::CrudController { name } => {
            ui.info("Generating CRUD controller…");
            match generate_crud_controller(name.clone()).await {
                Ok(file_name) => {
                    ui.success(&format!("Generated CRUD controller {}.", &file_name));
                    ui.info(
                        "Do not forget to route the controller's actions in ./web/src/routes.rs!",
                    );
                }
                Err(e) => ui.error("Could not generate CRUD controller!", e),
            }
            match generate_crud_controller_test(name).await {
                Ok(file_name) => ui.success(&format!(
                    "Generated test for CRUD controller {}.",
                    &file_name
                )),
                Err(e) => ui.error("Could not generate test for CRUD controller!", e),
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

async fn generate_entity_test_helper(name: String) -> Result<String, anyhow::Error> {
    let name = to_singular(&name).to_lowercase();
    let name_plural = to_plural(&name);
    let struct_name = to_title_case(&name);

    let template = get_liquid_template("entity-test-helper/file.rs.liquid")?;
    let variables = liquid::object!({
        "entity_struct_name": struct_name,
        "entity_singular_name": name,
        "entity_plural_name": name_plural,
    });
    let output = template
        .render(&variables)
        .context("Failed to render Liquid template")?;

    create_project_file(
        &format!("./db/src/test_helpers/{}.rs", name_plural),
        output.as_bytes(),
    )?;
    append_to_project_file(
        "./db/src/test_helpers/mod.rs",
        &format!("pub mod {};", name_plural),
    )?;

    Ok(struct_name)
}

async fn generate_middleware(name: String) -> Result<String, anyhow::Error> {
    let name = to_snake_case(&name).to_lowercase();

    let template = get_liquid_template("middleware/file.rs.liquid")?;
    let variables = liquid::object!({
        "name": name
    });
    let output = template
        .render(&variables)
        .context("Failed to render Liquid template")?;

    let file_path = format!("./web/src/middlewares/{}.rs", name);
    create_project_file(&file_path, output.as_bytes())?;
    append_to_project_file(
        "./web/src/middlewares/mod.rs",
        &format!("pub mod {};", name),
    )?;

    Ok(file_path)
}

async fn generate_crud_controller(name: String) -> Result<String, anyhow::Error> {
    let name = to_snake_case(&name).to_lowercase();
    let name_plural = to_plural(&name);
    let name_singular = to_singular(&name);
    let struct_name = to_title_case(&name_singular);
    let db_crate_name = get_member_package_name("db")?;
    let db_crate_name = to_snake_case(&db_crate_name);

    let template = get_liquid_template("controller/crud/controller.rs.liquid")?;
    let variables = liquid::object!({
        "entity_struct_name": struct_name,
        "entity_singular_name": name_singular,
        "entity_plural_name": name_plural,
        "db_crate_name": db_crate_name
    });
    let output = template
        .render(&variables)
        .context("Failed to render Liquid template")?;

    let file_path = format!("./web/src/controllers/{}.rs", name);
    create_project_file(&file_path, output.as_bytes())?;
    append_to_project_file(
        "./web/src/controllers/mod.rs",
        &format!("pub mod {};", name),
    )?;

    Ok(file_path)
}

async fn generate_crud_controller_test(name: String) -> Result<String, anyhow::Error> {
    let name = to_snake_case(&name).to_lowercase();
    let name_plural = to_plural(&name);
    let name_singular = to_singular(&name);
    let struct_name = to_title_case(&name_singular);
    let db_crate_name = get_member_package_name("db")?;
    let db_crate_name = to_snake_case(&db_crate_name);

    let template = get_liquid_template("controller/crud/test.rs.liquid")?;
    let variables = liquid::object!({
        "entity_struct_name": struct_name,
        "entity_singular_name": name_singular,
        "entity_plural_name": name_plural,
        "db_crate_name": db_crate_name
    });
    let output = template
        .render(&variables)
        .context("Failed to render Liquid template")?;

    let file_path = format!("./web/tests/{}_test.rs", name);
    create_project_file(&file_path, output.as_bytes())?;

    Ok(file_path)
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

fn get_member_package_name(path: &str) -> Result<String, anyhow::Error> {
    let mut cmd = MetadataCommand::new();
    let package_graph = PackageGraph::from_command(cmd.manifest_path("./Cargo.toml")).unwrap();
    let workspace = package_graph.workspace();
    for member in workspace.iter_by_path() {
        let (member_path, metadata) = member;
        if member_path == path {
            return Ok(String::from(metadata.name()));
        }
    }
    Err(anyhow!("Could not find workspace member at path: {}", path))
}
