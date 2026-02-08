use tauri::AppHandle;
use super::common::{self, ActionExecutor};
use crate::data_types::{KeyCombo, Macro, OsInfo, SysInfo};

pub async fn macro_runner(macr: Macro, app: AppHandle, sys_info: SysInfo) {
    match sys_info.os_info {
        OsInfo::Windows => {
            let exec = WinActionExecutor::new();
            common::macro_runner(macr, app, exec).await;
        }
        _ => {
            panic!("OS mismatch: windows macro runner called on non-windows system");
        }
    }
}

struct WinActionExecutor {}

impl WinActionExecutor {
    pub fn new() -> Self {
        WinActionExecutor {}
    }
}

impl ActionExecutor for WinActionExecutor {
    fn press_key(&self, key: &KeyCombo) -> Result<(), String> {
        todo!();
    }
}
