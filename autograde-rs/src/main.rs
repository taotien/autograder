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
            let tests: Tests = toml::from_str(&tests_path)
                .with_context(|| format!("Could not parse tests at {}!", tests_path))?;

            // TODO auto pull

            tests.run().await?;
        }
    }

    Ok(())
}
