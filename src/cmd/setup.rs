use anyhow::Error;
use owo_colors::OwoColorize;

use crate::{config::Configuration, consts::CONFIG_FILE};

pub fn setup(dry_run: bool, no_confirm: bool) -> Result<(), Error> {
    let mut cfg = Configuration::default();

    if std::path::Path::new(CONFIG_FILE).exists() {
        let toml_str = std::fs::read_to_string(CONFIG_FILE)?;
        cfg = toml::from_str(&toml_str)?;
    }

    if !no_confirm && !dry_run {
        match std::path::Path::new(CONFIG_FILE).exists() {
            true => {
                println!(
                    "This will set up your environment with the default configuration from {}.\nDo you want to continue? (y/N)",
                    CONFIG_FILE.green()
                );
            }
            false => {
                println!(
                    "This wil set up your environment with the default configuration.\nDo you want to continue? (y/N)",
                );
            }
        }

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Setup cancelled.");
            return Ok(());
        }
    }

    cfg.setup_environment(dry_run)?;

    Ok(())
}
