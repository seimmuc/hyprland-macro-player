use super::common::{self, ActionExecutor};
use crate::data_types::{KeyCombo, Macro, MacroOptions, ModifierKey, OsInfo, SysInfo, WindowsWIDMode, WindowsWinMatchMode};
use tauri::AppHandle;
use windows::Win32::Foundation::LPARAM;

use crate::backend::keycodes::to_enigo_key;
use core::result::Result;
use enigo::Direction::{Press, Release};
use enigo::{Enigo, InputError, Key, Keyboard, Settings};
use regex::RegexBuilder;
use windows::core::{BOOL, PWSTR};
use windows::Win32::Foundation::{HANDLE, HWND};
use windows::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION};
use windows::Win32::UI::Input::KeyboardAndMouse::{SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, VIRTUAL_KEY};
use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible, SetForegroundWindow};

pub async fn macro_runner(macr: Macro, app: AppHandle, sys_info: SysInfo) {
    match sys_info.os_info {
        OsInfo::Windows => {
            let exec = WinActionExecutor::new(&macr.options);
            common::macro_runner(macr, app, exec).await;
        }
        _ => {
            panic!("OS mismatch: windows macro runner called on non-windows system");
        }
    }
}

impl WindowsWinMatchMode {
    fn to_string(&self) -> String {
        match self {
            WindowsWinMatchMode::Simple => { "simple".to_owned() }
            WindowsWinMatchMode::Regex => { "regex".to_owned() }
        }
    }
}

struct WinActionExecutor {
    window_id_mode: WindowsWIDMode,
    window_id_str: String,
    match_mode: WindowsWinMatchMode,
    auto_focus: bool,
}

impl WinActionExecutor {
    pub fn new(options: &MacroOptions) -> Self {
        let (window_id_mode, window_id_str, match_mode, auto_focus) = match options {
            MacroOptions::Windows { window_id_mode, window_id_str, match_mode, auto_focus } => {
                (*window_id_mode, window_id_str.clone(), *match_mode, *auto_focus)
            }
            _ => {
                panic!("WinActionExecutor can only execute macros with MacroOptions::Windows options");
            }
        };
        WinActionExecutor { window_id_mode, window_id_str, match_mode, auto_focus }
    }
}

impl ActionExecutor for WinActionExecutor {
    async fn press_key(&self, key: &KeyCombo) -> Result<bool, String> {
        if key.key.is_empty() {
            return Ok(true);
        }

        let current_window_matched: bool = match self.window_id_mode {
            WindowsWIDMode::None => { true }
            WindowsWIDMode::Title => {
                match get_active_window_title() {
                    Ok(title) => {
                        match_string(&title, &self.window_id_str, self.match_mode)
                    }
                    Err(_) => { false }
                }
            }
            WindowsWIDMode::Process => {
                match get_active_window_process_name() {
                    Ok(proc_name) => {
                        match_string(&proc_name, &self.window_id_str, self.match_mode)
                    }
                    Err(_) => { false }
                }
            }
        };

        if !current_window_matched {
            if self.auto_focus {
                let window: VisibleWindow = match self.window_id_mode {
                    WindowsWIDMode::Title => {
                        find_window_by_title(&self.window_id_str, self.match_mode)?
                    }
                    WindowsWIDMode::Process => {
                        find_window_by_process_name(&self.window_id_str, self.match_mode)?
                    }
                    WindowsWIDMode::None => { panic!() }
                };
                focus_window(window.hwnd)?;
            } else {
                // current window is not correct, and we should wait for the right one
                return Ok(false);
            }
        }

        execute_key_action(&key)?;
        Ok(true)
    }
}

fn match_string(str: &str, pattern: &str, mode: WindowsWinMatchMode) -> bool {
    match mode {
        WindowsWinMatchMode::Simple => {
            str.to_lowercase() == pattern.to_lowercase()
        }
        WindowsWinMatchMode::Regex => {
            let regex = RegexBuilder::new(pattern).case_insensitive(true).build();
            if let Err(_) = regex { return false; }
            // is_match matches anywhere in the string, user must use ^pattern$ to match fully
            regex.unwrap().is_match(&str)
        }
    }
}

