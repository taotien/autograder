use std::fs::read_to_string;
use std::path::Path;

use anyhow::Context;
use autograde_rs::cli::{Cli, Command};
use autograde_rs::config::Config;
use autograde_rs::test::Tests;
use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::read_or_create()?;
    println!("{:#?}", config);

    let args = Cli::parse();

    match args.command {
        Command::Test => {
            let config_test = config.test.context("Config file missing test section!")?;

            let tests_path = &config_test
                .tests_path
                .context("Could not find test_path in config file!")?;

            // TODO support tilde expansion
            // TODO search pwd/parents for tests dir
            let tests_path = Path::new(tests_path);
            let tests_file = read_to_string(tests_path)?;
            let tests: Tests = toml::from_str(&tests_file)
                .with_context(|| format!("Could not parse tests at {}!", tests_path.display()))?;

            // TODO auto pull

            let grade = tests.run().await?;
            println!("{}", grade);
        }
    }

    Ok(())
}
