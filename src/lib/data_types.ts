/**
 * This file contains TS type definitions of data sent between frontend and backend portions of the tauri app
 * These types should be changed alongside their Rust counterparts in `src-tauri/src/data_types.rs`
 */


/** Loose TS version of Rust struct `data_types::SysInfo` */
export interface SysInfo {
  os_info: {
    os: 'linux' | 'windows' | 'macos' | 'other';
    distro_name?: string;
    desktop_environment?: 'kde' | 'gnome' | 'hyprland' | { other: string } | 'unknown';
    session_type?: 'wayland' | 'x11' | 'tty' | { other: string };
  };
  support: 'supported' | 'unsupported' | 'broken' | 'unknown';
}

export type ActionType = 'sleep' | 'key' | 'craft';
export type PlayerEventType = 'update' | 'running' | 'paused' | 'stopped' | 'error';


// Macro and Action types
export type ModifierKey = 'shift' | 'ctrl' | 'alt' | 'super';
export interface KeyCombo {
  modifiers?: ModifierKey[];
  key: string;
}
interface BaseAction {
  actionId: number;
  action: ActionType;
}
export interface SleepAction extends BaseAction {
  action: 'sleep';
  duration_ms: number;
}
export interface KeyAction extends BaseAction {
  action: 'key';
  key: KeyCombo | undefined | null;
}
export interface CraftAction extends BaseAction {
  action: 'craft';
}
export type Action = SleepAction | KeyAction | CraftAction;
/** TS version of Rust struct `data_types::Macro` */
export interface RustMacro {
  id: number;   // TODO move ID assignment to Rust
  actions: Array<Omit<SleepAction, 'actionId'> | Omit<KeyAction, 'actionId'> | Omit<CraftAction, 'actionId'>>;
  loops: number;
}


// Commands
// export type PlayerCommandType = 'execute' | 'pause' | 'resume' | 'stop';
// interface BasePlayerCommand {
//   command: PlayerCommandType;
// }
// interface ExecuteCommand extends BasePlayerCommand {
//   command: 'execute';
//   actions: Action[];
// }
// interface PauseCommand extends BasePlayerCommand {
//   command: 'pause';
// }
// interface ResumeCommand extends BasePlayerCommand {
//   command: 'resume';
// }
// interface StopCommand extends BasePlayerCommand {
//   command: 'stop';
// }
// export type PlayerCommand = ExecuteCommand | PauseCommand | ResumeCommand | StopCommand;


// Event types
export interface ProgressInfo {
  action_index: number;     // integer in (0, len(actions))
  action_progress: number;  // float in (0.0, 1.0)
  loop_count: number;       // integer in (0, loops)
}
interface BaseMacroEvent {
  event_type: PlayerEventType;
  /** Macro id */
  id: number;
}
interface BaseUpdateEvent extends BaseMacroEvent {
  event_type: 'update' | 'running' | 'paused' | 'error';
  progress: ProgressInfo;
}
export interface UpdateEvent extends BaseUpdateEvent {
  event_type: 'update';
}
export interface RunningEvent extends BaseUpdateEvent {
  event_type: 'running';
}
export interface PausedEvent extends BaseUpdateEvent {
  event_type: 'paused';
}
export interface StoppedEvent extends BaseMacroEvent {
  event_type: 'stopped';
}
export interface ErrorEvent extends BaseUpdateEvent {
  event_type: 'error';
  error: string;
}
export type MacroEvent = UpdateEvent | RunningEvent | PausedEvent | StoppedEvent | ErrorEvent;
