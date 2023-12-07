use anyhow::Context;
use cargo_generate::{GenerateArgs, TemplatePath};
use clap::Parser;
use std::env::current_dir;
use std::path::PathBuf;

static VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), " (", env!("VERGEN_GIT_SHA"), ")");

#[derive(Parser)]
#[clap(author, version = VERSION, about, long_about = None)]
struct Cli {
    #[arg(index = 1)]
    name: String,
    #[arg(short, long, value_parser)]
    outdir: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    info(format!("Generating {}…", cli.name).as_str());

    match generate(&cli.name, cli.outdir) {
        Ok(output_dir) => {
            success(format!("Generated {} at {}", cli.name, output_dir.display()).as_str())
        }
        Err(e) => error(format!("Error: {:?}", e).as_str()),
    }
}

fn generate(name: &str, output_dir: Option<PathBuf>) -> Result<PathBuf, anyhow::Error> {
    let current_dir = current_dir()?;
    let output_dir = if let Some(output_dir) = output_dir {
        output_dir
    } else {
        current_dir
    };

    let is_pacesetter_cli_dev = if let Some(env) = option_env!("PACESETTER_CLI_DEV") {
        env == "1"
    } else {
        false
    };
    let template_path = if is_pacesetter_cli_dev {
        TemplatePath {
            path: Some("./template".into()),
            ..Default::default()
        }
    } else {
        TemplatePath {
            git: Some("https://github.com/marcoow/pacesetter".into()),
            subfolder: Some("template".into()),
            revision: Some(env!("VERGEN_GIT_SHA").into()),
            ..Default::default()
        }
    };

    let generate_args = GenerateArgs {
        template_path,
        destination: Some(output_dir.clone()),
        name: Some(String::from(name)),
        force_git_init: true,
        ..Default::default()
    };

    let output_dir = cargo_generate::generate(generate_args)
        .context("failed to generate project from template")?;

    Ok(output_dir)
}

fn info(text: &str) {
    println!("ℹ️  {}", text);
}

fn success(text: &str) {
    println!("✅ {}", text);
}

fn error(text: &str) {
    eprintln!("❌ {}", text);
}
