#![allow(unused)]

use std::env::current_dir;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Context;
use clap::{Parser, Subcommand};
use log::info;
use miette::WrapErr;

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
    env_logger::init();

    let config = Config::read_or_create().unwrap();

    let args = Cli::parse();

    let pwd = current_dir().unwrap();
    let project = pwd.file_name().unwrap();
    let project = project
        .to_str()
        .map(|s| s.split('-').next().unwrap_or(s))
        .unwrap_or_else(|| project.to_str().unwrap());
    info!("project executable name: {}", project);

    match args.command {
        Command::Test => {
            let config_test = config
                .test
                .clone()
                .context("Config file missing test section!")
                .unwrap();

            let tests_path = &config_test
                .tests_path
                .clone()
                .context("Could not find test_path in config file!")
                .unwrap();

            let digital_path = config_test.digital_path();
            info!("Digital JAR path: {:?}", digital_path); // TODO: use log crate

            // make().await?;

            // TODO support tilde expansion
            // TODO search pwd/parents for tests dir
            let mut tests_path = PathBuf::from_str(tests_path)
                .with_context(|| format!("Invalid path! {}", tests_path))
                .unwrap();
            tests_path.push(project);
            tests_path.push(project);
            tests_path.set_extension("toml");
            info!("test path: {:?}", tests_path);

            // TODO move to tests.rs
            let tests_file = read_to_string(&tests_path).unwrap();
            let mut tests: Units = toml::from_str(&tests_file)
                .with_context(|| format!("Could not parse tests at {}!", tests_path.display()))
                .unwrap();

            tests.tests.iter_mut().for_each(|test| {
                test.interp_input(&config, project);
            });
            info!("test unit struct: {:?}", tests);

            // TODO auto pull
            let grade = tests.run().await?;
            info!("grade: {}", grade);
        }
    }

    Ok(())
}
