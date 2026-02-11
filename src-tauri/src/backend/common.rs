use super::{MacroState, SharedState};
use crate::data_types::{KeyCombo, Macro, MacroAction, MacroEvent, MacroProgress};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tokio::time::{sleep, sleep_until, Instant};


pub trait ActionExecutor {
    async fn press_key(&self, key: &KeyCombo) -> Result<(), String>;
}

pub async fn macro_runner(macr: Macro, app: AppHandle, executor: impl ActionExecutor) {
    let macro_state = app.state::<SharedState>().lock().unwrap().running_macros.read().unwrap().get(&macr.id).unwrap().clone();
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
                        let result = executor.press_key(key).await;
                        if let Err(error) = result {
                            app.emit("macro_event", MacroEvent::Error {
                                id: macr.id,
                                progress: MacroProgress {
                                    action_index: action_index as u32,
                                    action_progress: 0.5,
                                    loop_count: lup,
                                },
                                error,
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
