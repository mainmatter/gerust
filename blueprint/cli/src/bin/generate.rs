use anyhow::{anyhow, Context};
use clap::{Parser, Subcommand};
use cruet::{
    case::{snake::to_snake_case, {%- if template_type != "minimal" -%}to_class_case{%- endif -%}},
{% if template_type != "minimal" -%}
    string::{pluralize::to_plural, singularize::to_singular},
{% endif -%}
};
use guppy::{graph::PackageGraph, MetadataCommand};
use liquid::Template;
use {{crate_name}}_cli::util::ui::UI;
{% if template_type != "minimal" -%}
use regex::Regex;
use std::collections::HashMap;
{% endif -%}
use std::fs::{self, File, OpenOptions};
use std::io::prelude::*;
use std::path::Path;
use std::process::ExitCode;
{% if template_type != "minimal" -%}
use std::time::SystemTime;
{% endif -%}

static BLUEPRINTS_DIR: include_dir::Dir =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/blueprints");

#[tokio::main]
async fn main() -> ExitCode {
    let mut stdout = std::io::stdout();
    let mut stderr = std::io::stderr();

    let args = Cli::parse();
    let mut ui = UI::new(&mut stdout, &mut stderr, !args.no_color, !args.quiet);

    match cli(&mut ui, args).await {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            ui.error(e.to_string().as_str(), &e);
            ExitCode::FAILURE
        }
    }
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

    #[arg(long, global = false, help = "Override existing files.")]
    r#override: bool,
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
        #[arg(
            help = "The fields of the entity, each given as '<name>:<Rust type>'. Supported types are bool, i8, i16, i32, i64, f32, f64, String"
        )]
        fields: Vec<String>,
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
async fn cli(ui: &mut UI<'_>, cli: Cli) -> Result<(), anyhow::Error> {
    match cli.command {
        Commands::Middleware { name } => {
            ui.info("Generating middleware…");
            let file_name = generate_middleware(name, cli.r#override)
                .await
                .context("Could not generate middleware!")?;
            ui.success(&format!("Generated middleware {}.", &file_name));
            Ok(())
        }
        Commands::Controller { name } => {
            ui.info("Generating controller…");
            let file_name = generate_controller(name.clone(), cli.r#override)
                .await
                .context("Could not generate controller!")?;
            ui.success(&format!("Generated controller {}.", &file_name));
            ui.info("Do not forget to route the controller's actions in ./web/src/routes.rs!");
            ui.info("Generating test for controller…");
            let file_name = generate_controller_test(name, cli.r#override)
                .await
                .context("Could not generate test for controller!")?;
            ui.success(&format!("Generated test for controller {}.", &file_name));
            Ok(())
        }
        Commands::ControllerTest { name } => {
            ui.info("Generating test for controller…");
            let file_name = generate_controller_test(name, cli.r#override)
                .await
                .context("Could not generate test for controller!")?;
            ui.success(&format!("Generated test for controller {}.", &file_name));
            Ok(())
        }
        {% if template_type != "minimal" -%}
        Commands::Migration { name } => {
            ui.info("Generating migration…");
            let file_name = generate_migration(name, cli.r#override)
                .await
                .context("Could not generate migration!")?;
            ui.success(&format!("Generated migration {}.", &file_name));
            Ok(())
        }
        Commands::Entity { name, fields } => {
            ui.info("Generating entity…");
            let struct_name = generate_entity(name, fields, cli.r#override)
                .await
                .context("Could not generate entity!")?;
            ui.success(&format!("Generated entity {}.", &struct_name));
            Ok(())
        }
        Commands::EntityTestHelper { name } => {
            ui.info("Generating entity test helper…");
            let struct_name = generate_entity_test_helper(name, cli.r#override)
                .await
                .context("Could not generate entity test helper!")?;
            ui.success(&format!(
                "Generated test helper for entity {}.",
                &struct_name
            ));
            Ok(())
        }
        Commands::CrudController { name } => {
            ui.info("Generating CRUD controller…");
            let file_name = generate_crud_controller(name.clone(), cli.r#override)
                .await
                .context("Could not generate CRUD controller!")?;
            ui.success(&format!("Generated CRUD controller {}.", &file_name));
            ui.info("Do not forget to route the controller's actions in ./web/src/routes.rs!");
            let file_name = generate_crud_controller_test(name.clone(), cli.r#override)
                .await
                .context("Could not generate test for CRUD controller!")?;
            ui.success(&format!(
                "Generated test for CRUD controller {}.",
                &file_name
            ));
            Ok(())
        }
        Commands::CrudControllerTest { name } => {
            ui.info("Generating test for CRUD controller…");
            let file_name = generate_crud_controller_test(name.clone(), cli.r#override)
                .await
                .context("Could not generate test for CRUD controller!")?;
            ui.success(&format!(
                "Generated test for CRUD controller {}.",
                &file_name
            ));
            Ok(())
        }
        {% endif -%}
    }
}

async fn generate_middleware(name: String, r#override: bool) -> Result<String, anyhow::Error> {
    let name = to_snake_case(&name).to_lowercase();

    let template = get_liquid_template("middleware/file.rs")?;
    let variables = liquid::object!({
        "name": name
    });
    let output = template
        .render(&variables)
        .context("Failed to render Liquid template")?;

    let file_path = format!("./web/src/middlewares/{}.rs", name);
    create_project_file(&file_path, output.as_bytes(), r#override)?;
    append_to_project_file(
        "./web/src/middlewares/mod.rs",
        &format!("pub mod {};", name),
    )?;

    Ok(file_path)
}

async fn generate_controller(name: String, r#override: bool) -> Result<String, anyhow::Error> {
    let name = to_snake_case(&name).to_lowercase();

    let template = get_liquid_template("controller/minimal/controller.rs")?;
    let variables = liquid::object!({
        "name": name,
    });
    let output = template
        .render(&variables)
        .context("Failed to render Liquid template")?;

    let file_path = format!("./web/src/controllers/{}.rs", name);
    create_project_file(&file_path, output.as_bytes(), r#override)?;
    append_to_project_file(
        "./web/src/controllers/mod.rs",
        &format!("pub mod {};", name),
    )?;

    Ok(file_path)
}

async fn generate_controller_test(name: String, r#override: bool) -> Result<String, anyhow::Error> {
    let name = to_snake_case(&name).to_lowercase();
    let macros_crate_name = get_member_package_name("macros")?;
    let macros_crate_name = to_snake_case(&macros_crate_name);
    let web_crate_name = get_member_package_name("web")?;
    let web_crate_name = to_snake_case(&web_crate_name);
    let has_db = has_db();

    let template = get_liquid_template("controller/minimal/test.rs")?;
    let variables = liquid::object!({
        "name": name,
        "macros_crate_name": macros_crate_name,
        "web_crate_name": web_crate_name,
        "has_db": has_db,
    });
    let output = template
        .render(&variables)
        .context("Failed to render Liquid template")?;

    let file_path = format!("./web/tests/api/{name}_test.rs");
    create_project_file(&file_path, output.as_bytes(), r#override)?;
    append_to_project_file("./web/tests/api/main.rs", &format!("mod {name}_test;"))?;

    Ok(file_path)
}

{% if template_type != "minimal" -%}
async fn generate_migration(name: String, r#override: bool) -> Result<String, anyhow::Error> {
    let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
    let file_name = format!("{}__{}.sql", timestamp.as_secs(), name);
    let path = format!("./db/migrations/{}", file_name);
    create_project_file(&path, "".as_bytes(), r#override)?;

    Ok(path)
}

async fn generate_entity(name: String, fields: Vec<String>, r#override: bool) -> Result<String, anyhow::Error> {
    let fields = validate_fields(&fields)?;
    let name = to_singular(&name).to_lowercase();
    let name_plural = to_plural(&name);
    let struct_name = to_class_case(&name);

    let template = get_liquid_template("entity/file.rs")?;
    let variables = liquid::object!({
        "entity_struct_name": struct_name,
        "entity_singular_name": name,
        "entity_plural_name": name_plural,
        "fields": fields,
    });
    let output = template
        .render(&variables)
        .context("Failed to render Liquid template")?;

    create_project_file(
        &format!("./db/src/entities/{}.rs", name_plural),
        output.as_bytes(),
        r#override,
    )?;
    append_to_project_file(
        "./db/src/entities/mod.rs",
        &format!("pub mod {};", name_plural),
    )?;

    Ok(struct_name)
}

async fn generate_entity_test_helper(name: String, r#override: bool) -> Result<String, anyhow::Error> {
    let name = to_singular(&name).to_lowercase();
    let name_plural = to_plural(&name);
    let struct_name = to_class_case(&name);

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
        r#override,
    )?;
    append_to_project_file(
        "./db/src/test_helpers/mod.rs",
        &format!("pub mod {};", name_plural),
    )?;

    Ok(struct_name)
}

async fn generate_crud_controller(name: String, r#override: bool) -> Result<String, anyhow::Error> {
    let name = to_snake_case(&name).to_lowercase();
    let name_plural = to_plural(&name);
    let name_singular = to_singular(&name);
    let struct_name = to_class_case(&name_singular);
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
    create_project_file(&file_path, output.as_bytes(), r#override)?;
    append_to_project_file(
        "./web/src/controllers/mod.rs",
        &format!("pub mod {};", name),
    )?;

    Ok(file_path)
}

async fn generate_crud_controller_test(name: String, r#override: bool) -> Result<String, anyhow::Error> {
    let name = to_snake_case(&name).to_lowercase();
    let name_plural = to_plural(&name);
    let name_singular = to_singular(&name);
    let struct_name = to_class_case(&name_singular);
    let db_crate_name = get_member_package_name("db")?;
    let db_crate_name = to_snake_case(&db_crate_name);
    let macros_crate_name = get_member_package_name("macros")?;
    let macros_crate_name = to_snake_case(&macros_crate_name);
    let web_crate_name = get_member_package_name("web")?;
    let web_crate_name = to_snake_case(&web_crate_name);

    let template = get_liquid_template("controller/crud/test.rs")?;
    let variables = liquid::object!({
        "entity_struct_name": struct_name,
        "entity_singular_name": name_singular,
        "entity_plural_name": name_plural,
        "db_crate_name": db_crate_name,
        "macros_crate_name": macros_crate_name,
        "web_crate_name": web_crate_name,
    });
    let output = template
        .render(&variables)
        .context("Failed to render Liquid template")?;

    let file_path = format!("./web/tests/api/{name}_test.rs");
    create_project_file(&file_path, output.as_bytes(), r#override)?;
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

fn create_project_file(path: &str, contents: &[u8], r#override: bool) -> Result<(), anyhow::Error> {
    if !r#override && Path::new(path).exists() {
        Err(anyhow!("File {} already exists!", path))
    } else {
        let mut file = File::create(path).context(format!(r#"Could not create file "{}""#, path))?;
        file.write_all(contents)
            .context(format!(r#"Could not write file "{}""#, path))?;
        
        Ok(())
    }
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

{% if template_type != "minimal" -%}
fn validate_fields(fields: &Vec<String>) -> Result<Vec<HashMap<String, String>>, anyhow::Error> {
    let re =
        Regex::new(r"^([a-zA-Z][a-zA-Z0-9_]+)\:(bool|Bool|i8|i16|i32|i64|f32|f64|String|string)$")
            .unwrap();
    let mut mapped_fields = Vec::<HashMap<String, String>>::new();
    for field in fields {
        let Some(captures) = re.captures(field.trim()) else {
            return Err(anyhow!("Invalid field definition: {}!", field));
        };
        if captures.len() != 3 {
            return Err(anyhow!("Invalid field definition: {}!", field));
        }

        let field_name = String::from(captures.get(1).unwrap().as_str());
        let field_type = captures
            .get(2)
            .unwrap()
            .as_str()
            .replace("Bool", "bool")
            .replace("string", "String");

        let mut field = HashMap::new();
        field.insert("name".to_string(), field_name);
        field.insert("type".to_string(), field_type);
        mapped_fields.push(field);
    }

    Ok(mapped_fields)
}
{% endif -%}