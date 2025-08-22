use anyhow::{Context, Error, Result};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
    process::Command,
};

use crate::{apply::SetupStep, diff::Diff};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OhMyPosh {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorized_keys: Option<Vec<String>>,
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

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tailscale: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh: Option<SshConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub neofetch: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub doppler: Option<bool>,
}

impl Configuration {
    pub fn empty() -> Self {
        Self {
            stow: None,
            mise: None,
            nix: None,
            apt_get: None,
            pkgx: None,
            curl: None,
            blesh: None,
            oh_my_posh: None,
            zoxide: None,
            alias: None,
            tailscale: None,
            ssh: None,
            neofetch: None,
            doppler: None,
        }
    }

    pub fn setup_environment(&self, dry_run: bool, diffs: Vec<Diff>) -> Result<()> {
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

        let steps = self.diffs_to_setup_steps(diffs);

        if dry_run {
            println!("{}", "=== Dry Run: Environment Setup ===".yellow().bold());
            println!("Steps to be executed ({} total):", steps.len());
            for (i, step) in steps.iter().enumerate() {
                println!("\n=> Step {}:\n{}", i + 1, step.format_dry_run());
            }
            println!("{}", "=== Dry Run Complete ===".yellow().bold());
            return Ok(());
        }

        for step in steps {
            step.run()?;
        }

        self.write_lock_file()?;

        println!("{}", "Environment setup completed successfully 🎉".green());
        println!("You can now open a new terminal to see the changes.");
        println!(
            "Or run {} to apply the changes to the current terminal session.",
            "source ~/.bashrc".green()
        );

        Ok(())
    }

    pub fn write_lock_file(&self) -> Result<()> {
        let home_dir = dirs::home_dir().context("Failed to get home directory")?;
        let config_path = home_dir.join(".oh-my-droid/lock.toml");

        fs::create_dir_all(
            config_path
                .parent()
                .context("Failed to get parent directory of lock file")?,
        )?;

        let mut file = File::create(&config_path).context("Failed to create lock file")?;
        file.write_all(
            toml::to_string(&self)
                .context("Failed to serialize config")?
                .as_bytes(),
        )
        .context("Failed to write lock file")?;

        Ok(())
    }

    pub fn load_lock_file() -> Result<Configuration> {
        let home_dir = dirs::home_dir().context("Failed to get home directory")?;
        let config_path = home_dir.join(".oh-my-droid/lock.toml");

        let toml_str = fs::read_to_string(&config_path).context("Failed to read lock file")?;
        let loaded_config: Configuration =
            toml::from_str(&toml_str).context("Failed to parse lock file")?;

        Ok(loaded_config)
    }

    pub fn diffs_to_setup_steps<'a>(&'a self, diffs: Vec<Diff>) -> Vec<SetupStep<'a>> {
        let mut steps = Vec::new();

        steps.push(SetupStep::Paths);

        for diff in diffs {
            match diff {
                Diff::Added(parent, _child, _value) => {
                    self.add_setup_step_for_parent(&mut steps, &parent);
                }
                Diff::Changed(parent, _child, _old, _new) => {
                    self.add_setup_step_for_parent(&mut steps, &parent);
                }
                Diff::Nested(parent, nested_diffs) => {
                    self.add_setup_step_for_parent(&mut steps, &parent);
                    let nested_steps = self.diffs_to_setup_steps(nested_diffs);
                    steps.extend(
                        nested_steps
                            .into_iter()
                            .filter(|step| !matches!(step, SetupStep::Paths)),
                    );
                }
                Diff::Removed(_parent, _child, _value) => {
                    // For removed items, we typically don't need to do anything
                    // as the setup is additive, but you could add cleanup logic here if needed
                }
            }
        }

        steps.dedup_by(|a, b| std::mem::discriminant(a) == std::mem::discriminant(b));

        steps
    }

    fn add_setup_step_for_parent<'a>(&'a self, steps: &mut Vec<SetupStep<'a>>, parent: &str) {
        match parent {
            "apt-get" => {
                if let Some(apt_packages) = &self.apt_get {
                    steps.push(SetupStep::AptGet(apt_packages));
                }
            }
            "pkgx" => {
                if let Some(pkgx_packages) = &self.pkgx {
                    steps.push(SetupStep::Pkgx(pkgx_packages));
                }
            }
            "curl" => {
                if let Some(curl_installers) = &self.curl {
                    steps.push(SetupStep::Curl(curl_installers));
                }
            }
            "mise" => {
                if let Some(mise_tools) = &self.mise {
                    steps.push(SetupStep::Mise(mise_tools));
                }
            }
            "ble.sh" => {
                if let Some(blesh_enabled) = self.blesh {
                    steps.push(SetupStep::BleSh(blesh_enabled));
                }
            }
            "nix" => {
                if let Some(nix_packages) = &self.nix {
                    steps.push(SetupStep::Nix(nix_packages));
                }
            }
            "stow" => {
                if let Some(stow_configs) = &self.stow {
                    steps.push(SetupStep::Stow(stow_configs));
                }
            }
            "oh_my_posh" => {
                if let Some(oh_my_posh) = &self.oh_my_posh {
                    let theme = oh_my_posh.theme.as_deref().unwrap_or("tokyonight_storm");
                    steps.push(SetupStep::OhMyPosh(theme));
                }
            }
            "zoxide" => {
                if let Some(zoxide_enabled) = self.zoxide {
                    steps.push(SetupStep::Zoxide(zoxide_enabled));
                }
            }
            "alias" => {
                if let Some(aliases) = &self.alias {
                    steps.push(SetupStep::Alias(aliases));
                }
            }
            "ssh" => {
                if let Some(ssh_config) = &self.ssh {
                    steps.push(SetupStep::Ssh(ssh_config));
                }
            }
            "tailscale" => {
                if let Some(tailscale_enabled) = self.tailscale {
                    steps.push(SetupStep::Tailscale(tailscale_enabled));
                }
            }
            "neofetch" => {
                if let Some(neofetch_enabled) = self.neofetch {
                    steps.push(SetupStep::Neofetch(neofetch_enabled));
                }
            }
            "doppler" => {
                if let Some(doppler_enabled) = self.doppler {
                    steps.push(SetupStep::Doppler(doppler_enabled));
                }
            }
            _ => {} // Ignore unknown configuration keys
        }
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
                    "neofetch",
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
            tailscale: Some(false),
            ssh: Some(SshConfig {
                port: Some(8022),
                authorized_keys: Some(vec![]),
            }),
            neofetch: Some(true),
            doppler: Some(false),
        }
    }
}
