use anyhow::{anyhow, Context};
use clap::{Parser, Subcommand};
use cruet::{
    case::{snake::to_snake_case, {%- if template_type != "minimal" -%}title::to_title_case{%- endif -%}},
{% if template_type != "minimal" -%}
    string::{pluralize::to_plural, singularize::to_singular},
{% endif -%}
};
use guppy::{graph::PackageGraph, MetadataCommand};
use liquid::Template;
use {{crate_name}}_cli::util::ui::UI;
use std::fs::{self, File, OpenOptions};
use std::io::prelude::*;
{% if template_type != "minimal" -%}
use std::time::SystemTime;
{% endif -%}

static BLUEPRINTS_DIR: include_dir::Dir =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/blueprints");

#[tokio::main]
async fn main() {
    cli().await;
}

#[derive(Parser)]
#[command(author, version, about = "A CLI tool to generate project files.", long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, global = true, help = "Disable colored output.")]
    no_color: bool,

    #[arg(long, global = true, help = "Disable debug output.")]
    quiet: bool,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Generate a middleware")]
    Middleware {
        #[arg(help = "The name of the middleware.")]
        name: String,
    },
    #[command(about = "Generate a controller")]
    Controller {
        #[arg(help = "The name of the controller.")]
        name: String,
    },
    #[command(about = "Generate a test for a controller")]
    ControllerTest {
        #[arg(help = "The name of the controller.")]
        name: String,
    },
    {% if template_type != "minimal" -%}
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
    #[command(about = "Generate an example CRUD controller")]
    CrudController {
        #[arg(help = "The name of the entity the controller is for.")]
        name: String,
    },
    #[command(about = "Generate a test for a CRUD controller")]
    CrudControllerTest {
        #[arg(help = "The name of the entity the controller is for.")]
        name: String,
    },
    {% endif -%}
}

#[allow(missing_docs)]
pub async fn cli() {
    let cli = Cli::parse();
    let mut stdout = std::io::stdout();
    let mut stderr = std::io::stderr();
    let mut ui = UI::new(&mut stdout, &mut stderr, !cli.no_color, !cli.quiet);

    match cli.command {
        Commands::Middleware { name } => {
            ui.info("Generating middleware…");
            match generate_middleware(name).await {
                Ok(file_name) => ui.success(&format!("Generated middleware {}.", &file_name)),
                Err(e) => ui.error("Could not generate middleware!", e),
            }
        }
        Commands::Controller { name } => {
            ui.info("Generating controller…");
            match generate_controller(name.clone()).await {
                Ok(file_name) => {
                    ui.success(&format!("Generated controller {}.", &file_name));
                    ui.info(
                        "Do not forget to route the controller's actions in ./web/src/routes.rs!",
                    );
                }
                Err(e) => ui.error("Could not generate controller!", e),
            }
            ui.info("Generating test for controller…");
            match generate_controller_test(name).await {
                Ok(file_name) => {
                    ui.success(&format!("Generated test for controller {}.", &file_name))
                }
                Err(e) => ui.error("Could not generate test for controller!", e),
            }
        }
        Commands::ControllerTest { name } => {
            ui.info("Generating test for controller…");
            match generate_controller_test(name).await {
                Ok(file_name) => {
                    ui.success(&format!("Generated test for controller {}.", &file_name))
                }
                Err(e) => ui.error("Could not generate test for controller!", e),
            }
        }
        {% if template_type != "minimal" -%}
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
            ui.info("Generating test for CRUD controller…");
            match generate_crud_controller_test(name).await {
                Ok(file_name) => ui.success(&format!(
                    "Generated test for CRUD controller {}.",
                    &file_name
                )),
                Err(e) => ui.error("Could not generate test for CRUD controller!", e),
            }
        }
        Commands::CrudControllerTest { name } => {
            ui.info("Generating test for CRUD controller…");
            match generate_crud_controller_test(name).await {
                Ok(file_name) => ui.success(&format!(
                    "Generated test for CRUD controller {}.",
                    &file_name
                )),
                Err(e) => ui.error("Could not generate test for CRUD controller!", e),
            }
        }
        {% endif -%}
    }
}

