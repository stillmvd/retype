use super::hook::HookEvent;
use crossbeam_channel::Sender;
use std::sync::atomic::{AtomicBool, Ordering::SeqCst};
use std::sync::OnceLock;
use windows::core::BOOL;
use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND, LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST};
use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED};
use windows::Win32::System::Threading::{
    GetCurrentProcess, GetCurrentProcessId, OpenProcess, OpenProcessToken, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
    PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, SetWinEventHook, HWINEVENTHOOK,
};
use windows::Win32::UI::Shell::{
    SHQueryUserNotificationState, QUNS_PRESENTATION_MODE, QUNS_RUNNING_D3D_FULL_SCREEN,
};
use windows::Win32::UI::WindowsAndMessaging::*;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Context {
    pub exe: String,
    pub elevated: bool,
    pub fullscreen: bool,
    pub password: bool,
    pub own: bool,
}

static SENDER: OnceLock<Sender<HookEvent>> = OnceLock::new();
static CURRENT: OnceLock<arc_swap::ArcSwap<Context>> = OnceLock::new();
static BUSY: AtomicBool = AtomicBool::new(false);

thread_local! {
    static UIA: Option<IUIAutomation> = unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER).ok()
    };
}

pub fn current() -> Context {
    CURRENT.get().map(|c| (**c.load()).clone()).unwrap_or_default()
}

fn process_name(pid: u32) -> String {
    unsafe {
        let Ok(h) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else { return String::new() };
        let mut buf = [0u16; 520];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(h, PROCESS_NAME_WIN32, windows::core::PWSTR(buf.as_mut_ptr()), &mut len).is_ok();
        let _ = CloseHandle(h);
        if !ok {
            return String::new();
        }
        let path = String::from_utf16_lossy(&buf[..len as usize]);
        path.rsplit('\\').next().unwrap_or_default().to_lowercase()
    }
}

fn token_elevated(process: HANDLE) -> Option<bool> {
    unsafe {
        let mut token = HANDLE::default();
        OpenProcessToken(process, TOKEN_QUERY, &mut token).ok()?;
        let mut e = TOKEN_ELEVATION::default();
        let mut ret = 0;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut e as *mut _ as *mut _),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut ret,
        )
        .is_ok();
        let _ = CloseHandle(token);
        ok.then_some(e.TokenIsElevated != 0)
    }
}

fn elevated(pid: u32) -> bool {
    if token_elevated(unsafe { GetCurrentProcess() }) == Some(true) {
        return false;
    }
    unsafe {
        let Ok(h) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else { return true };
        let r = token_elevated(h).unwrap_or(true);
        let _ = CloseHandle(h);
        r
    }
}

fn class_name(hwnd: HWND) -> String {
    let mut buf = [0u16; 128];
    let n = unsafe { GetClassNameW(hwnd, &mut buf) };
    String::from_utf16_lossy(&buf[..n.max(0) as usize])
}

const TYPING_APPS: &[&str] = &[
    "firefox.exe", "chrome.exe", "msedge.exe", "brave.exe", "opera.exe", "vivaldi.exe", "yandex.exe", "browser.exe",
    "windowsterminal.exe", "wt.exe", "code.exe", "vscodium.exe", "cursor.exe", "telegram.exe", "bitrix24.exe",
    "notepad.exe", "explorer.exe", "zen.exe", "librewolf.exe", "arc.exe", "discord.exe", "slack.exe", "obsidian.exe",
    "idea64.exe", "webstorm64.exe", "wezterm-gui.exe", "alacritty.exe", "winword.exe", "notepad++.exe",
];

fn judge_fullscreen(exe: &str, shell_class: bool, covers_monitor: bool, bordered: bool, d3d: bool, presentation: bool) -> bool {
    if presentation {
        return true;
    }
    if shell_class || !covers_monitor || TYPING_APPS.contains(&exe) {
        return false;
    }
    d3d || !bordered
}

fn fullscreen(hwnd: HWND, exe: &str) -> bool {
    let state = unsafe { SHQueryUserNotificationState() }.ok();
    let presentation = state == Some(QUNS_PRESENTATION_MODE);
    let d3d = state == Some(QUNS_RUNNING_D3D_FULL_SCREEN);
    let shell_class = matches!(class_name(hwnd).as_str(), "Progman" | "WorkerW" | "Shell_TrayWnd");
    let style = unsafe { GetWindowLongW(hwnd, GWL_STYLE) } as u32;
    let bordered = style & (WS_CAPTION.0 | WS_THICKFRAME.0) != 0;
    let covers_monitor = unsafe {
        let mut rect = RECT::default();
        let mut info = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
        let mon = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        GetWindowRect(hwnd, &mut rect).is_ok() && GetMonitorInfoW(mon, &mut info).as_bool() && rect == info.rcMonitor
    };
    judge_fullscreen(exe, shell_class, covers_monitor, bordered, d3d, presentation)
}

