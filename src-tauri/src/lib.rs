use std::sync::Mutex;
use tauri::RunEvent;

mod backend;
use crate::backend::{SharedState, AppState, check_count, increment_count, run_cmd, long_task, system_info, start_macro, pause_macro, resume_macro, stop_macro};
mod utils;
use crate::utils::{get_desktop_environment, get_session_type, read_distro_name};
mod data_types;
use crate::data_types::{DEInfo, OsInfo, Support, SysInfo};


// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
async fn greet(name: String) -> String {
    let os_info: String = format!("Your OS is '{}'", std::env::consts::OS);
    format!("Hello, {}! You've been greeted from Rust! {}", name, os_info)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    println!("Hello, world!");
    let sys_info = get_system_info();
    let state = AppState::new(sys_info);

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Mutex::new(state) as SharedState)
        .invoke_handler(tauri::generate_handler![greet, check_count, increment_count, run_cmd, long_task, system_info, start_macro, pause_macro, resume_macro, stop_macro])
        .build(tauri::generate_context!())
        .expect("error while running tauri application");
    app.run(|_app_handle, event| {
        match event {
            RunEvent::ExitRequested { .. } => {
                // Async background tasks should stop automatically, no actions needed here
                println!("exiting");
            }
            _ => {}
        }
    });
}

fn get_system_info() -> SysInfo {
    let os = std::env::consts::OS;
    let os_info: OsInfo = match os {
        "linux" => {
            let distro_name = read_distro_name().unwrap_or(String::from("Unknown"));
            let desktop_environment = get_desktop_environment();
            let session_type = get_session_type();
            OsInfo::Linux {
                distro_name,
                desktop_environment,
                session_type
            }
        },
        "windows" => OsInfo::Windows,
        "macos" => OsInfo::MacOS,
        _ => OsInfo::Other,
    };
    match &os_info {
        OsInfo::Linux { desktop_environment, distro_name: _distro_name, session_type: _session_type } => {
            match desktop_environment {
                DEInfo::Hyprland => SysInfo {os_info, support: Support::Supported},
                DEInfo::KDE => SysInfo { os_info, support: Support::Unsupported },
                DEInfo::Gnome => SysInfo { os_info, support: Support::Unsupported },
                DEInfo::Other(_) => SysInfo { os_info, support: Support::Unknown },
                DEInfo::Unknown => SysInfo { os_info, support: Support::Unknown },
            }
        },
        _ => SysInfo { os_info, support: Support::Unknown },
    }

}
