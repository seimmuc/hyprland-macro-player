use std::env;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tokio::time::{sleep, sleep_until, Instant};
use crate::backend::{MacroState, SharedState};
use crate::data_types::{DEInfo, Macro, MacroAction, MacroEvent, MacroOptions, MacroProgress, OsInfo, SysInfo};
use crate::utils::hyprland_key_mods;

#[cfg(target_os = "linux")]
pub async fn macro_runner(macr: Macro, app: AppHandle, sys_info: SysInfo) {
    match sys_info.os_info {
        OsInfo::Linux { desktop_environment, .. } => {
            if matches!(desktop_environment,  DEInfo::Hyprland) {
                hypr_macro_runner(macr, app).await;
            } else {
                panic!("Unsupported desktop environment");
            }
        }
        _ => {
            panic!("OS mismatch: linux macro runner called on non-linux system");
        }
    }
}

#[cfg(target_os = "linux")]
async fn hypr_macro_runner(macr: Macro, app: AppHandle) {
    let macro_state = app.state::<SharedState>().lock().unwrap().running_macros.read().unwrap().get(&macr.id).unwrap().clone();
    let [window_identifier] = match macr.options {
        MacroOptions::Hyprland { window_identifier } => {
            [window_identifier]
        }
        // _ => {
        //     panic!("hypr_macro_runner can only execute macros with MacroOptions::Hyprland options")
        // }
    };
    let his = env::var("HYPRLAND_INSTANCE_SIGNATURE").expect("HYPRLAND_INSTANCE_SIGNATURE is not set");
    let xdg_run_dir = env::var("XDG_RUNTIME_DIR").expect("XDG_RUNTIME_DIR is not set");
    let hypr_socket_path = format!("{xdg_run_dir}/hypr/{his}/.socket.sock");

    let update_interval = Duration::from_millis(25);
    let mut next_update = Instant::now();

    // Iterate through actions
    'runner: for lup in 0..macr.loops {
        for (action_index, action) in macr.actions.iter().enumerate() {
            // Update progress
            macro_state.lock().unwrap().progress = MacroProgress {
                action_index: action_index as u32,
                action_progress: 0.0,
                loop_count: lup,
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
                            loop_count: lup,
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
                        loop_count: lup,
                    };
                    macro_state.lock().unwrap().progress = progress.clone();
                    app.emit("macro_event", MacroEvent::Update {
                        id: macr.id,
                        progress,
                    }).unwrap();
                    if let Some(key) = key {
                        let window_id = &window_identifier;
                        let mod_str = hyprland_key_mods(&key.modifiers);
                        let send_sh_param = format!("{mod_str},{},{window_id}", key.key);

                        let mut stream = UnixStream::connect(&hypr_socket_path).unwrap();
                        stream.write_all(format!("dispatch sendshortcut {send_sh_param}").as_bytes()).unwrap();
                        stream.flush().unwrap();
                        drop(stream);
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
                            loop_count: lup,
                        },
                    }).unwrap();
                }
            }
        }
    }

    // Runner loop exited, send stop event
    app.emit("macro_event", MacroEvent::Stopped { id: macr.id }).unwrap();
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
