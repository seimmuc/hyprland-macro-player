/// This file contains Rust type definitions of data sent between frontend and backend portions of the tauri app
/// These types should be changed alongside their TS counterparts in `src/lib/data_types.ts`

// SysInfo types
#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum SessionType {
    Wayland,
    X11,
    Tty,
    Other(String),
}
#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum DEInfo {
    KDE,
    Gnome,
    Hyprland,
    Other(String),
    Unknown,
}
#[derive(serde::Serialize, Clone)]
#[serde(tag = "os", rename_all = "lowercase")]
pub enum OsInfo {
    Linux {
        distro_name: String,
        desktop_environment: DEInfo,
        session_type: SessionType,
    },
    Windows,
    MacOS,
    Other,
}
#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Support {
    Supported,
    Unsupported,
    // Broken,
    Unknown,
}
#[derive(serde::Serialize, Clone)]
pub struct SysInfo {
    pub os_info: OsInfo,
    pub support: Support,
}

// Macro options
#[derive(serde::Deserialize, Copy, Clone, std::fmt::Debug)]
#[serde(rename_all = "lowercase")]
pub enum WindowsWIDMode {
    None,
    Title,
    Process,
}
#[derive(serde::Deserialize, Copy, Clone, std::fmt::Debug)]
#[serde(rename_all = "lowercase")]
pub enum WindowsWinMatchMode {
    Simple,
    Regex,
}
#[derive(serde::Deserialize, std::fmt::Debug)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum MacroOptions {
    Hyprland { window_identifier: String },
    Windows {
        /// Determines if the input should go to a specific window and how that window will be identified
        window_id_mode: WindowsWIDMode,
        /// Window identifier string
        window_id_str: String,
        /// Window identifier match mode
        match_mode: WindowsWinMatchMode,
        /// Whether the macro executor should focus the target window or wait for it to get focused
        auto_focus: bool,
    }
}

// Macro action types
#[derive(serde::Deserialize, std::fmt::Debug)]
#[serde(rename_all = "lowercase")]
pub enum ModifierKey {
    Shift,
    Ctrl,
    Alt,
    Super,
}
#[derive(serde::Deserialize, std::fmt::Debug)]
pub struct KeyCombo {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modifiers: Vec<ModifierKey>,
    pub key: String,
}
#[derive(serde::Deserialize, std::fmt::Debug)]
#[serde(tag = "action", rename_all = "lowercase")]
pub enum MacroAction {
    Sleep { duration_ms: u64 },
    Key { key: Option<KeyCombo> },
    Craft,
}

// Macro
#[derive(serde::Deserialize)]
pub struct Macro {
    pub id: u32,
    pub actions: Vec<MacroAction>,
    pub loops: u32,
    pub options: MacroOptions,
}

// Event types
#[derive(serde::Serialize, Clone)]
pub struct MacroProgress {
    /// index of the currently running action
    pub action_index: u32,
    /// current action's progress as a floating point value between 0.0 and 1.0
    pub action_progress: f64,
    /// current loop number
    pub loop_count: u32,
}
#[derive(serde::Serialize, Clone)]
#[serde(tag = "event_type", rename_all = "lowercase")]
pub enum MacroEvent {
    Update { id: u32, progress: MacroProgress },
    Running { id: u32, progress: MacroProgress },
    Paused { id: u32, progress: MacroProgress },
    Stopped { id: u32 },
    Error { id: u32, progress: MacroProgress, error: String },
}
