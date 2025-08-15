use std::{collections::HashMap, path::Path};

use anyhow::{Context, Error};
use owo_colors::OwoColorize;

use crate::command::{run_command, run_command_without_local_path};

#[derive(Debug)]
pub enum SetupStep<'a> {
    AptGet(&'a [String]),
    Pkgx(&'a HashMap<String, String>),
    Curl(&'a HashMap<String, String>),
    Mise(&'a HashMap<String, String>),
    BleSh(bool),
    Nix(&'a HashMap<String, String>),
    Stow(&'a HashMap<String, String>),
    OhMyPosh(&'a str),
    Zoxide(bool),
    Alias(&'a HashMap<String, String>),
    Paths,
}

impl<'a> SetupStep<'a> {
    pub fn run(&self) -> Result<(), Error> {
        match self {
            SetupStep::AptGet(pkgs) => install_apt(pkgs),
            SetupStep::Pkgx(map) => install_pkgx(map),
            SetupStep::Curl(map) => run_curl_installers(map),
            SetupStep::Mise(map) => setup_mise(map),
            SetupStep::BleSh(enabled) => enable_blesh(*enabled),
            SetupStep::Nix(map) => setup_nix(map),
            SetupStep::Stow(map) => setup_stow(map),
            SetupStep::OhMyPosh(theme) => setup_oh_my_posh(theme),
            SetupStep::Zoxide(enabled) => enable_zoxide(*enabled),
            SetupStep::Alias(map) => setup_alias(map),
            SetupStep::Paths => setup_paths(),
        }
    }

    pub fn format_dry_run(&self) -> String {
        match self {
            SetupStep::AptGet(pkgs) => {
                let pkg_list = pkgs
                    .iter()
                    .map(|p| format!("  - {}", p.green()))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!(
                    "{} {}\n{}",
                    "AptGet".blue().bold(),
                    "(Install system packages via apt-get)".italic(),
                    pkg_list
                )
            }
            SetupStep::Pkgx(map) => {
                let pkg_list = map
                    .iter()
                    .map(|(k, v)| format!("  - {}: {}", k.green(), v.cyan()))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!(
                    "{} {}\n{}",
                    "Pkgx".blue().bold(),
                    "(Install tools via pkgx)".italic(),
                    pkg_list
                )
            }
            SetupStep::Curl(map) => {
                let curl_list = map
                    .iter()
                    .map(|(k, v)| format!("  - {}: {}", k.green(), v.cyan()))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!(
                    "{} {}\n{}",
                    "Curl".blue().bold(),
                    "(Run curl-based installers)".italic(),
                    curl_list
                )
            }
            SetupStep::Mise(map) => {
                let mise_list = map
                    .iter()
                    .map(|(k, v)| format!("  - {}: {}", k.green(), v.cyan()))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!(
                    "{} {}\n{}",
                    "Mise".blue().bold(),
                    "(Configure tools via mise)".italic(),
                    mise_list
                )
            }
            SetupStep::BleSh(enabled) => {
                format!(
                    "{} {}\n  - Enabled: {}",
                    "BleSh".blue().bold(),
                    "(Enable ble.sh shell enhancements)".italic(),
                    enabled.to_string().green()
                )
            }
            SetupStep::Zoxide(enabled) => {
                format!(
                    "{} {}\n  - Enabled: {}",
                    "Zoxide".blue().bold(),
                    "(Enable zoxide for directory navigation)".italic(),
                    enabled.to_string().green()
                )
            }
            SetupStep::Nix(map) => {
                let nix_list = map
                    .iter()
                    .map(|(k, v)| format!("  - {}: {}", k.green(), v.cyan()))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!(
                    "{} {}\n{}",
                    "Nix".blue().bold(),
                    "(Install tools via nix)".italic(),
                    nix_list
                )
            }
            SetupStep::Stow(map) => {
                let stow_list = map
                    .iter()
                    .map(|(k, v)| format!("  - {}: {}", k.green(), v.cyan()))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!(
                    "{} {}\n{}",
                    "Stow".blue().bold(),
                    "(Manage dotfiles via stow)".italic(),
                    stow_list
                )
            }
            SetupStep::OhMyPosh(theme) => {
                format!(
                    "{} {}\n  - Theme: {}",
                    "OhMyPosh".blue().bold(),
                    "(Setup Oh My Posh for shell prompt)".italic(),
                    theme.green()
                )
            }
            SetupStep::Alias(map) => {
                let alias_list = map
                    .iter()
                    .map(|(k, v)| format!("  - {}: {}", k.green(), v.cyan()))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!(
                    "{} {}\n{}",
                    "Alias".blue().bold(),
                    "(Setup shell aliases)".italic(),
                    alias_list
                )
            }
            SetupStep::Paths => {
                format!(
                    "{} {}\n{}",
                    "Paths".blue().bold(),
                    "(Setup paths for binaries)".italic(),
                    "  - ~/.local/bin".green()
                )
            }
        }
    }
}

fn install_apt(pkgs: &[String]) -> Result<(), Error> {
    if pkgs.is_empty() {
        return Ok(());
    }

    run_command("sudo", &["apt-get", "update"]).context("Failed to run apt-get update")?;
    if !Path::new("/etc/apt/sources.list.d/vscode.list").exists() {
        run_command("sudo", &["apt-get", "install", "-y", "wget", "curl", "gpg"])?;
        run_command(
            "bash",
            &[
                "-c",
                "wget -qO- https://packages.microsoft.com/keys/microsoft.asc | sudo gpg --dearmor > packages.microsoft.gpg",
            ],
        )?;
        run_command(
            "sudo",
            &[
                "install",
                "-D",
                "-o",
                "root",
                "-g",
                "root",
                "-m",
                "644",
                "packages.microsoft.gpg",
                "/etc/apt/keyrings/packages.microsoft.gpg",
            ],
        )?;
        run_command(
            "bash",
            &[
                "-c",
                "echo 'deb [arch=amd64,arm64,armhf signed-by=/etc/apt/keyrings/packages.microsoft.gpg] https://packages.microsoft.com/repos/code stable main' | sudo tee /etc/apt/sources.list.d/vscode.list",
            ],
        )?;
        run_command("rm", &["-f", "packages.microsoft.gpg"])?;
        run_command("sudo", &["apt-get", "update"]).context("Failed to run apt-get update")?;
    }

    if !Path::new("/etc/apt/sources.list.d/mise.list").exists() {
        run_command("bash", &[
      "-c",
      "wget -qO - https://mise.jdx.dev/gpg-key.pub | gpg --dearmor | sudo tee /etc/apt/keyrings/mise-archive-keyring.gpg 1> /dev/null
"])?;
        run_command(
            "bash",
            &[
                "-c",
                "echo 'deb [signed-by=/etc/apt/keyrings/mise-archive-keyring.gpg arch=amd64,arm64] https://mise.jdx.dev/deb stable main' | sudo tee /etc/apt/sources.list.d/mise.list",
            ],
        )?;
        run_command("sudo", &["apt-get", "update"]).context("Failed to run apt-get update")?;
    }

    let mut args: Vec<&str> = vec!["apt-get", "install", "-y"];
    args.extend(pkgs.iter().map(|s| s.as_str()));
    run_command("sudo", &args).context("Failed to run apt-get install")?;

    run_command(
        "sudo",
        &["rm", "-rf", "/etc/apt/sources.list.d/vscode.list"],
    )?;

    Ok(())
}

fn install_pkgx(map: &HashMap<String, String>) -> Result<(), Error> {
    for (name, ver) in map {
        run_command("pkgm", &["install", &format!("{name}@{ver}")])
            .context(format!("Failed to install {name} via pkgx"))?;
    }
    run_command("pkgm", &["uninstall", "curl"]).context("Failed to uninstall curl via pkgx")?;
    Ok(())
}

fn run_curl_installers(map: &HashMap<String, String>) -> Result<(), Error> {
    for (name, url) in map {
        run_command("bash", &["-c", &format!("curl -fsSL {} | bash -s", url)])
            .context(format!("Failed to run curl installer for {name}"))?;
    }
    Ok(())
}

fn setup_mise(map: &HashMap<String, String>) -> Result<(), Error> {
    if !Path::new("/usr/bin/mise").exists() {
        run_command("sudo", &["apt-get", "install", "-y", "mise"])
            .context("Failed to install mise")?;
    }

    run_command(
        "bash",
        &[
            "-c",
            "sed -i '/mise /d' ~/.bashrc || echo 'No existing mise line found in .bashrc'",
        ],
    )?;
    run_command(
        "bash",
        &[
            "-c",
            "echo '\neval \"$(mise activate bash)\"' | tee -a ~/.bashrc",
        ],
    )?;

    for (tool, ver) in map {
        run_command("mise", &["use", "-g", &format!("{tool}@{ver}")])
            .context(format!("Failed to configure {tool} via mise"))?;
    }
    Ok(())
}

fn enable_blesh(enabled: bool) -> Result<(), Error> {
    let home = dirs::home_dir().ok_or_else(|| Error::msg("Failed to get home directory"))?;
    let blesh_path = home.join("ble.sh");
    if enabled && !blesh_path.exists() {
        run_command_without_local_path(
            "bash",
            &[
                "-c", "rm -rf ~/.local/bin/gettext* &&git clone --recursive --depth 1 --shallow-submodules https://github.com/akinomyoga/ble.sh.git",
            ],
        )
        .context("Failed to clone ble.sh repository")?;
        run_command_without_local_path("make", &["-C", "ble.sh"])
            .context("Failed to build ble.sh")?;
        run_command_without_local_path(
            "bash",
            &[
                "-c",
                "grep 'source ble' ~/.bashrc || echo '\nsource ble.sh/out/ble.sh' | tee -a ~/.bashrc",
            ],
        )
        .context("Failed to add ble.sh to .bashrc")?;
    }
    Ok(())
}

fn enable_zoxide(enabled: bool) -> Result<(), Error> {
    if enabled {
        run_command("bash", &["-c", "curl -sSL https://raw.githubusercontent.com/ajeetdsouza/zoxide/main/install.sh | bash"])
            .context("Failed to install zoxide")?;
        run_command(
            "bash",
            &[
                "-c",
                "grep zoxide ~/.bashrc || echo '\neval \"$(zoxide init bash)\"' | tee -a ~/.bashrc",
            ],
        )
        .context("Failed to add zoxide initialization to .bashrc")?;
    }
    Ok(())
}

fn setup_nix(_map: &HashMap<String, String>) -> Result<(), Error> {
    // nix logic here
    Ok(())
}

fn setup_stow(_map: &HashMap<String, String>) -> Result<(), Error> {
    // stow logic here
    Ok(())
}

fn setup_oh_my_posh(theme: &str) -> Result<(), Error> {
    run_command(
        "bash",
        &[
            "-c",
            "sed -i '/oh-my-posh/d' ~/.bashrc || echo 'No existing oh-my-posh line found in .bashrc'",
        ],
    )?;
    run_command("bash", &["-c", &format!("echo 'eval \"$(oh-my-posh init bash --config $HOME/.cache/oh-my-posh/themes/{}.omp.json)\"' >> ~/.bashrc", theme)])
        .context("Failed to set up Oh My Posh")?;
    Ok(())
}

fn setup_alias(map: &HashMap<String, String>) -> Result<(), Error> {
    for (alias, command) in map {
        run_command(
            "bash",
            &["-c", &format!("sed -i '/alias {}/d' ~/.bashrc", alias)],
        )?;
        run_command(
            "bash",
            &[
                "-c",
                &format!("echo 'alias {}=\"{}\"' >> ~/.bashrc", alias, command),
            ],
        )
        .context(format!(
            "Failed to set up alias {} for command {}",
            alias, command
        ))?;
    }
    Ok(())
}

fn setup_paths() -> Result<(), Error> {
    let home = dirs::home_dir().ok_or_else(|| Error::msg("Failed to get home directory"))?;
    let local_bin = home.join(".local/bin");
    if !local_bin.exists() {
        std::fs::create_dir_all(&local_bin).context("Failed to create ~/.local/bin directory")?;
    }

    run_command(
        "bash",
        &["-c", "grep -q 'export PATH=\"$HOME/.local/bin:$PATH\"' ~/.bashrc || echo 'export PATH=\"$HOME/.local/bin:$PATH\"' >> ~/.bashrc"],
    )
    .context("Failed to add ~/.local/bin to PATH in .bashrc")?;

    Ok(())
}
