use std::{fs, path::Path};

use anyhow::{Context, Error};
use owo_colors::OwoColorize;
use url::Url;

use crate::{
    command::run_command, config::Configuration, consts::CONFIG_FILE, diff::compare_configurations,
};

pub fn setup(dry_run: bool, no_confirm: bool, config_path: &str) -> Result<(), Error> {
    let mut cfg = Configuration::default();

    let repo_url = parse_config_path(config_path)?;

    let toml_config = match clone_repo(&repo_url) {
        Ok(toml_config) => toml_config,
        Err(err) => {
            if !repo_url.starts_with("https://") {
                repo_url
            } else {
                return Err(err);
            }
        }
    };

    if std::path::Path::new(&toml_config).exists() {
        let toml_str = std::fs::read_to_string(&toml_config)?;
        cfg = toml::from_str(&toml_str)?;
    }

    if toml_config != CONFIG_FILE && !std::path::Path::new(&toml_config).exists() {
        return Err(anyhow::anyhow!(
            "{} does not exist.",
            toml_config.as_str().green()
        ));
    }

    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Failed to get home directory"))?;
    let diffs = match Path::new(&home_dir).join(".oh-my-droid/lock.toml").exists() {
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

    cfg.validate()?;

    println!("The following changes will be made:");
    for d in diffs.iter().clone() {
        println!("{}", d);
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

fn parse_config_path(config_path: &str) -> Result<String, Error> {
    if config_path.starts_with("github:") {
        let repo = &config_path["github:".len()..];
        return Ok(format!("https://github.com/{}.git", repo));
    }

    if config_path.starts_with("tangled:") {
        let repo = &config_path["tangled:".len()..];
        return Ok(format!("https://tangled.sh/{}", repo));
    }

    Ok(config_path.to_string())
}

fn clone_repo(repo_url: &str) -> Result<String, Error> {
    if !repo_url.starts_with("https://") {
        return Err(anyhow::anyhow!(
            "Unsupported repository URL. Only HTTPS URLs are supported."
        ));
    }

    // extract repo name: username-repo: e.g: https://github.com/tsirysndr/pkgs.git -> tsirysndr-pkgs
    let repo_name = extract_repo_name(repo_url).context("Failed to extract repository name")?;
    let home_dir = dirs::home_dir().context("Failed to get home directory")?;
    let cache = home_dir.join(".oh-my-droid").join("cache");
    fs::create_dir_all(&cache)?;
    let dest = cache.join(repo_name);

    run_command(
        "bash",
        &["-c", "type git || (apt update && apt install -y git)"],
    )?;

    match dest.exists() {
        true => {
            run_command("git", &["-C", dest.to_str().unwrap(), "pull"])?;
        }
        false => {
            run_command(
                "git",
                &["clone", "--depth", "1", repo_url, dest.to_str().unwrap()],
            )?;
        }
    }

    if !dest.join("oh-my-droid.toml").exists() {
        return Err(anyhow::anyhow!(
            "The repository does not contain an oh-my-droid.toml configuration file."
        ));
    }

    Ok(dest.join("oh-my-droid.toml").to_str().unwrap().to_string())
}

fn extract_repo_name(url: &str) -> Option<String> {
    let parsed = Url::parse(url).ok()?;
    let mut segments = parsed.path_segments()?;
    let username = segments.next()?;
    let mut repo = segments.next()?;

    if let Some(stripped) = repo.strip_suffix(".git") {
        repo = stripped;
    }

    Some(format!("{}-{}", username, repo))
}
