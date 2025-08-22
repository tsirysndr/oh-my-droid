use owo_colors::OwoColorize;
use std::{collections::HashMap, fmt};

use crate::config::{Configuration, OhMyPosh, SshConfig};

#[derive(Debug)]
pub enum Diff {
    Added(String, String, String),           // Parent, child, value
    Removed(String, String, String),         // Parent, child, value
    Changed(String, String, String, String), // Parent, child, old value, new value
    Nested(String, Vec<Diff>),               // Parent field, nested differences
}

impl fmt::Display for Diff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Diff::Added(parent, child, value) => {
                if child.is_empty() {
                    write!(f, "+ {}: {}", parent.green(), value.green())
                } else {
                    write!(
                        f,
                        "+ {}:\n  + {}: {}",
                        parent.green(),
                        child.green(),
                        value.green()
                    )
                }
            }
            Diff::Removed(parent, child, value) => {
                if child.is_empty() {
                    write!(f, "- {}: {}", parent.magenta(), value.magenta())
                } else {
                    write!(
                        f,
                        "- {}:\n  - {}: {}",
                        parent.magenta(),
                        child.magenta(),
                        value.magenta()
                    )
                }
            }
            Diff::Changed(parent, child, old_value, new_value) => {
                if child.is_empty() {
                    write!(
                        f,
                        "- {}: {}\n+ {}: {}",
                        parent.magenta(),
                        old_value.magenta(),
                        parent.green(),
                        new_value.green()
                    )
                } else {
                    write!(
                        f,
                        "- {}:\n  - {}: {}\n+ {}:\n  + {}: {}",
                        parent.magenta(),
                        child.magenta(),
                        old_value.magenta(),
                        parent.green(),
                        child.green(),
                        new_value.green()
                    )
                }
            }
            Diff::Nested(parent, diffs) => {
                write!(f, "{}:", parent)?;
                for diff in diffs {
                    write!(f, "\n  {}", diff)?;
                }
                Ok(())
            }
        }
    }
}

fn compare_hashmap(
    parent: &str,
    old: &Option<HashMap<String, String>>,
    new: &Option<HashMap<String, String>>,
) -> Vec<Diff> {
    let mut diffs = Vec::new();
    match (old, new) {
        (None, Some(new_map)) => {
            for (key, value) in new_map {
                diffs.push(Diff::Added(parent.to_string(), key.clone(), value.clone()));
            }
        }
        (Some(old_map), None) => {
            for (key, value) in old_map {
                diffs.push(Diff::Removed(
                    parent.to_string(),
                    key.clone(),
                    value.clone(),
                ));
            }
        }
        (Some(old_map), Some(new_map)) => {
            for (key, old_value) in old_map {
                match new_map.get(key) {
                    None => diffs.push(Diff::Removed(
                        parent.to_string(),
                        key.clone(),
                        old_value.clone(),
                    )),
                    Some(new_value) if new_value != old_value => {
                        diffs.push(Diff::Changed(
                            parent.to_string(),
                            key.clone(),
                            old_value.clone(),
                            new_value.clone(),
                        ));
                    }
                    _ => {}
                }
            }
            for (key, new_value) in new_map {
                if !old_map.contains_key(key) {
                    diffs.push(Diff::Added(
                        parent.to_string(),
                        key.clone(),
                        new_value.clone(),
                    ));
                }
            }
        }
        (None, None) => {}
    }
    diffs
}

fn compare_vec(parent: &str, old: &Option<Vec<String>>, new: &Option<Vec<String>>) -> Vec<Diff> {
    let mut diffs = Vec::new();
    match (old, new) {
        (None, Some(new_vec)) => {
            for item in new_vec {
                diffs.push(Diff::Added(
                    parent.to_string(),
                    "".to_string(),
                    item.clone(),
                ));
            }
        }
        (Some(old_vec), None) => {
            for item in old_vec {
                diffs.push(Diff::Removed(
                    parent.to_string(),
                    "".to_string(),
                    item.clone(),
                ));
            }
        }
        (Some(old_vec), Some(new_vec)) => {
            let old_set: std::collections::HashSet<_> = old_vec.iter().collect();
            let new_set: std::collections::HashSet<_> = new_vec.iter().collect();
            for item in new_set.difference(&old_set) {
                diffs.push(Diff::Added(
                    parent.to_string(),
                    "".to_string(),
                    item.to_string(),
                ));
            }
            for item in old_set.difference(&new_set) {
                diffs.push(Diff::Removed(
                    parent.to_string(),
                    "".to_string(),
                    item.to_string(),
                ));
            }
        }
        (None, None) => {}
    }
    diffs
}

fn compare_bool(parent: &str, old: &Option<bool>, new: &Option<bool>) -> Vec<Diff> {
    match (old, new) {
        (None, Some(new_val)) => vec![Diff::Added(
            parent.to_string(),
            "".to_string(),
            new_val.to_string(),
        )],
        (Some(old_val), None) => vec![Diff::Removed(
            parent.to_string(),
            "".to_string(),
            old_val.to_string(),
        )],
        (Some(old_val), Some(new_val)) if old_val != new_val => {
            vec![Diff::Changed(
                parent.to_string(),
                "".to_string(),
                old_val.to_string(),
                new_val.to_string(),
            )]
        }
        _ => vec![],
    }
}

