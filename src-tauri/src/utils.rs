use crate::data_types::{DEInfo, ModifierKey, SessionType};
use anyhow::Result;
use regex::RegexBuilder;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn read_distro_name() -> Result<String> {
    // Init regex
    // If we switch to enums later, we should use the ID field instead
    let reg_pattern = RegexBuilder::new(r#"^PRETTY_NAME="(.+)"$"#)
        .case_insensitive(true).build()?;

    // File reader
    let file = File::open("/etc/os-release")?;
    let buffered = BufReader::new(file);

    // Read every line
    for line in buffered.lines() {
        if let Some(captures) = reg_pattern.captures(&(line?)) {
            if let Some(d_name) = captures.get(1) {
                return Ok(String::from(d_name.as_str()));
            }
        }
    }

    Err(anyhow::anyhow!("Distro name was not found"))
}

pub fn get_desktop_environment() -> DEInfo {
    let de_name = env::var("XDG_CURRENT_DESKTOP").unwrap_or(String::new());
    match de_name.to_lowercase().trim() {
        "" => DEInfo::Unknown,
        "kde" => DEInfo::KDE,
        "gnome" => DEInfo::Gnome,
        "hyprland" => DEInfo::Hyprland,
        other => DEInfo::Other(other.into()),
    }
}

pub fn get_session_type() -> SessionType {
    let ses = env::var("XDG_SESSION_TYPE").unwrap_or(String::new());
    match ses.to_lowercase().trim() {
        "wayland" => SessionType::Wayland,
        "x11" => SessionType::X11,
        "tty" => SessionType::Tty,
        other => SessionType::Other(other.into()),
    }
}

pub fn hyprland_key_mods(mods: &Vec<ModifierKey>) -> String {
    mods.iter().map(|m| match m {
        ModifierKey::Shift => {"SHIFT"}
        ModifierKey::Ctrl => {"CTRL"}
        ModifierKey::Alt => {"ALT"}
        ModifierKey::Super => {"SUPER"}
    }).collect::<Vec<&str>>().join("_")
}