/// Gets current window handle
fn get_active_window() -> Result<HWND, String> {
    let hwnd: HWND = unsafe { GetForegroundWindow() };
    if hwnd.is_invalid() {
        return Err("No active window".to_owned());
    }
    Ok(hwnd)
}

/// Get window title of a specific window (from window handle)
fn get_window_title(hwnd: HWND) -> Result<String, String> {
    let mut text: [u16; 512] = [0; 512];
    let len: i32 = unsafe { GetWindowTextW(hwnd, &mut text) };
    if len == 0 {
        return Err("Failed to get window title".to_string());
    }
    Ok(String::from_utf16_lossy(&text[..len as usize]))
}

/// Get the title of the current active window
fn get_active_window_title() -> Result<String, String> {
    get_window_title(get_active_window()?)
}

/// Get window's process name of a specific window (from window handle)
fn get_window_process_name(hwnd: HWND) -> Result<String, String> {
    // Get process id
    let mut process_id: u32 = 0;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut process_id)); }
    if process_id == 0 {
        return Err("Failed to get process ID".to_string());
    }

    // Get process handle
    let process_handle: HANDLE = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id) }
        .map_err(|e| format!("Failed to open process: {:?}", e))?;

    // Get full executable path of the process
    let mut buffer_raw: [u16; 1024] = [0; 1024];
    let buffer: PWSTR = PWSTR::from_raw(buffer_raw.as_mut_ptr());
    let mut buffer_size: u32 = buffer_raw.len() as u32;
    unsafe {
        QueryFullProcessImageNameW(process_handle, PROCESS_NAME_WIN32, buffer, &mut buffer_size)
            .map_err(|e| format!("Failed to query process name: {:?}", e))?;
    }
    let full_path = String::from_utf16_lossy(&buffer_raw[..buffer_size as usize]);

    // Return the executable filename from the full path
    Ok(full_path.split('\\').last().unwrap_or(&full_path).to_string())
}

/// Get the process name of the current active window
fn get_active_window_process_name() -> Result<String, String> {
    get_window_process_name(get_active_window()?)
}

/// Find all windows using a predicate function
fn find_windows<PV>(predicate_func: fn(HWND, &PV) -> bool, pv: PV) -> Result<Vec<HWND>, String> {
    struct CallbackFuncData<PV> { predicate_func: fn(HWND, &PV) -> bool, pv: PV, windows: Vec<HWND> }
    let mut data = CallbackFuncData {
        predicate_func,
        pv,
        windows: Vec::new(),
    };

    extern "system" fn enum_win_cb<PV>(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let data: &mut CallbackFuncData<PV> = unsafe { &mut *(lparam.0 as *mut CallbackFuncData<PV>) };

        if (data.predicate_func)(hwnd, &data.pv) {
            data.windows.push(hwnd);
        }
        BOOL::from(true) // Continue window enumeration
    }

    let data_ptr = &mut data as *mut CallbackFuncData<PV>;
    unsafe {
        EnumWindows(Some(enum_win_cb::<PV>), LPARAM(data_ptr as isize))
            .map_err(|e| format!("Failed to enumerate windows: {:?}", e))?;
    }

    Ok(data.windows)
}

/// Find all windows that belong to a process with a given name
fn find_windows_by_process_name(process_name: &str, match_mode: WindowsWinMatchMode) -> Result<Vec<HWND>, String> {
    fn prd(hwnd: HWND, pat: &(String, WindowsWinMatchMode)) -> bool {
        if let Ok(name) = get_window_process_name(hwnd) {
            match_string(&name, &pat.0, pat.1)
        } else { false }
    }
    let process_name = process_name.to_lowercase();
    let windows = find_windows(prd, (process_name.clone(), match_mode))?;
    Ok(windows)
}

/// Find all windows that belong to a process with a given name
fn find_windows_by_title(window_title: &str, match_mode: WindowsWinMatchMode) -> Result<Vec<HWND>, String> {
    fn prd(hwnd: HWND, pat: &(String, WindowsWinMatchMode)) -> bool {
        if let Ok(name) = get_window_title(hwnd) {
            match_string(&name, &pat.0, pat.1)
        } else { false }
    }
    let window_title = window_title.to_lowercase();
    let windows = find_windows(prd, (window_title.clone(), match_mode))?;
    Ok(windows)
}

#[derive(Clone)]
struct VisibleWindow {
    hwnd: HWND,
    title: String,
}

