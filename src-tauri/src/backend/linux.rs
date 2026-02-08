use super::common::{self, ActionExecutor};
use crate::data_types::{DEInfo, KeyCombo, Macro, MacroOptions, OsInfo, SysInfo};
use crate::utils::hyprland_key_mods;
use std::env;
use std::io::Write;
use std::os::unix::net::UnixStream;
use tauri::AppHandle;

pub async fn macro_runner(macr: Macro, app: AppHandle, sys_info: SysInfo) {
    match sys_info.os_info {
        OsInfo::Linux { desktop_environment, .. } => {
            if matches!(desktop_environment,  DEInfo::Hyprland) {
                let exec = HyprActionExecutor::new(&macr.options);
                common::macro_runner(macr, app, exec).await;
            } else {
                unimplemented!("Unsupported desktop environment");
            }
        }
        _ => {
            panic!("OS mismatch: linux macro runner called on non-linux system");
        }
    }
}

struct HyprActionExecutor {
    window_identifier: String,
    hypr_socket_path: String,
}
impl HyprActionExecutor {
    fn new(options: &MacroOptions) -> Self {
        let [window_identifier] = match options {
            MacroOptions::Hyprland { window_identifier } => {
                [window_identifier]
            }
            // _ => {
            //     panic!("hypr_macro_runner can only execute macros with MacroOptions::Hyprland options")
            // }
        };
        let his = env::var("HYPRLAND_INSTANCE_SIGNATURE").expect("HYPRLAND_INSTANCE_SIGNATURE is not set");
        let xdg_run_dir = env::var("XDG_RUNTIME_DIR").expect("XDG_RUNTIME_DIR is not set");
        HyprActionExecutor {
            window_identifier: window_identifier.to_owned(),
            hypr_socket_path: format!("{xdg_run_dir}/hypr/{his}/.socket.sock"),
        }
    }
}
impl ActionExecutor for HyprActionExecutor {
    fn press_key(&self, key: &KeyCombo) -> Result<(), String> {
        let mod_str = hyprland_key_mods(&key.modifiers);
        let send_sh_param = format!("{mod_str},{},{}", key.key, &self.window_identifier);

        let mut stream = UnixStream::connect(&self.hypr_socket_path).unwrap();
        stream.write_all(format!("dispatch sendshortcut {send_sh_param}").as_bytes()).unwrap();
        stream.flush().unwrap();
        Ok(())
    }
}
