use std::env::current_dir;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Context;
use clap::{Parser, Subcommand};

use autograde_rs::build::make;
use autograde_rs::config::Config;
use autograde_rs::unit::Units;

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Command,
    #[arg(short, long)]
    project_path: Option<String>,
    #[arg(short, long)]
    tests_path: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Command {
    Test,
    // Configure,
}

#[tokio::main]
async fn main() -> miette::Result<()> {
    let config = Config::read_or_create().unwrap();

    let args = Cli::parse();

    let pwd = current_dir().unwrap();
    let project = pwd.file_name().unwrap();

    match args.command {
        Command::Test => {
            let config_test = config
                .test
                .context("Config file missing test section!")
                .unwrap();

            let tests_path = &config_test
                .tests_path
                .context("Could not find test_path in config file!")
                .unwrap();

            // make().await?;

            // TODO support tilde expansion
            // TODO search pwd/parents for tests dir
            let mut tests_path = PathBuf::from_str(&tests_path)
                .with_context(|| format!("Invalid path! {}", tests_path))
                .unwrap();
            tests_path.push(project);
            tests_path.push(project);
            tests_path.set_extension("toml");

            // TODO move to tests.rs
            let tests_file = read_to_string(&tests_path).unwrap();
            // let tests_file = match read_to_string(&tests_path) {
            //     Ok(f)
            // }
            let tests: Units = toml::from_str(&tests_file)
                .with_context(|| format!("Could not parse tests at {}!", tests_path.display()))
                .unwrap();

            // TODO auto pull
            let grade = tests.run().await?;
            println!("{}", grade);
        }
    }

    Ok(())
}
