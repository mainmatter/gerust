use cargo_generate::{GenerateArgs, TemplatePath};
use clap::{arg, command, value_parser, Command};
use std::env::current_dir;
use std::path::PathBuf;

fn cli() -> Command {
    command!()
        .arg(arg!(name: <NAME>).value_parser(value_parser!(String)))
        .arg(arg!(outdir: -o <OUTDIR>).value_parser(value_parser!(String)))
}

fn main() {
    let matches = cli().get_matches();
    let name = matches
        .get_one::<String>("name")
        .map(|s| s.as_str())
        .unwrap();
    let output_dir = matches
        .get_one::<String>("outdir")
        .map(|s| s.as_str())
        .unwrap_or(name);

    generate(name, output_dir);
}

fn generate(name: &str, output_dir: &str) {
    let current_dir = current_dir().unwrap();
    let output_dir = PathBuf::from(current_dir.join(output_dir).parent().unwrap());

    let generate_args = GenerateArgs {
        template_path: TemplatePath {
            git: Some("https://github.com/marcoow/pacesetter".into()),
            subfolder: Some("template".into()),
            branch: Some("cli".into()),
            revision: Some(env!("VERGEN_GIT_SHA").into()),
            ..Default::default()
        },
        destination: Some(output_dir),
        name: Some(String::from(name)),
        force_git_init: true,
        ..Default::default()
    };
    cargo_generate::generate(generate_args).unwrap();
}
