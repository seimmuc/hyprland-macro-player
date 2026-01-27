use std::collections::HashMap;
use std::process::Command;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tokio::time::{sleep, sleep_until, Instant};
use crate::data_types::{Macro, MacroAction, MacroEvent, MacroProgress, SysInfo};
use crate::utils::hyprland_key_mods;

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

#[derive(serde::Serialize)]
pub struct CommandResult {
    exit_code: i32,
    stdout: String,
    stderr: String
}

#[tauri::command]
pub async fn run_cmd(command: &str, _state: tauri::State<'_, SharedState>) -> Result<CommandResult, String> {
    if command.trim().is_empty() {
        return Ok(CommandResult {exit_code:0,stdout:"".to_string(),stderr:"".to_string()});
    }
    let mut parts: Vec<&str> = vec![];
    let mut remaining = command.trim();
    while remaining.len() > 0 {
        match remaining.split_once(' ') {
            Some((p, r)) => {
                if p.contains('"') {
                    parts.push(remaining);
                    break;
                }
                parts.push(p);
                remaining = r;
            },
            None => {
                parts.push(remaining);
                break;
            }
        };
    }

    println!("parts: {parts:?}");

    let mut cmd = Command::new(parts[0]);
    if parts.len() > 1 {
        cmd.args(&parts[1..]);
    }
    let c_out = match cmd.output() {
        Ok(out) => out,
        Err(err) => return Err(err.to_string())
    };
    Ok(CommandResult {
        exit_code: c_out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&c_out.stdout).to_string(),
        stderr: String::from_utf8_lossy(&c_out.stderr).to_string()
    })
}

#[tauri::command]
pub async fn long_task(ping: String, app: AppHandle, _state: tauri::State<'_, SharedState>) -> Result<String, String> {
    println!("long_task {ping}");
    tauri::async_runtime::spawn(async {
        long_task_runner(5_000, app).await;
    });
    Ok("pong".to_string())
}

async fn long_task_runner(duration_ms: u64, app: AppHandle) {
    println!("long_task_runner started");
    tokio::time::sleep(Duration::from_millis(duration_ms)).await;
    println!("long_task_runner finished");
    app.emit("long_task_complete", format!("long_task {duration_ms} is done")).unwrap();
}

#[tauri::command]
pub async fn start_macro(macr: Macro, app: AppHandle, state_mutex: tauri::State<'_, SharedState>) -> Result<MacroEvent, String> {
    let macro_id = macr.id;

    // Create and store new macro state
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
    state_mutex.lock().unwrap().running_macros.write().unwrap().insert(macro_id, macro_state.clone());

    // Spawn the macro runner
    tauri::async_runtime::spawn(async move {
        macro_runner(macr, app.clone()).await;

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

async fn macro_handle_pause_and_stop(state: &Arc<Mutex<MacroState>>, sleep_millis: u64) -> (bool, Option<Duration>) {
    let dur: Option<Duration> = if state.lock().unwrap().paused {
        let pause_start = Instant::now();
        loop {
            sleep(Duration::from_millis(sleep_millis)).await;
            if !state.lock().unwrap().paused {
                break;
            }
        }
        Some(pause_start.elapsed())
    } else {
        None
    };
    (state.lock().unwrap().should_stop, dur)
}

async fn macro_runner(macr: Macro, app: AppHandle) {
    let macro_state = app.state::<SharedState>().lock().unwrap().running_macros.read().unwrap().get(&macr.id).unwrap().clone();

    let update_interval = Duration::from_millis(25);
    let mut next_update = Instant::now();

    // Iterate through actions
    'runner: for (action_index, action) in macr.actions.iter().enumerate() {
        // Update progress
        macro_state.lock().unwrap().progress = MacroProgress {
            action_index: action_index as u32,
            action_progress: 0.0,
            loop_count: 0,
        };

        // Wait if paused and check if we should stop
        if macro_handle_pause_and_stop(&macro_state, 500).await.0 {
            break 'runner;
        }

        match action {
            MacroAction::Sleep { duration_ms } => {
                let sleep_duration = Duration::from_millis(*duration_ms);
                let mut action_end = Instant::now() + sleep_duration;
                let duration_ms = *duration_ms as f64;
                let mut now: Instant;

                // Sleep with progress updates
                while { now = Instant::now(); now < action_end } {
                    let remaining = action_end.duration_since(now).as_millis() as f64;
                    let action_progress = ((duration_ms - remaining) / duration_ms).min(1.0).max(0.0);
                    let progress = MacroProgress {
                        action_index: action_index as u32,
                        action_progress,
                        loop_count: 0,
                    };
                    macro_state.lock().unwrap().progress = progress.clone();

                    // Send progress update
                    if next_update <= Instant::now() {
                        app.emit(
                            "macro_event",
                            MacroEvent::Update {
                                id: macr.id,
                                progress,
                            },
                        ).unwrap();
                        next_update = Instant::now() + update_interval;
                    }

                    // Handle stopping and pausing
                    let (stopped, paused) = macro_handle_pause_and_stop(&macro_state, 50).await;
                    if stopped {
                        break 'runner;
                    }
                    if let Some(paused_for) = paused {
                        action_end = action_end + paused_for;
                    }

                    // Sleep until the next interruption
                    sleep_until(action_end.min(next_update)).await;
                }
            }
            MacroAction::Key { key } => {
                let progress = MacroProgress {
                    action_index: action_index as u32,
                    action_progress: 0.0,
                    loop_count: 0,
                };
                macro_state.lock().unwrap().progress = progress.clone();
                app.emit("macro_event", MacroEvent::Update {
                    id: macr.id,
                    progress,
                }).unwrap();
                if let Some(key) = key {
                    // window identifier is hard-coded for now TODO add as a setting
                    let window_id = "";
                    // let window_id = "class:org.kde.kate";
                    // let window_id = "class:thunar";
                    let mod_str = hyprland_key_mods(&key.modifiers);
                    let send_sh_param = format!("{mod_str},{},{window_id}", key.key);

                    let mut cmd = Command::new("hyprctl");
                    cmd.arg("dispatch").arg("sendshortcut").arg(&send_sh_param);
                    let cmd_out = cmd.output().unwrap();
                    let std_out = String::from_utf8_lossy(&cmd_out.stdout).trim().to_string();
                    if std_out != "ok" {
                        app.emit("macro_event", MacroEvent::Error {
                            id: macr.id,
                            progress: MacroProgress {
                                action_index: action_index as u32,
                                action_progress: 1.0,
                                loop_count: 0,
                            },
                            error: format!("Hyprland dispatcher error: {std_out}"),
                        }).unwrap();
                        break 'runner;
                    }
                }
            }
            // MacroAction::Craft => {}
            other => {
                println!("Executing {other:?} action");
                // Placeholder for future implementation TODO
                app.emit("macro_event", MacroEvent::Update {
                    id: macr.id,
                    progress: MacroProgress {
                        action_index: action_index as u32,
                        action_progress: 0.0,
                        loop_count: 0,
                    },
                }).unwrap();
            }
        }
    }

    // Runner loop exited, send stop event
    app.emit("macro_event", MacroEvent::Stopped { id: macr.id }).unwrap();
}
