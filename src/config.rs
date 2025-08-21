use anyhow::{Context, Error, Result};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, process::Command};

use crate::apply::SetupStep;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OhMyPosh {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stow: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mise: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub nix: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "apt-get")]
    pub apt_get: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub pkgx: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub curl: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ble.sh")]
    pub blesh: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub oh_my_posh: Option<OhMyPosh>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub zoxide: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<HashMap<String, String>>,
}

impl Configuration {
    pub fn setup_environment(&self, dry_run: bool) -> Result<()> {
        let output = Command::new("df")
            .args(&["-BG", "--output=size", "/"])
            .output()
            .context("Failed to check disk size")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let disk_size = stdout
            .lines()
            .nth(1)
            .unwrap_or("0")
            .replace("G", "")
            .trim()
            .parse::<u64>()
            .unwrap_or(0);

        if disk_size < 7 {
            println!("Disk size: {}GB", disk_size);
            return Err(Error::msg("Insufficient disk size: >= 7GB required"));
        }

        let steps: Vec<SetupStep> = vec![
            Some(SetupStep::Paths),
            self.apt_get.as_deref().map(SetupStep::AptGet),
            self.curl.as_ref().map(SetupStep::Curl),
            self.pkgx.as_ref().map(SetupStep::Pkgx),
            self.mise.as_ref().map(SetupStep::Mise),
            self.blesh.map(SetupStep::BleSh),
            self.zoxide.map(SetupStep::Zoxide),
            self.nix.as_ref().map(SetupStep::Nix),
            self.stow.as_ref().map(SetupStep::Stow),
            self.oh_my_posh
                .as_ref()
                .map(|omp| SetupStep::OhMyPosh(omp.theme.as_deref().unwrap_or("tokyonight_storm"))),
            self.alias.as_ref().map(SetupStep::Alias),
            Some(SetupStep::Ssh),
        ]
        .into_iter()
        .flatten()
        .collect();

        if dry_run {
            println!("{}", "=== Dry Run: Environment Setup ===".yellow().bold());
            println!("Steps to be executed ({} total):", steps.len());
            for (i, step) in steps.iter().enumerate() {
                println!("\n=> Step {}:\n{}", i + 1, step.format_dry_run());
            }
            println!("{}", "=== Dry Run Complete ===".yellow().bold());
        } else {
            for step in steps {
                step.run()?;
            }
            println!("{}", "Environment setup completed successfully 🎉".green());
            println!("You can now open a new terminal to see the changes.");
            println!(
                "Or run {} to apply the changes to the current terminal session.",
                "source ~/.bashrc".green()
            );
        }
        Ok(())
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            apt_get: Some(
                vec![
                    "build-essential",
                    "curl",
                    "git",
                    "gawk",
                    "wget",
                    "unzip",
                    "autoconf",
                    "automake",
                    "cmake",
                    "tmux",
                    "openssh-server",
                    "openssh-client",
                    "httpie",
                    "code",
                    "screenfetch",
                    "stow",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
            ),
            pkgx: Some(HashMap::from([
                ("tig".into(), "latest".into()),
                ("rg".into(), "latest".into()),
                ("jq".into(), "latest".into()),
                ("neovim.io".into(), "latest".into()),
                ("fzf".into(), "latest".into()),
                ("zellij".into(), "latest".into()),
                ("glow".into(), "latest".into()),
                ("gh".into(), "latest".into()),
                ("eza".into(), "latest".into()),
            ])),
            curl: Some(HashMap::from([
                (
                    "oh-my-posh".into(),
                    "https://ohmyposh.dev/install.sh".into(),
                ),
                ("atuin".into(), "https://setup.atuin.sh".into()),
                ("bun".into(), "https://bun.sh/install".into()),
                ("deno".into(), "https://deno.land/install.sh".into()),
                ("pkgx".into(), "https://pkgx.sh".into()),
            ])),
            mise: Some(HashMap::from([("node".into(), "latest".into())])),
            blesh: Some(true),
            zoxide: Some(true),
            nix: None,
            stow: Some(HashMap::from([(
                "git".into(),
                "github:tsirysndr/android-dotfiles".into(),
            )])),
            oh_my_posh: Some(OhMyPosh {
                theme: Some("tokyonight_storm".into()),
            }),
            alias: Some(HashMap::from([("ls".into(), "eza -lh".into())])),
        }
    }
}
