//! Throwaway spike for EasyDeck issue A-1: focus and input. Not product code.
//!
//! Run mode: a window with WS_EX_NOACTIVATE; every tap is logged with the foreground window
//! before and after. `--selftest` is for CI only (question 6) and never for a real desktop.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde_json::{json, Value};
use std::ffi::c_void;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{PhysicalPosition, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use windows::Win32::Foundation::{CloseHandle, GetLastError, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
use windows::Win32::System::Threading::{
    GetCurrentProcessId, OpenProcess, OpenProcessToken, QueryFullProcessImageNameW,
    PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetCapture, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
    KEYEVENTF_SCANCODE, VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetClassNameW, GetCursorInfo, GetForegroundWindow, GetGUIThreadInfo,
    GetWindowLongPtrW, GetWindowRect, GetWindowTextW, GetWindowThreadProcessId, IsIconic,
    SetForegroundWindow, SetWindowLongPtrW, SetWindowsHookExW, CURSORINFO, GUITHREADINFO,
    GWL_EXSTYLE, HHOOK, KBDLLHOOKSTRUCT, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN,
    WM_SYSKEYUP, WS_EX_NOACTIVATE,
};

/// Scan codes for the keys 1, 2, 3 on a US layout (set 1).
const SCAN_1_2_3: [u16; 3] = [0x02, 0x03, 0x04];

static IS_SELFTEST: AtomicBool = AtomicBool::new(false);
static PAGE_READY: AtomicBool = AtomicBool::new(false);
static JS_KEYS: Mutex<Vec<Value>> = Mutex::new(Vec::new());
static LL_KEYS: Mutex<Vec<Value>> = Mutex::new(Vec::new());
static STATE: Mutex<Flags> = Mutex::new(Flags {
    noactivate: false,
    topmost: false,
});

struct Flags {
    noactivate: bool,
    topmost: bool,
}

fn flags() -> (bool, bool) {
    STATE
        .lock()
        .map(|s| (s.noactivate, s.topmost))
        .unwrap_or((false, false))
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

fn log_path() -> PathBuf {
    std::env::temp_dir().join("easydeck-a1-spike.jsonl")
}

fn append_log(v: &Value) {
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path())
    {
        let _ = writeln!(f, "{v}");
    }
}

fn arg_value(name: &str) -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1).cloned())
}

fn has_flag(name: &str) -> bool {
    std::env::args().any(|a| a == name)
}

fn hwnd_of(w: &WebviewWindow) -> Option<HWND> {
    // tauri 2.12 uses the same `windows` version as this crate, so the types match.
    w.hwnd().ok()
}

fn hwnd_str(h: HWND) -> String {
    format!("{:#x}", h.0 as usize)
}

fn wide_to_string(buf: &[u16], n: usize) -> String {
    String::from_utf16_lossy(&buf[..n.min(buf.len())])
}

// ---------- window style ----------

fn set_noactivate(hwnd: HWND, on: bool) {
    unsafe {
        let cur = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let bit = WS_EX_NOACTIVATE.0 as isize;
        let next = if on { cur | bit } else { cur & !bit };
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, next);
    }
    if let Ok(mut s) = STATE.lock() {
        s.noactivate = on;
    }
}

// ---------- foreground snapshot ----------

fn fg_snapshot() -> Value {
    unsafe {
        let h = GetForegroundWindow();
        if h.0.is_null() {
            return json!({ "hwnd": null });
        }
        let mut pid = 0u32;
        let tid = GetWindowThreadProcessId(h, Some(&mut pid));
        let mut buf = [0u16; 512];
        let n = GetWindowTextW(h, &mut buf);
        let title = wide_to_string(&buf, n.max(0) as usize);
        let n = GetClassNameW(h, &mut buf);
        let class = wide_to_string(&buf, n.max(0) as usize);

        let mut process = Value::Null;
        let mut elevated = Value::Null;
        if let Ok(hp) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            let mut len = buf.len() as u32;
            if QueryFullProcessImageNameW(
                hp,
                PROCESS_NAME_WIN32,
                windows::core::PWSTR(buf.as_mut_ptr()),
                &mut len,
            )
            .is_ok()
            {
                let full = wide_to_string(&buf, len as usize);
                process = json!(full.rsplit('\\').next().unwrap_or(&full));
            }
            let mut tok = Default::default();
            if OpenProcessToken(hp, TOKEN_QUERY, &mut tok).is_ok() {
                let mut te = TOKEN_ELEVATION::default();
                let mut ret = 0u32;
                if GetTokenInformation(
                    tok,
                    TokenElevation,
                    Some(&mut te as *mut _ as *mut c_void),
                    std::mem::size_of::<TOKEN_ELEVATION>() as u32,
                    &mut ret,
                )
                .is_ok()
                {
                    elevated = json!(te.TokenIsElevated != 0);
                }
                let _ = CloseHandle(tok);
            }
            let _ = CloseHandle(hp);
        }

        let mut r = RECT::default();
        let rect = if GetWindowRect(h, &mut r).is_ok() {
            json!([r.left, r.top, r.right, r.bottom])
        } else {
            Value::Null
        };
        let mut mi = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        let hm = MonitorFromWindow(h, MONITOR_DEFAULTTONEAREST);
        let fills_monitor = if GetMonitorInfoW(hm, &mut mi).as_bool() {
            let m = mi.rcMonitor;
            json!(r.left <= m.left && r.top <= m.top && r.right >= m.right && r.bottom >= m.bottom)
        } else {
            Value::Null
        };

        let mut gi = GUITHREADINFO {
            cbSize: std::mem::size_of::<GUITHREADINFO>() as u32,
            ..Default::default()
        };
        let gui = if GetGUIThreadInfo(tid, &mut gi).is_ok() {
            json!({
                "active": hwnd_str(gi.hwndActive),
                "focus": hwnd_str(gi.hwndFocus),
                "capture": hwnd_str(gi.hwndCapture),
            })
        } else {
            Value::Null
        };

        json!({
            "hwnd": hwnd_str(h),
            "title": title,
            "class": class,
            "process": process,
            "pid": pid,
            "elevated": elevated,
            "is_ours": pid == GetCurrentProcessId(),
            "minimized": IsIconic(h).as_bool(),
            "rect": rect,
            "fills_monitor": fills_monitor,
            "gui_thread": gui,
        })
    }
}

