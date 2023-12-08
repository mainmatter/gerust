use anyhow::Context;
use cargo_generate::{GenerateArgs, TemplatePath};
use clap::Parser;
use std::env;
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

    let is_local = env::var("PS_CLI_LOCAL_DEV").is_ok();

    info(format!("Generating {}…", cli.name).as_str());

    match generate(&cli.name, cli.outdir, is_local) {
        Ok(output_dir) => {
            success(format!("Generated {} at {}", cli.name, output_dir.display()).as_str())
        }
        Err(e) => error(format!("Error: {:?}", e).as_str()),
    }
}

fn generate(
    name: &str,
    output_dir: Option<PathBuf>,
    is_local: bool,
) -> Result<PathBuf, anyhow::Error> {
    if is_local {
        info("Using local template ./template");
    }

    let output_dir = if let Some(output_dir) = output_dir {
        output_dir
    } else {
        env::current_dir()?
    };

    let template_path = build_template_path(is_local);

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

fn build_template_path(is_local: bool) -> TemplatePath {
    if is_local {
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
    }
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
