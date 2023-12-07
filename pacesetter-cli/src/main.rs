use anyhow::anyhow;
use cargo_generate::{GenerateArgs, TemplatePath};
use clap::Parser;
use owo_colors::OwoColorize;
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

    info("Generating", "pacesetter project…");

    match generate(&cli.name, cli.outdir) {
        Ok(output_dir) => success(
            "Generated",
            format!("{} at {}.", cli.name, output_dir.display()).as_str(),
        ),
        Err(e) => error("Failed", format!("to generate project: {:?}!", e).as_str()),
    }
}

fn generate(
    name: &str,
    output_dir: Option<PathBuf>,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let current_dir = current_dir()?;
    let output_dir = if let Some(output_dir) = output_dir {
        current_dir.join(output_dir)
    } else {
        current_dir.join(name)
    };
    let output_dir = output_dir
        .parent()
        .ok_or_else(|| anyhow!("Cannot get output directory"))?;
    let output_dir = PathBuf::from(output_dir);

    let generate_args = GenerateArgs {
        template_path: TemplatePath {
            git: Some("https://github.com/marcoow/pacesetter".into()),
            subfolder: Some("template".into()),
            branch: Some("cli".into()),
            revision: Some(env!("VERGEN_GIT_SHA").into()),
            ..Default::default()
        },
        destination: Some(output_dir.clone()),
        name: Some(String::from(name)),
        force_git_init: true,
        ..Default::default()
    };

    let output_dir = cargo_generate::generate(generate_args)?;

    Ok(output_dir)
}

fn info(title: &str, text: &str) {
    println!("ℹ️  {} {}", pad_title(title).bright_blue(), text);
}

fn success(title: &str, text: &str) {
    println!("✅ {} {}", pad_title(title).green(), text);
}

fn error(title: &str, text: &str) {
    eprintln!("❌ {} {}", pad_title(title).red(), text);
}

fn pad_title(text: &str) -> String {
    format!("{: >10}", text)
}