fn cursor_snapshot() -> Value {
    unsafe {
        let mut ci = CURSORINFO {
            cbSize: std::mem::size_of::<CURSORINFO>() as u32,
            ..Default::default()
        };
        let info = if GetCursorInfo(&mut ci).is_ok() {
            // flags: 1 = CURSOR_SHOWING, 2 = CURSOR_SUPPRESSED (touch input hides the cursor)
            json!({ "flags": ci.flags.0, "x": ci.ptScreenPos.x, "y": ci.ptScreenPos.y })
        } else {
            Value::Null
        };
        json!({ "cursor": info, "own_thread_capture": hwnd_str(GetCapture()) })
    }
}

// ---------- SendInput ----------

/// Sends each scan code as down then up, one `SendInput` call per event, with small gaps.
fn send_scan_keys(codes: &[u16]) -> Value {
    let mut events = Vec::new();
    for &code in codes {
        for up in [false, true] {
            let flags = if up {
                KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP
            } else {
                KEYEVENTF_SCANCODE
            };
            let input = INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VIRTUAL_KEY(0),
                        wScan: code,
                        dwFlags: flags,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };
            let sent = unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };
            let err = if sent == 0 {
                unsafe { GetLastError().0 }
            } else {
                0
            };
            events.push(json!({ "scan": code, "up": up, "sent": sent, "last_error": err }));
            std::thread::sleep(Duration::from_millis(30));
        }
    }
    let ok = events.iter().all(|e| e["sent"] == 1);
    json!({ "all_sent": ok, "events": events })
}

// ---------- commands ----------

#[tauri::command]
fn get_info(window: WebviewWindow) -> Value {
    let monitors: Vec<Value> = window
        .available_monitors()
        .unwrap_or_default()
        .iter()
        .enumerate()
        .map(|(i, m)| {
            json!({ "index": i, "name": m.name(), "x": m.position().x, "y": m.position().y,
                    "w": m.size().width, "h": m.size().height, "scale": m.scale_factor() })
        })
        .collect();
    let (noactivate, topmost) = flags();
    json!({
        "pid": std::process::id(),
        "noactivate": noactivate,
        "topmost": topmost,
        "selftest": IS_SELFTEST.load(Ordering::SeqCst),
        "log_path": log_path().display().to_string(),
        "monitors": monitors,
    })
}

/// One tap: snapshot, act, wait about 200 ms, snapshot, log. Runs off the main thread.
#[tauri::command]
async fn tap(window: WebviewWindow, pointer_type: String, action: String) -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let t0 = now_ms();
        let before = fg_snapshot();
        let cursor_before = cursor_snapshot();
        let mut detail = Value::Null;
        match action.as_str() {
            "keys" => detail = send_scan_keys(&SCAN_1_2_3),
            "keys_delayed" => {
                std::thread::sleep(Duration::from_secs(3));
                detail = send_scan_keys(&SCAN_1_2_3);
            }
            "toggle_edit" => {
                if let Some(h) = hwnd_of(&window) {
                    set_noactivate(h, !flags().0);
                }
            }
            "toggle_topmost" => {
                let on = flags().1;
                let _ = window.set_always_on_top(!on);
                if let Ok(mut s) = STATE.lock() {
                    s.topmost = !on;
                }
            }
            _ => {}
        }
        std::thread::sleep(Duration::from_millis(200));
        let after = fg_snapshot();
        let cursor_after = cursor_snapshot();
        let (noactivate, topmost) = flags();
        let rec = json!({
            "t_ms": t0,
            "action": action,
            "pointer_type": pointer_type,
            "noactivate": noactivate,
            "topmost": topmost,
            "fg_before": before,
            "fg_after": after,
            "cursor_before": cursor_before,
            "cursor_after": cursor_after,
            "send_input": detail,
        });
        append_log(&rec);
        rec
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn selftest_ready() {
    PAGE_READY.store(true, Ordering::SeqCst);
}

