#![deny(missing_docs)]

//! Pacesetter provides blueprints and generators for axum projects. It establishes a standard project structure with a folder layout, standard patterns for composing applications into e.g. database access and web API, running tests and migrations, as well as tracing.

use anyhow::Context;
use cargo_generate::{GenerateArgs, TemplatePath};
use clap::{ArgAction, Parser};
use std::env;
use std::fs;
use std::path::PathBuf;

#[allow(dead_code)]
mod ui;

#[doc(hidden)]
static VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), " (", env!("VERGEN_GIT_SHA"), ")");

#[doc(hidden)]
static BLUEPRINTS_DIR: include_dir::Dir =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/blueprint");

#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
enum Blueprint {
    Minimal,
    Default,
    Full,
}

impl std::fmt::Display for Blueprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Blueprint::Minimal => write!(f, "minimal"),
            Blueprint::Default => write!(f, "default"),
            Blueprint::Full => write!(f, "full"),
        }
    }
}

#[doc(hidden)]
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

#[doc(hidden)]
#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let mut stdout = std::io::stdout();
    let mut stderr = std::io::stderr();
    let mut ui = ui::UI::new(&mut stdout, &mut stderr, !cli.no_color, cli.debug);

    let blueprint = if cli.full {
        Blueprint::Full
    } else if cli.minimal {
        Blueprint::Minimal
    } else {
        Blueprint::Default
    };

    ui.info(&format!("Generating {}…", cli.name));
    ui.indent();

    match generate(&cli.name, cli.outdir, blueprint).await {
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

#[doc(hidden)]
async fn generate(
    name: &str,
    output_dir: Option<PathBuf>,
    blueprint: Blueprint,
) -> Result<PathBuf, anyhow::Error> {
    let output_dir = if let Some(output_dir) = output_dir {
        output_dir
    } else {
        env::current_dir()?
    };

    let mut defines: Vec<String> = vec![];
    defines.push(format!("template_type={blueprint}"));

    let template_path = build_template_path().await?;

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

    // Try to format the generated code.
    let _ = std::process::Command::new("cargo")
        .arg("fmt")
        .current_dir(&output_dir)
        .status();

    Ok(output_dir)
}

#[doc(hidden)]
async fn build_template_path() -> Result<TemplatePath, anyhow::Error> {
    let target_directory = env::temp_dir().join(format!("pacesetter-blueprint-{}", VERSION));
    fs::create_dir_all(&target_directory)
        .context("Failed to create a temporary directory for Pacesetter CLI's blueprints")?;
    BLUEPRINTS_DIR
        .extract(&target_directory)
        .context("Failed to extract Pacesetter CLI's blueprints to a temporary directory")?;
    let bluprint_path = target_directory
        .to_str()
        .unwrap_or("Failed to get full path to Pacesetter CLI's blueprint");

    Ok(TemplatePath {
        path: Some(String::from(bluprint_path)),
        ..Default::default()
    })
}
