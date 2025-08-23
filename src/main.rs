use anyhow::Error;
use clap::{Command, arg};
use owo_colors::OwoColorize;

use crate::{
    cmd::{init::init, setup::setup},
    consts::CONFIG_FILE,
};

pub mod apply;
pub mod cmd;
pub mod command;
pub mod config;
pub mod consts;
pub mod diff;

fn cli() -> Command {
    let banner = format!(
        "{}\nTurn a fresh {} into a fully-configured, beautiful, and modern web development system by running a single command.",
        r#"
       ______                              _________            ______________
 _________  /_     _______ ________  __    ______  /_______________(_)_____  /
 _  __ \_  __ \    __  __ `__ \_  / / /    _  __  /__  ___/  __ \_  /_  __  /
 / /_/ /  / / /    _  / / / / /  /_/ /     / /_/ / _  /   / /_/ /  / / /_/ /
 \____//_/ /_/     /_/ /_/ /_/_\__, /      \__,_/  /_/    \____//_/  \__,_/
                              /____/

"#
        .green(),
        "Android 15+ Linux Terminal".green()
    );

    Command::new("oh-my-droid")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Tsiry Sandratraina <tsiry.sndr@rocksky.app>")
        .about(&banner)
        .subcommand(Command::new("init").about(&format!(
            "Write the initial configuration file {}.",
            CONFIG_FILE.green()
        )))
        .subcommand(
            Command::new("setup")
                .about("Set up the environment with the default configuration.")
                .arg(arg!(-d --"dry-run" "Simulate the setup process without making any changes."))
                .arg(arg!(-y --"yes" "Skip confirmation prompts during setup."))
                .arg(
                    arg!([config] "Path to a custom configuration file or a remote git repository e.g., github:tsirysndr/pkgs")
                        .default_value(CONFIG_FILE),
                )
                .alias("apply"),
        )
        .arg(arg!(-d --"dry-run" "Simulate the setup process without making any changes."))
        .arg(arg!(-y --"yes" "Skip confirmation prompts during setup."))
        .arg(
            arg!([config] "Path to a custom configuration file or a remote git repository e.g., github:tsirysndr/pkgs")
                .default_value(CONFIG_FILE),
        )
        .after_help("If no subcommand is provided, the 'setup' command will be executed by default.")
}

fn main() -> Result<(), Error> {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("init", _)) => init()?,
        Some(("setup", args)) => {
            let yes = args.get_flag("yes");
            let dry_run = args.get_flag("dry-run");
            let config = args.get_one::<String>("config").unwrap();
            setup(dry_run, yes, config)?
        }
        _ => {
            let yes = matches.get_flag("yes");
            let dry_run = matches.get_flag("dry-run");
            let config = matches.get_one::<String>("config").unwrap();
            setup(dry_run, yes, config)?
        }
    }

    Ok(())
}