/// Filter a vector of windows only returning the ones that are visible and have a title
fn filter_visible_windows(window_handlers: Vec<HWND>, process_name: Option<&str>) -> Vec<VisibleWindow> {
    let mut result: Vec<VisibleWindow> = Vec::new();
    for win in window_handlers {
        let visible: bool = unsafe { IsWindowVisible(win) }.as_bool();
        if !visible { continue; }
        let title_res: Result<String, String> = get_window_title(win);
        if let Ok(title) = title_res {
            if let Some(process_name) = process_name {
                // Known "windows" that should be ignored
                if process_name == "explorer.exe" && title == "Program Manager" { continue; }
            }
            result.push(VisibleWindow { hwnd: win, title });
        }
    }
    result
}

/// Find the only visible window by process name, errors out of there are more or fewer than one
fn find_window_by_process_name(process_name: &str, match_mode: WindowsWinMatchMode) -> Result<VisibleWindow, String> {
    // Fetch and filter windows
    let windows = find_windows_by_process_name(&process_name, match_mode)?;
    let windows = filter_visible_windows(windows, match match_mode {
        WindowsWinMatchMode::Simple => { Some(&process_name) }
        _ => { None }
    });
    // Return the window if only 1 was found, otherwise return an error message
    match windows.len() {
        1 => { Ok(windows[0].clone()) }
        0 => { Err(format!("No window belonging to <{}>'{process_name}' found", match_mode.to_string())) }
        _ => { Err(format!("Process <{}>'{process_name}' has multiple windows", match_mode.to_string())) }
    }
}

/// Find the only visible window by its title, errors out of there are more or fewer than one
fn find_window_by_title(window_title: &str, match_mode: WindowsWinMatchMode) -> Result<VisibleWindow, String> {
    // Fetch and filter windows
    let windows = find_windows_by_title(window_title, match_mode)?;
    let windows = filter_visible_windows(windows, None);
    // Return the window if only 1 was found, otherwise return an error message
    match windows.len() {
        1 => { Ok(windows[0].clone()) }
        0 => { Err(format!("No window with <{}>'{window_title}' title found", match_mode.to_string())) }
        _ => { Err(format!("There are multiple windows with <{}>'{window_title}' title", match_mode.to_string())) }
    }
}

/// Focus a window
fn focus_window(hwnd: HWND) -> Result<(), String> {
    if unsafe { SetForegroundWindow(hwnd) }.as_bool() {
        Ok(())
    } else {
        Err("Failed to set foreground window".to_string())
    }
}

fn send_input_key(key: VIRTUAL_KEY) -> bool {
    let inputs = [
        // key down input
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: key,
                    wScan: 0,
                    dwFlags: KEYBD_EVENT_FLAGS(0),
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        },
        // key up input
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: key,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        },
    ];
    let events = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
    events == 2
}

fn mod_keycode(mod_key: &ModifierKey) -> Key {
    match mod_key {
        ModifierKey::Shift => { Key::Shift }
        ModifierKey::Ctrl => { Key::Control }
        ModifierKey::Alt => { Key::Alt }
        ModifierKey::Super => { Key::Meta }
    }
}

fn inp_err_to_string(err: InputError) -> String {
    match err {
        InputError::Mapping(str) => { format!("Mapping({str})") }
        InputError::Unmapping(str) => { format!("Unmapping({str})") }
        InputError::NoEmptyKeycodes => { "NoEmptyKeycodes".to_owned() }
        InputError::Simulate(str) => { format!("Simulate({str})") }
        InputError::InvalidInput(str) => { format!("InvalidInput({str})") }
    }
}

fn execute_key_action(key: &KeyCombo) -> Result<(), String> {
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    let ek = to_enigo_key(&key.key);
    if let None = ek {
        return Err(format!("Unknown key: {}", &key.key));
    }
    let ek = ek.unwrap();

    for modifier in &key.modifiers {
        enigo.key(mod_keycode(modifier), Press).map_err(inp_err_to_string)?;
    }
    enigo.key(ek, Press).map_err(inp_err_to_string)?;
    enigo.key(ek, Release).map_err(inp_err_to_string)?;
    for modifier in &key.modifiers {
        enigo.key(mod_keycode(modifier), Release).map_err(inp_err_to_string)?;
    }
    Ok(())
}