async fn generate_middleware(name: String) -> Result<String, anyhow::Error> {
    let name = to_snake_case(&name).to_lowercase();

    let template = get_liquid_template("middleware/file.rs")?;
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

async fn generate_controller(name: String) -> Result<String, anyhow::Error> {
    let name = to_snake_case(&name).to_lowercase();

    let template = get_liquid_template("controller/minimal/controller.rs")?;
    let variables = liquid::object!({
        "name": name,
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

async fn generate_controller_test(name: String) -> Result<String, anyhow::Error> {
    let name = to_snake_case(&name).to_lowercase();
    let macros_crate_name = get_member_package_name("macros")?;
    let macros_crate_name = to_snake_case(&macros_crate_name);
    let has_db = has_db();

    let template = get_liquid_template("controller/minimal/test.rs")?;
    let variables = liquid::object!({
        "name": name,
        "macros_crate_name": macros_crate_name,
        "has_db": has_db,
    });
    let output = template
        .render(&variables)
        .context("Failed to render Liquid template")?;

    let file_path = format!("./web/tests/api/{name}_test.rs");
    create_project_file(&file_path, output.as_bytes())?;
    append_to_project_file("./web/tests/api/main.rs", &format!("mod {name}_test;"))?;

    Ok(file_path)
}

{% if template_type != "minimal" -%}
async fn generate_migration(name: String) -> Result<String, anyhow::Error> {
    let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
    let file_name = format!("{}__{}.sql", timestamp.as_secs(), name);
    let path = format!("./db/migrations/{}", file_name);
    create_project_file(&path, "".as_bytes())?;

    Ok(path)
}

async fn generate_entity(name: String) -> Result<String, anyhow::Error> {
    let name = to_singular(&name).to_lowercase();
    let name_plural = to_plural(&name);
    let struct_name = to_title_case(&name);

    let template = get_liquid_template("entity/file.rs")?;
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

    let template = get_liquid_template("entity-test-helper/file.rs")?;
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

async fn generate_crud_controller(name: String) -> Result<String, anyhow::Error> {
    let name = to_snake_case(&name).to_lowercase();
    let name_plural = to_plural(&name);
    let name_singular = to_singular(&name);
    let struct_name = to_title_case(&name_singular);
    let db_crate_name = get_member_package_name("db")?;
    let db_crate_name = to_snake_case(&db_crate_name);
    let macros_crate_name = get_member_package_name("macros")?;
    let macros_crate_name = to_snake_case(&macros_crate_name);

    let template = get_liquid_template("controller/crud/controller.rs")?;
    let variables = liquid::object!({
        "entity_struct_name": struct_name,
        "entity_singular_name": name_singular,
        "entity_plural_name": name_plural,
        "db_crate_name": db_crate_name,
        "macros_crate_name": macros_crate_name
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
    let macros_crate_name = get_member_package_name("macros")?;
    let macros_crate_name = to_snake_case(&macros_crate_name);

    let template = get_liquid_template("controller/crud/test.rs")?;
    let variables = liquid::object!({
        "entity_struct_name": struct_name,
        "entity_singular_name": name_singular,
        "entity_plural_name": name_plural,
        "db_crate_name": db_crate_name,
        "macros_crate_name": macros_crate_name
    });
    let output = template
        .render(&variables)
        .context("Failed to render Liquid template")?;

    let file_path = format!("./web/tests/api/{name}_test.rs");
    create_project_file(&file_path, output.as_bytes())?;
    append_to_project_file("./web/tests/api/main.rs", &format!("mod {name}_test;"))?;

    Ok(file_path)
}
{% endif -%}

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
    let file_contents =
        fs::read_to_string(path).context(format!(r#"Could not read file "{}"!"#, path))?;
    let file_contents = file_contents.trim();

    let mut options = OpenOptions::new();
    options.write(true);

    if file_contents.is_empty() {
        options.truncate(true);
    } else {
        options.append(true);
    }

    let mut file = options
        .open(path)
        .context(format!(r#"Could not open file "{}"!"#, path))?;

    writeln!(file, "{}", contents).context(format!(r#"Failed to append to file "{}"!"#, path))?;

    Ok(())
}

fn has_db() -> bool {
    get_member_package_name("db").is_ok()
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
