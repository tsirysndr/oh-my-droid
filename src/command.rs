use std::process::Command;

use anyhow::Error;
use owo_colors::OwoColorize;

pub fn run_command(cmd: &str, args: &[&str]) -> Result<(), Error> {
    println!(
        "{} {} {}",
        "=>".green(),
        cmd.green(),
        args.join(" ").green()
    );
    Command::new(cmd)
        .args(args)
        .env(
            "PATH",
            format!(
                "{}/.local/bin:{}",
                std::env::var("HOME")?,
                std::env::var("PATH")?
            ),
        )
        .status()?;
    Ok(())
}

pub fn run_command_without_local_path(cmd: &str, args: &[&str]) -> Result<(), Error> {
    println!(
        "{} {} {}",
        "=>".green(),
        cmd.green(),
        args.join(" ").green()
    );
    Command::new(cmd).args(args).status()?;
    Ok(())
}
