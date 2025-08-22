use std::path::Path;

use anyhow::Error;
use owo_colors::OwoColorize;

use crate::{config::Configuration, consts::CONFIG_FILE, diff::compare_configurations};

pub fn setup(dry_run: bool, no_confirm: bool) -> Result<(), Error> {
    let mut cfg = Configuration::default();

    if std::path::Path::new(CONFIG_FILE).exists() {
        let toml_str = std::fs::read_to_string(CONFIG_FILE)?;
        cfg = toml::from_str(&toml_str)?;
    }

    let mut diffs = Vec::new();

    if !no_confirm && !dry_run {
        let home_dir =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Failed to get home directory"))?;
        diffs = match Path::new(&home_dir).join(".oh-my-droid/lock.toml").exists() {
            true => {
                let old_cfg = Configuration::load_lock_file()?;
                compare_configurations(&old_cfg, &cfg)
            }
            false => compare_configurations(&Configuration::empty(), &cfg),
        };

        if diffs.is_empty() {
            println!(
                "{}",
                "No changes detected. Your environment is already up to date.".green()
            );
            return Ok(());
        }

        println!("The following changes will be made:");
        for d in diffs.iter().clone() {
            println!("{}", d);
        }

        match std::path::Path::new(CONFIG_FILE).exists() {
            true => {
                println!(
                    "This will set up your environment with the default configuration from {}.\nDo you want to continue? (y/N)",
                    CONFIG_FILE.green()
                );
            }
            false => {
                println!(
                    "This will set up your environment with the default configuration.\nDo you want to continue? (y/N)",
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

    cfg.setup_environment(dry_run, diffs)?;

    Ok(())
}