fn compare_oh_my_posh(old: &Option<OhMyPosh>, new: &Option<OhMyPosh>) -> Vec<Diff> {
    let mut diffs = Vec::new();
    match (old, new) {
        (None, Some(new_omp)) => {
            if let Some(theme) = &new_omp.theme {
                diffs.push(Diff::Added(
                    "oh_my_posh".to_string(),
                    "theme".to_string(),
                    theme.clone(),
                ));
            }
        }
        (Some(old_omp), None) => {
            if let Some(theme) = &old_omp.theme {
                diffs.push(Diff::Removed(
                    "oh_my_posh".to_string(),
                    "theme".to_string(),
                    theme.clone(),
                ));
            }
        }
        (Some(old_omp), Some(new_omp)) => match (&old_omp.theme, &new_omp.theme) {
            (None, Some(new_theme)) => {
                diffs.push(Diff::Added(
                    "oh_my_posh".to_string(),
                    "theme".to_string(),
                    new_theme.clone(),
                ));
            }
            (Some(old_theme), None) => {
                diffs.push(Diff::Removed(
                    "oh_my_posh".to_string(),
                    "theme".to_string(),
                    old_theme.clone(),
                ));
            }
            (Some(old_theme), Some(new_theme)) if old_theme != new_theme => {
                diffs.push(Diff::Changed(
                    "oh_my_posh".to_string(),
                    "theme".to_string(),
                    old_theme.clone(),
                    new_theme.clone(),
                ));
            }
            _ => {}
        },
        (None, None) => {}
    }
    if !diffs.is_empty() {
        vec![Diff::Nested("oh_my_posh".to_string(), diffs)]
    } else {
        vec![]
    }
}

fn compare_ssh_config(old: &Option<SshConfig>, new: &Option<SshConfig>) -> Vec<Diff> {
    let mut diffs = Vec::new();
    match (old, new) {
        (None, Some(new_ssh)) => {
            if let Some(port) = new_ssh.port {
                diffs.push(Diff::Added(
                    "ssh".to_string(),
                    "port".to_string(),
                    port.to_string(),
                ));
            }
            if let Some(keys) = &new_ssh.authorized_keys {
                for key in keys {
                    diffs.push(Diff::Added(
                        "ssh".to_string(),
                        "authorized_keys".to_string(),
                        key.clone(),
                    ));
                }
            }
        }
        (Some(old_ssh), None) => {
            if let Some(port) = old_ssh.port {
                diffs.push(Diff::Removed(
                    "ssh".to_string(),
                    "port".to_string(),
                    port.to_string(),
                ));
            }
            if let Some(keys) = &old_ssh.authorized_keys {
                for key in keys {
                    diffs.push(Diff::Removed(
                        "ssh".to_string(),
                        "authorized_keys".to_string(),
                        key.clone(),
                    ));
                }
            }
        }
        (Some(old_ssh), Some(new_ssh)) => {
            match (old_ssh.port, new_ssh.port) {
                (None, Some(new_port)) => {
                    diffs.push(Diff::Added(
                        "ssh".to_string(),
                        "port".to_string(),
                        new_port.to_string(),
                    ));
                }
                (Some(old_port), None) => {
                    diffs.push(Diff::Removed(
                        "ssh".to_string(),
                        "port".to_string(),
                        old_port.to_string(),
                    ));
                }
                (Some(old_port), Some(new_port)) if old_port != new_port => {
                    diffs.push(Diff::Changed(
                        "ssh".to_string(),
                        "port".to_string(),
                        old_port.to_string(),
                        new_port.to_string(),
                    ));
                }
                _ => {}
            }
            let key_diffs = compare_vec("ssh", &old_ssh.authorized_keys, &new_ssh.authorized_keys);
            diffs.extend(key_diffs.into_iter().map(|diff| match diff {
                Diff::Added(_, _, value) => {
                    Diff::Added("ssh".to_string(), "authorized_keys".to_string(), value)
                }
                Diff::Removed(_, _, value) => {
                    Diff::Removed("ssh".to_string(), "authorized_keys".to_string(), value)
                }
                Diff::Changed(_, _, old_value, new_value) => Diff::Changed(
                    "ssh".to_string(),
                    "authorized_keys".to_string(),
                    old_value,
                    new_value,
                ),
                Diff::Nested(_, _) => diff, // Unreachable
            }));
        }
        (None, None) => {}
    }
    if !diffs.is_empty() {
        vec![Diff::Nested("ssh".to_string(), diffs)]
    } else {
        vec![]
    }
}

pub fn compare_configurations(old: &Configuration, new: &Configuration) -> Vec<Diff> {
    let mut diffs = Vec::new();

    diffs.extend(compare_hashmap("stow", &old.stow, &new.stow));
    diffs.extend(compare_hashmap("mise", &old.mise, &new.mise));
    diffs.extend(compare_hashmap("nix", &old.nix, &new.nix));
    diffs.extend(compare_hashmap("pkgx", &old.pkgx, &new.pkgx));
    diffs.extend(compare_hashmap("curl", &old.curl, &new.curl));
    diffs.extend(compare_hashmap("alias", &old.alias, &new.alias));

    diffs.extend(compare_vec("apt-get", &old.apt_get, &new.apt_get));

    diffs.extend(compare_bool("blesh", &old.blesh, &new.blesh));
    diffs.extend(compare_bool("zoxide", &old.zoxide, &new.zoxide));
    diffs.extend(compare_bool("tailscale", &old.tailscale, &new.tailscale));
    diffs.extend(compare_bool("neofetch", &old.neofetch, &new.neofetch));

    diffs.extend(compare_oh_my_posh(&old.oh_my_posh, &new.oh_my_posh));
    diffs.extend(compare_ssh_config(&old.ssh, &new.ssh));

    diffs
}
