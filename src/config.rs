use std::fs;
use anyhow::Context;
use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub name: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            name: "CHANGEME".to_owned(),
        }
    }
}

const CONFIGFILE : &str = "odr-dabmux-gui-config.toml";

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        if std::path::Path::new(CONFIGFILE).exists() {
            let file_contents = fs::read_to_string(CONFIGFILE)?;
            toml::from_str(&file_contents).context("parsing config file")
        }
        else {
            Ok(Default::default())
        }
    }

    pub fn store(&self) -> anyhow::Result<()> {
        fs::write(CONFIGFILE, toml::to_string_pretty(&self)?)
            .context("writing config file")
    }
}
