use anyhow::Context;
use cargo_generate::{GenerateArgs, TemplatePath};
use clap::{ArgAction, Parser};
use pacesetter_util::ui::UI;
use std::env;
use std::fs;
use std::path::PathBuf;

static VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), " (", env!("VERGEN_GIT_SHA"), ")");

static BLUEPRINTS_DIR: include_dir::Dir =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/../blueprints");

enum Blueprint {
    Minimal,
    Default,
    Full,
}

#[derive(Parser)]
#[clap(author, version = VERSION, about, long_about = None)]
struct Cli {
    #[arg(index = 1)]
    name: String,
    #[arg(short, long, value_parser)]
    outdir: Option<PathBuf>,
    #[arg(short, long, action(ArgAction::SetTrue))]
    full: bool,
    #[arg(short, long, action(ArgAction::SetTrue))]
    minimal: bool,

    #[arg(long, global = true, help = "Disable colored output.")]
    no_color: bool,

    #[arg(long, global = true, help = "Enable debug output.")]
    debug: bool,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let mut ui = UI::new(!cli.no_color, cli.debug);

    let is_local = env::var("PS_CLI_LOCAL_DEV").is_ok();

    let blueprint = if cli.full {
        Blueprint::Full
    } else if cli.minimal {
        Blueprint::Minimal
    } else {
        Blueprint::Default
    };

    ui.info(&format!("Generating {}…", cli.name));
    ui.indent();

    match generate(&cli.name, cli.outdir, is_local, blueprint, &ui).await {
        Ok(output_dir) => {
            ui.outdent();
            ui.success(&format!(
                "Generated {} at {}.",
                cli.name,
                output_dir.display()
            ));
        }
        Err(e) => {
            ui.outdent();
            ui.error("Could not generate project!", e);
        }
    }
}

async fn generate(
    name: &str,
    output_dir: Option<PathBuf>,
    is_local: bool,
    blueprint: Blueprint,
    ui: &UI,
) -> Result<PathBuf, anyhow::Error> {
    if is_local {
        ui.log("Using local template ./template");
        ui.log("Using local pacesetter ./pacesetter");
        ui.log("Using local pacesetter-procs ./pacesetter-procs");
    }

    let output_dir = if let Some(output_dir) = output_dir {
        output_dir
    } else {
        env::current_dir()?
    };

    let mut defines: Vec<String> = vec![];
    if is_local {
        defines.push(format!(
            "use_local_pacesetter={}",
            get_local_pacesetter_path("pacesetter")?
        ));
        defines.push(format!(
            "use_local_pacesetter_procs={}",
            get_local_pacesetter_path("pacesetter-procs")?
        ));
    }

    let template_path = build_template_path(blueprint).await?;

    let generate_args = GenerateArgs {
        template_path,
        destination: Some(output_dir.clone()),
        name: Some(String::from(name)),
        force_git_init: true,
        define: defines,
        ..Default::default()
    };

    let output_dir = cargo_generate::generate(generate_args)
        .context("failed to generate project from template")?;

    Ok(output_dir)
}

async fn build_template_path(blueprint: Blueprint) -> Result<TemplatePath, anyhow::Error> {
    let blueprint_folder = match blueprint {
        Blueprint::Full => "full",
        Blueprint::Default => "default",
        Blueprint::Minimal => "minimal",
    };

    let target_directory =
        std::env::temp_dir().join(format!("pacesetter-cli-blueprint-{}", VERSION));
    std::fs::create_dir_all(&target_directory)
        .context("Failed to create a temporary directory for Pacesetter CLI's blueprints")?;
    BLUEPRINTS_DIR
        .extract(&target_directory)
        .context("Failed to extract Pacesetter CLI's blueprints to a temporary directory")?;
    let bluprint_path = target_directory.join(blueprint_folder);
    let bluprint_path = bluprint_path
        .to_str()
        .unwrap_or("Failed to get full path to Pacesetter CLI's blueprint");

    Ok(TemplatePath {
        path: Some(String::from(bluprint_path)),
        ..Default::default()
    })
}

fn get_local_pacesetter_path(lib: &str) -> Result<String, anyhow::Error> {
    let current_dir = env::current_dir()?;
    let local_pacesetter = current_dir.join(lib);
    let local_pacesetter = fs::canonicalize(local_pacesetter)?;
    let local_pacesetter = local_pacesetter.as_path().display().to_string();
    Ok(local_pacesetter)
}
