use std::fs::{create_dir, read_to_string, write};

use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    // Canvas: ,
    // CanvasMapper: ,
    // Config: ,
    // Git: ,
    // Github:

    // TODO convert existing files to not use uppercase?
    #[serde(rename(deserialize = "Test"))]
    pub test: Option<Test>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Test {
    // TODO make this a vec to support multiple classes
    pub tests_path: Option<String>,

    // TODO this should just be on user's PATH
    pub digital_path: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        todo!()
    }
}

impl Default for Test {
    fn default() -> Self {
        todo!()
    }
}

impl Config {
    pub fn read_or_create() -> anyhow::Result<Self> {
        let config_path = dirs::config_dir();
        let mut config_path = config_path.context("Default config directory missing!")?;
        config_path.push("grade/config.toml");

        let config_str = read_to_string(&config_path);

        match config_str {
            Ok(str) => toml::from_str(&str)
                .with_context(|| format!("Could not parse config at {}!", config_path.display())),

            Err(_) => {
                config_path.pop();
                create_dir(&config_path).with_context(|| {
                    format!("Could not create directory {}!", &config_path.display())
                })?;

                // TODO interactive config builder
                let config = Self::default();

                let config_str = toml::to_string(&config)?;
                config_path.push("grade/config.toml");
                write(&config_path, config_str).with_context(|| {
                    format!("Could not create config at {}!", &config_path.display())
                })?;

                Ok(config)
            }
        }
    }
}
