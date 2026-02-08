use crate::data_types::{Macro, MacroEvent, MacroProgress, SysInfo};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use tauri::{AppHandle, Manager};

// Platform-specific imports
#[cfg(target_os = "linux")] mod linux;
#[cfg(target_os = "linux")] use self::linux::macro_runner;
#[cfg(target_os = "windows")] mod windows;
#[cfg(target_os = "windows")] use self::windows::macro_runner;

pub struct MacroState {
    paused: bool,
    should_stop: bool,
    progress: MacroProgress,
}

pub struct AppState {
    counter: i32,
    sys_info: SysInfo,
    // TODO reconsider using Arc and possibly inner Mutex?
    running_macros: Arc<RwLock<HashMap<u32, Arc<Mutex<MacroState>>>>>,
}

impl AppState {
    pub fn new(sys_info: SysInfo) -> Self {
        Self {
            counter: 0,
            sys_info,
            running_macros: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

pub type SharedState = Mutex<AppState>;

#[tauri::command]
pub fn system_info<'a>(state: tauri::State<SharedState>) -> Option<SysInfo> {
    println!("system_info");
    match state.lock() {
        Ok(app_state) => Some(app_state.sys_info.clone()),
        Err(_err) => None,
    }
}

#[tauri::command]
pub fn check_count(state: tauri::State<SharedState>) -> i32 {
    println!("Check count");
    match state.lock() {
        Ok(app_state) => {app_state.counter}
        Err(_err) => {-1}
    }
}
#[tauri::command]
pub fn increment_count(state: tauri::State<SharedState>) -> i32 {
    println!("Increment count");
    match state.lock() {
        Ok(mut app_state) => {
            app_state.counter += 1;
            app_state.counter
        }
        Err(_err) => {-1}
    }
}

#[tauri::command]
pub async fn start_macro(macr: Macro, app: AppHandle, state_mutex: tauri::State<'_, SharedState>) -> Result<MacroEvent, String> {
    let macro_id = macr.id;

    // Create new macro state
    let progress = MacroProgress {
        action_index: 0,
        action_progress: 0.0,
        loop_count: 0,
    };
    let macro_state = Arc::new(Mutex::new(MacroState {
        paused: false,
        should_stop: false,
        progress: progress.clone(),
    }));

    // Store macro state and pull sys_info
    let sys_info = {
        let state = state_mutex.lock().unwrap();
        state.running_macros.write().unwrap().insert(macro_id, macro_state.clone());
        state.sys_info.clone()
    };

    // Spawn the macro runner
    tauri::async_runtime::spawn(async move {
        macro_runner(macr, app.clone(), sys_info).await;

        let state_mutex = app.state::<SharedState>();
        let app_state = state_mutex.lock().unwrap();
        app_state.running_macros.write().unwrap().remove(&macro_id);
    });

    Ok(MacroEvent::Running {
        id: macro_id,
        progress,
    })
}

#[tauri::command]
pub async fn pause_macro(id: u32, state_mutex: tauri::State<'_, SharedState>) -> Result<MacroEvent, String> {
    let app_state = state_mutex.lock().unwrap();
    let macros = app_state.running_macros.read().unwrap();
    if let Some(macro_state) = macros.get(&id) {
        let progress: MacroProgress = {
            let mut m_state = macro_state.lock().unwrap();
            m_state.paused = true;
            m_state.progress.clone()
        };
        Ok(MacroEvent::Paused { id, progress })
    } else {
        Err(format!("Macro with id {} not found", id))
    }
}

#[tauri::command]
pub async fn resume_macro(id: u32, state_mutex: tauri::State<'_, SharedState>) -> Result<MacroEvent, String> {
    let app_state = state_mutex.lock().unwrap();
    let macros = app_state.running_macros.read().unwrap();

    if let Some(macro_state) = macros.get(&id) {
        let progress: MacroProgress = {
            let mut m_state = macro_state.lock().unwrap();
            m_state.paused = false;
            m_state.progress.clone()
        };
        Ok(MacroEvent::Running { id, progress })
    } else {
        Err(format!("Macro with id {} not found", id))
    }
}

#[tauri::command]
pub async fn stop_macro(id: u32, state_mutex: tauri::State<'_, SharedState>) -> Result<MacroEvent, String> {
    let app_state = state_mutex.lock().unwrap();
    let macros = app_state.running_macros.read().unwrap();

    if let Some(macro_state) = macros.get(&id) {
        let mut state = macro_state.lock().unwrap();
        state.should_stop = true;
        state.paused = false;
        Ok(MacroEvent::Stopped { id })
    } else {
        Err(format!("Macro with id {} not found", id))
    }
}
