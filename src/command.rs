use std::process::{self, Command};

use anyhow::Error;
use owo_colors::OwoColorize;

pub fn run_command(cmd: &str, args: &[&str]) -> Result<(), Error> {
    println!(
        "{} {} {}",
        "=>".green(),
        cmd.green(),
        args.join(" ").green()
    );
    let status = Command::new(cmd)
        .args(args)
        .env(
            "PATH",
            format!(
                "{}:{}/.local/bin:{}",
                "/nix/var/nix/profiles/default/bin",
                std::env::var("HOME")?,
                std::env::var("PATH")?
            ),
        )
        .status()?;

    if !status.success() {
        println!("Command failed: {}", status);
        process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

pub fn run_command_without_local_path(cmd: &str, args: &[&str]) -> Result<(), Error> {
    println!(
        "{} {} {}",
        "=>".green(),
        cmd.green(),
        args.join(" ").green()
    );
    let status = Command::new(cmd).args(args).status()?;

    if !status.success() {
        println!("Command failed: {}", status);
        process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}
