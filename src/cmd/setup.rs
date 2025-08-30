use std::{fs, path::Path};

use anyhow::{Context, Error};
use owo_colors::OwoColorize;

use crate::{
    command::run_command,
    config::Configuration,
    consts::CONFIG_FILE,
    diff::compare_configurations,
    git::{extract_repo_name, extract_version},
};

pub fn setup(dry_run: bool, no_confirm: bool, config_path: &str) -> Result<(), Error> {
    let mut cfg = Configuration::default();

    let repo_url = parse_config_path(config_path)?;
    let (repo_url, version) = match repo_url.starts_with("https://") {
        true => extract_version(&repo_url),
        false => (repo_url, None),
    };

    let toml_config = match clone_repo(&repo_url, version) {
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
        return Ok(format!("https://github.com/{}", repo));
    }

    if config_path.starts_with("tangled:") {
        let repo = &config_path["tangled:".len()..];
        return Ok(format!("https://tangled.sh/{}", repo));
    }

    Ok(config_path.to_string())
}

fn clone_repo(repo_url: &str, version: Option<String>) -> Result<String, Error> {
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
            run_command("git", &["clone", repo_url, dest.to_str().unwrap()])?;
        }
    }

    if version.is_some() {
        run_command("git", &["-C", dest.to_str().unwrap(), "fetch", "--all"])?;
        run_command(
            "git",
            &[
                "-C",
                dest.to_str().unwrap(),
                "checkout",
                version.as_ref().unwrap(),
            ],
        )?;
    }

    if !dest.join("oh-my-droid.toml").exists() {
        return Err(anyhow::anyhow!(
            "The repository does not contain an oh-my-droid.toml configuration file."
        ));
    }

    Ok(dest.join("oh-my-droid.toml").to_str().unwrap().to_string())
}

#[cfg(test)]
mod tests {
    use crate::git::extract_version;

    use super::*;

    #[test]
    fn test_parse_config_path_github() {
        let path = "github:tsirysndr/pkgs";
        let expected = Some("https://github.com/tsirysndr/pkgs".into());
        let result = parse_config_path(path).ok();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_config_path_tangled() {
        let path = "tangled:@tsirysandratraina/pkgs";
        let expected = Some("https://tangled.sh/@tsirysandratraina/pkgs".into());
        let result = parse_config_path(path).ok();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_config_path_git_url() {
        let path = "https://github.com/tsirysndr/pkgs@main";
        let expected = Some("https://github.com/tsirysndr/pkgs@main".into());
        let result = parse_config_path(path).ok();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_extract_version() {
        let url = "https://tangled.sh/@tsirysandratraina/pkgs@main";
        let expected = (
            "https://tangled.sh/@tsirysandratraina/pkgs".into(),
            Some("main".into()),
        );
        let result = extract_version(url);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_extract_version_2() {
        let url = "https://tangled.sh/@tsirysandratraina/pkgs";
        let expected = ("https://tangled.sh/@tsirysandratraina/pkgs".into(), None);
        let result = extract_version(url);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_extract_version_3() {
        let url = "https://github.com/tsirysndr/pkgs";
        let expected = ("https://github.com/tsirysndr/pkgs".into(), None);
        let result = extract_version(url);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_extract_version_4() {
        let url = "https://github.com/tsirysndr/pkgs@main";
        let expected = (
            "https://github.com/tsirysndr/pkgs".into(),
            Some("main".into()),
        );
        let result = extract_version(url);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_config_path_github_with_branch() {
        let path = "github:tsirysndr/pkgs@main";
        let expected = Some("https://github.com/tsirysndr/pkgs@main".into());
        let result = parse_config_path(path).ok();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_config_path() {
        let path = "/custom/path/to/oh-my-droid.toml";
        let expected = Some("/custom/path/to/oh-my-droid.toml".into());
        let result = parse_config_path(path).ok();
        assert_eq!(result, expected);
    }
}
