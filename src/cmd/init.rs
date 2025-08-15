use anyhow::Error;
use owo_colors::OwoColorize;

use crate::{config::Configuration, consts::CONFIG_FILE};

pub fn init() -> Result<(), Error> {
    let cfg = Configuration::default();
    let toml_str = toml::to_string(&cfg)?;
    std::fs::write(CONFIG_FILE, toml_str)?;
    println!(
        "Initial configuration file {} created successfully.",
        CONFIG_FILE.green()
    );
    Ok(())
}