fn password(focus: HWND) -> bool {
    if !focus.is_invalid() && class_name(focus).eq_ignore_ascii_case("edit") {
        let style = unsafe { GetWindowLongW(focus, GWL_STYLE) } as u32;
        if style & ES_PASSWORD as u32 != 0 {
            return true;
        }
    }
    UIA.with(|uia| {
        uia.as_ref()
            .and_then(|a| unsafe { a.GetFocusedElement() }.ok())
            .and_then(|e| unsafe { e.CurrentIsPassword() }.ok())
            .is_some_and(|b| b.as_bool())
    })
}

unsafe extern "system" fn find_child(child: HWND, lparam: LPARAM) -> BOOL {
    let out = &mut *(lparam.0 as *mut (u32, u32));
    let mut pid = 0;
    GetWindowThreadProcessId(child, Some(&mut pid));
    if pid != out.0 {
        out.1 = pid;
        return false.into();
    }
    true.into()
}

fn compute() -> Context {
    unsafe {
        let hwnd = GetForegroundWindow();
        let mut pid = 0;
        let thread = GetWindowThreadProcessId(hwnd, Some(&mut pid));
        let mut exe = process_name(pid);
        if exe == "applicationframehost.exe" {
            let mut pair = (pid, 0u32);
            let _ = EnumChildWindows(Some(hwnd), Some(find_child), LPARAM(&mut pair as *mut _ as isize));
            if pair.1 != 0 {
                pid = pair.1;
                exe = process_name(pid);
            }
        }
        let mut info = GUITHREADINFO { cbSize: std::mem::size_of::<GUITHREADINFO>() as u32, ..Default::default() };
        let focus = if GetGUIThreadInfo(thread, &mut info).is_ok() { info.hwndFocus } else { HWND::default() };
        let fs = fullscreen(hwnd, &exe);
        Context { exe, elevated: elevated(pid), fullscreen: fs, password: password(focus), own: pid == GetCurrentProcessId() }
    }
}

pub fn refresh() {
    if BUSY.swap(true, SeqCst) {
        return;
    }
    let ctx = compute();
    BUSY.store(false, SeqCst);
    if let Some(c) = CURRENT.get() {
        c.store(ctx.into());
    }
    if let Some(tx) = SENDER.get() {
        let _ = tx.try_send(HookEvent::Focus);
    }
}

unsafe extern "system" fn on_event(_: HWINEVENTHOOK, _: u32, _: HWND, _: i32, _: i32, _: u32, _: u32) {
    let _ = std::panic::catch_unwind(refresh);
}

pub fn run(tx: Sender<HookEvent>) -> std::thread::JoinHandle<()> {
    let _ = SENDER.set(tx);
    let _ = CURRENT.set(arc_swap::ArcSwap::from_pointee(Context::default()));
    std::thread::Builder::new()
        .name("ghbdtn-context".into())
        .spawn(|| unsafe {
            refresh();
            let flags = WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS;
            let _a = SetWinEventHook(EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_FOREGROUND, None, Some(on_event), 0, 0, flags);
            let _b = SetWinEventHook(EVENT_OBJECT_FOCUS, EVENT_OBJECT_FOCUS, None, Some(on_event), 0, 0, flags);
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        })
        .expect("context thread")
}

#[cfg(test)]
mod tests {
    use super::judge_fullscreen as j;

    #[test]
    fn game_on_other_monitor_does_not_mute() {
        assert!(!j("code.exe", false, false, true, true, false));
        assert!(!j("somewindow.exe", false, false, true, true, false));
    }

    #[test]
    fn real_fullscreen_mutes() {
        assert!(j("dota2.exe", false, true, false, false, false));
        assert!(j("game.exe", false, true, true, true, false));
        assert!(j("mpc-hc64.exe", false, true, false, false, false));
    }

    #[test]
    fn maximized_and_typing_apps_stay_active() {
        assert!(!j("game.exe", false, true, true, false, false));
        assert!(!j("firefox.exe", false, true, false, false, false));
        assert!(!j("windowsterminal.exe", false, true, false, true, false));
        assert!(!j("x.exe", true, true, false, false, false));
    }

    #[test]
    fn presentation_mode_always_mutes() {
        assert!(j("firefox.exe", false, false, true, false, true));
    }
}
