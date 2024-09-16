use std::env::current_dir;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::Context;
use autograde_rs::cli::{Cli, Command};
use autograde_rs::config::Config;
use autograde_rs::test::Tests;
use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::read_or_create()?;

    let args = Cli::parse();

    let pwd = current_dir()?;
    let project = pwd.file_name().unwrap();

    match args.command {
        Command::Test => {
            let config_test = config.test.context("Config file missing test section!")?;

            let tests_path = &config_test
                .tests_path
                .context("Could not find test_path in config file!")?;

            // TODO support tilde expansion
            // TODO search pwd/parents for tests dir
            let mut tests_path = PathBuf::from_str(&tests_path)
                .with_context(|| format!("Invalid path! {}", tests_path))?;
            tests_path.push(project);
            tests_path.push(project);
            tests_path.set_extension("toml");
            let tests_file = read_to_string(&tests_path)?;

            let tests: Tests = toml::from_str(&tests_file)
                .with_context(|| format!("Could not parse tests at {}!", tests_path.display()))?;

            // TODO auto pull

            let grade = tests.run().await?;
            println!("{}", grade);
        }
    }

    Ok(())
}