#[tauri::command]
fn report_key(kind: String, code: String, key: String) {
    if let Ok(mut v) = JS_KEYS.lock() {
        v.push(json!({ "t_ms": now_ms(), "type": kind, "code": code, "key": key }));
    }
}

// ---------- selftest (CI only) ----------

unsafe extern "system" fn ll_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let k = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
        let kind = match wparam.0 as u32 {
            WM_KEYDOWN | WM_SYSKEYDOWN => "down",
            WM_KEYUP | WM_SYSKEYUP => "up",
            _ => "other",
        };
        if let Ok(mut v) = LL_KEYS.lock() {
            v.push(json!({ "type": kind, "scan": k.scanCode, "vk": k.vkCode,
                           "injected": k.flags.0 & 0x10 != 0 }));
        }
    }
    CallNextHookEx(Some(HHOOK::default()), code, wparam, lparam)
}

fn write_selftest_result(out: &Path, extra: Value) -> bool {
    let js = JS_KEYS.lock().map(|v| v.clone()).unwrap_or_default();
    let ll = LL_KEYS.lock().map(|v| v.clone()).unwrap_or_default();
    let got = |c: &str| js.iter().any(|e| e["type"] == "down" && e["code"] == c);
    let reached = got("Digit1") && got("Digit2") && got("Digit3");
    let result = json!({
        "question": "A-1 Q6: does SendInput reach a window of our own process",
        "keys_reached_webview": reached,
        "js_keys": js,
        "ll_hook_keys": ll,
        "details": extra,
    });
    let _ = std::fs::write(
        out,
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    );
    reached
}

fn run_selftest(window: WebviewWindow) {
    let out = PathBuf::from(arg_value("--out").unwrap_or_else(|| {
        std::env::temp_dir()
            .join("easydeck-a1-selftest.json")
            .display()
            .to_string()
    }));
    // Watchdog: a result file and an exit code within about 20 s, whatever happens.
    let wd_out = out.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(18));
        write_selftest_result(&wd_out, json!({ "error": "timeout" }));
        std::process::exit(1);
    });
    std::thread::spawn(move || {
        let start = std::time::Instant::now();
        while !PAGE_READY.load(Ordering::SeqCst) && start.elapsed() < Duration::from_secs(12) {
            std::thread::sleep(Duration::from_millis(100));
        }
        let page_ready = PAGE_READY.load(Ordering::SeqCst);
        let hwnd = hwnd_of(&window);
        let _ = window.set_focus();
        let fg_set = hwnd
            .map(|h| unsafe { SetForegroundWindow(h).as_bool() })
            .unwrap_or(false);
        std::thread::sleep(Duration::from_millis(700));
        let fg = fg_snapshot();
        let sent = send_scan_keys(&SCAN_1_2_3);
        std::thread::sleep(Duration::from_millis(1000));
        let ok = write_selftest_result(
            &out,
            json!({ "page_ready": page_ready, "set_foreground_ok": fg_set,
                    "foreground_before_send": fg, "send_input": sent }),
        );
        std::process::exit(if ok { 0 } else { 1 });
    });
}

// ---------- main ----------

fn main() {
    let selftest = has_flag("--selftest");
    IS_SELFTEST.store(selftest, Ordering::SeqCst);
    let monitor_idx: Option<usize> = arg_value("--monitor").and_then(|s| s.parse().ok());
    let want_topmost = has_flag("--topmost");
    let want_noactivate = !has_flag("--no-noactivate") && !selftest;

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_info,
            tap,
            selftest_ready,
            report_key
        ])
        .setup(move |app| {
            // Hidden until the style is set, so the first show does not activate the window.
            let window =
                WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                    .title("EasyDeck A-1 focus spike")
                    .inner_size(1024.0, 600.0)
                    .visible(selftest)
                    .focused(selftest)
                    .build()?;
            if selftest {
                unsafe {
                    let _ = SetWindowsHookExW(WH_KEYBOARD_LL, Some(ll_proc), None, 0);
                }
                run_selftest(window);
                return Ok(());
            }
            if let Some(i) = monitor_idx {
                if let Some(m) = window.available_monitors()?.get(i) {
                    let _ = window.set_position(PhysicalPosition::new(
                        m.position().x + 40,
                        m.position().y + 40,
                    ));
                }
            }
            if let Some(h) = hwnd_of(&window) {
                set_noactivate(h, want_noactivate);
            }
            if want_topmost {
                let _ = window.set_always_on_top(true);
                if let Ok(mut s) = STATE.lock() {
                    s.topmost = true;
                }
            }
            append_log(
                &json!({ "t_ms": now_ms(), "event": "start", "noactivate": want_noactivate,
                                "topmost": want_topmost, "monitor": monitor_idx,
                                "args": std::env::args().collect::<Vec<_>>() }),
            );
            window.show()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running the spike");
}
