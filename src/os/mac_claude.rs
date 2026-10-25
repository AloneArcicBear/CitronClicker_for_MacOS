// #claude
//! macos input backend (quartz event services + nsworkspace). no hook thread needed:
//! - clicks are injected at the session tap from a *private* event source, so they never touch
//!   the hid system state
//! - physical button/key state is read from the hid system state, which only the real hardware
//!   drives. our own injected clicks therefore can't feed back into physical_button_held.
//! needs "accessibility" (post events) and "input monitoring" (read key state) in system settings.

use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::{AtomicPtr, Ordering};

use objc2_app_kit::{NSRunningApplication, NSWorkspace};

type CGEventRef = *mut c_void;
type CGEventSourceRef = *mut c_void;

#[repr(C)]
#[derive(Clone, Copy)]
struct CGPoint {
    x: f64,
    y: f64,
}

// CGEventSourceStateID
const STATE_PRIVATE: i32 = -1;
const STATE_HID_SYSTEM: i32 = 1;
// CGEventTapLocation
const TAP_SESSION: u32 = 1;
// CGEventType
const EV_LEFT_DOWN: u32 = 1;
const EV_LEFT_UP: u32 = 2;
const EV_RIGHT_DOWN: u32 = 3;
const EV_RIGHT_UP: u32 = 4;
const EV_MOUSE_MOVED: u32 = 5;
// CGMouseButton
const BTN_LEFT: u32 = 0;
const BTN_RIGHT: u32 = 1;
// CGEventField
const FIELD_CLICK_STATE: u32 = 1;
const FIELD_DELTA_X: u32 = 4;
const FIELD_DELTA_Y: u32 = 5;

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGEventSourceCreate(state: i32) -> CGEventSourceRef;
    fn CGEventSourceButtonState(state: i32, button: u32) -> bool;
    fn CGEventSourceKeyState(state: i32, key: u16) -> bool;
    fn CGEventCreate(source: CGEventSourceRef) -> CGEventRef;
    fn CGEventGetLocation(event: CGEventRef) -> CGPoint;
    fn CGEventCreateMouseEvent(
        source: CGEventSourceRef,
        ty: u32,
        pos: CGPoint,
        button: u32,
    ) -> CGEventRef;
    fn CGEventSetIntegerValueField(event: CGEventRef, field: u32, value: i64);
    fn CGEventPost(tap: u32, event: CGEventRef);
    fn CGCursorIsVisible() -> i32;
    fn CGRequestPostEventAccess() -> bool;
    fn CGRequestListenEventAccess() -> bool;
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFRelease(cf: *const c_void);
}

static SOURCE: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static SEND_LOCK: Mutex<()> = Mutex::new(());

fn source() -> CGEventSourceRef {
    let p = SOURCE.load(Ordering::Acquire);
    if !p.is_null() {
        return p;
    }
    let created = unsafe { CGEventSourceCreate(STATE_PRIVATE) };
    match SOURCE.compare_exchange(
        std::ptr::null_mut(),
        created,
        Ordering::AcqRel,
        Ordering::Acquire,
    ) {
        Ok(_) => created,
        Err(existing) => {
            if !created.is_null() {
                unsafe { CFRelease(created) };
            }
            existing
        }
    }
}

fn cursor_pos() -> CGPoint {
    unsafe {
        let ev = CGEventCreate(std::ptr::null_mut());
        if ev.is_null() {
            return CGPoint { x: 0.0, y: 0.0 };
        }
        let p = CGEventGetLocation(ev);
        CFRelease(ev);
        p
    }
}

pub fn begin_timer_period() {}

/// menu bar icon: 18pt tall, rendered at 2x for retina
pub fn small_icon_px() -> u32 {
    36
}

pub fn acquire_single_instance() -> bool {
    true
}

pub fn focus_existing_window() {}

/// no hook thread on macos; just trigger the permission prompts once. returns 0 (no thread id).
pub fn start_input_hook() -> u32 {
    unsafe {
        CGRequestPostEventAccess();
        CGRequestListenEventAccess();
    }
    0
}

pub fn stop_input_hook(_thread_id: u32) {}

pub fn physical_button_held(is_left: bool) -> bool {
    unsafe { CGEventSourceButtonState(STATE_HID_SYSTEM, if is_left { BTN_LEFT } else { BTN_RIGHT }) }
}

fn post_mouse(ty: u32, button: u32, pos: CGPoint, delta: Option<(i32, i32)>, click: bool) {
    let src = source();
    if src.is_null() {
        return;
    }
    // serialize every injected event (both clickers + jitter) so they never interleave
    let _g = SEND_LOCK.lock().unwrap();
    unsafe {
        let ev = CGEventCreateMouseEvent(src, ty, pos, button);
        if ev.is_null() {
            return;
        }
        if click {
            CGEventSetIntegerValueField(ev, FIELD_CLICK_STATE, 1);
        }
        if let Some((dx, dy)) = delta {
            CGEventSetIntegerValueField(ev, FIELD_DELTA_X, dx as i64);
            CGEventSetIntegerValueField(ev, FIELD_DELTA_Y, dy as i64);
        }
        CGEventPost(TAP_SESSION, ev);
        CFRelease(ev);
    }
}

pub fn click_down(is_left: bool) {
    let (ty, btn) = if is_left {
        (EV_LEFT_DOWN, BTN_LEFT)
    } else {
        (EV_RIGHT_DOWN, BTN_RIGHT)
    };
    post_mouse(ty, btn, cursor_pos(), None, true);
}

pub fn click_up(is_left: bool) {
    let (ty, btn) = if is_left {
        (EV_LEFT_UP, BTN_LEFT)
    } else {
        (EV_RIGHT_UP, BTN_RIGHT)
    };
    post_mouse(ty, btn, cursor_pos(), None, true);
}

pub fn jitter_move(dx: i32, dy: i32) {
    let p = cursor_pos();
    let to = CGPoint {
        x: p.x + dx as f64,
        y: p.y + dy as f64,
    };
    post_mouse(EV_MOUSE_MOVED, BTN_LEFT, to, Some((dx, dy)), false);
}

/// false while a game has grabbed (hidden) the cursor, i.e. you're in the 3d view
pub fn cursor_visible() -> bool {
    unsafe { CGCursorIsVisible() != 0 }
}

/// windows virtual-key code (what the engine/ui uses everywhere) -> macos virtual keycode
fn mac_keycode(vk: i32) -> Option<u16> {
    let c = match vk {
        0x20 => 49,                   // space
        0x10 => 56,                   // shift
        0x11 => 59,                   // ctrl
        0x12 => 58,                   // alt / option
        0x09 => 48,                   // tab
        0x14 => 57,                   // caps lock
        0x2D => 114,                  // insert -> help key
        0x41 => 0,                    // a
        0x42 => 11,                   // b
        0x43 => 8,                    // c
        0x44 => 2,                    // d
        0x45 => 14,                   // e
        0x46 => 3,                    // f
        0x47 => 5,                    // g
        0x48 => 4,                    // h
        0x49 => 34,                   // i
        0x4A => 38,                   // j
        0x4B => 40,                   // k
        0x4C => 37,                   // l
        0x4D => 46,                   // m
        0x4E => 45,                   // n
        0x4F => 31,                   // o
        0x50 => 35,                   // p
        0x51 => 12,                   // q
        0x52 => 15,                   // r
        0x53 => 1,                    // s
        0x54 => 17,                   // t
        0x55 => 32,                   // u
        0x56 => 9,                    // v
        0x57 => 13,                   // w
        0x58 => 7,                    // x
        0x59 => 16,                   // y
        0x5A => 6,                    // z
        0x30 => 29,                   // 0
        0x31 => 18,                   // 1
        0x32 => 19,                   // 2
        0x33 => 20,                   // 3
        0x34 => 21,                   // 4
        0x35 => 23,                   // 5
        0x36 => 22,                   // 6
        0x37 => 26,                   // 7
        0x38 => 28,                   // 8
        0x39 => 25,                   // 9
        0x70 => 122,                  // f1
        0x71 => 120,                  // f2
        0x72 => 99,                   // f3
        0x73 => 118,                  // f4
        0x74 => 96,                   // f5
        0x75 => 97,                   // f6
        0x76 => 98,                   // f7
        0x77 => 100,                  // f8
        0x78 => 101,                  // f9
        0x79 => 109,                  // f10
        0x7A => 103,                  // f11
        0x7B => 111,                  // f12
        0x7C => 105,                  // f13
        0x7D => 107,                  // f14
        0x7E => 113,                  // f15
        0x7F => 106,                  // f16
        0x80 => 64,                   // f17
        0x81 => 79,                   // f18
        0x82 => 80,                   // f19
        0x83 => 90,                   // f20
        _ => return None,
    };
    Some(c)
}

pub fn key_held(vk: i32) -> bool {
    match vk {
        0 => false,
        0x01 => physical_button_held(true),
        0x02 => physical_button_held(false),
        // middle / side buttons
        0x04 => unsafe { CGEventSourceButtonState(STATE_HID_SYSTEM, 2) },
        0x05 => unsafe { CGEventSourceButtonState(STATE_HID_SYSTEM, 3) },
        0x06 => unsafe { CGEventSourceButtonState(STATE_HID_SYSTEM, 4) },
        _ => match mac_keycode(vk) {
            Some(code) => unsafe { CGEventSourceKeyState(STATE_HID_SYSTEM, code) },
            None => false,
        },
    }
}

fn app_looks_like_minecraft(app: &NSRunningApplication) -> bool {
    let name = app
        .localizedName()
        .map(|n| n.to_string().to_lowercase())
        .unwrap_or_default();
    let bundle = app
        .bundleIdentifier()
        .map(|b| b.to_string().to_lowercase())
        .unwrap_or_default();
    if name.contains("launcher") || bundle.contains("launcher") {
        return false;
    }
    // the java game process usually shows up as plain "java" (or the client's own name)
    const NAMES: [&str; 6] = ["minecraft", "lunar", "badlion", "feather", "labymod", "java"];
    NAMES.iter().any(|n| name == *n || name.contains(n))
        && !name.contains("javascript")
}

/// true when mc is the frontmost app, gates clicking in "only in-game"
pub fn is_minecraft_active() -> bool {
    match NSWorkspace::sharedWorkspace().frontmostApplication() {
        Some(app) => app_looks_like_minecraft(&app),
        None => false,
    }
}

/// true if an mc process is running anywhere (even in the background), for the status badge
pub fn is_minecraft_running() -> bool {
    NSWorkspace::sharedWorkspace()
        .runningApplications()
        .iter()
        .any(|a| app_looks_like_minecraft(&a))
}

pub fn any_window_focused() -> bool {
    NSWorkspace::sharedWorkspace().frontmostApplication().is_some()
}

/// true when our own app is frontmost, so we never click into our own ui
pub fn foreground_is_self() -> bool {
    NSRunningApplication::currentApplication().isActive()
}

const LAUNCH_AGENT: &str = "com.citron.v2.plist";

/// start at login via a per-user launch agent (~/Library/LaunchAgents). takes effect at next login.
pub fn set_autostart(enabled: bool) {
    let Some(home) = std::env::var_os("HOME") else {
        return;
    };
    let dir = std::path::Path::new(&home).join("Library/LaunchAgents");
    let plist = dir.join(LAUNCH_AGENT);
    if !enabled {
        let _ = std::fs::remove_file(&plist);
        return;
    }
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let exe = exe.display().to_string().replace('&', "&amp;").replace('<', "&lt;");
    let body = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.citron.v2</string>
    <key>ProgramArguments</key>
    <array>
        <string>{exe}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>
"#
    );
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::write(&plist, body);
}

/// hide (or restore) our windows from screenshots / screen recording. main thread only.
pub fn set_screen_capture_excluded(excluded: bool) {
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSApplication, NSWindowSharingType};
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let ty = if excluded {
        NSWindowSharingType::None
    } else {
        NSWindowSharingType::ReadOnly
    };
    for win in NSApplication::sharedApplication(mtm).windows().iter() {
        win.setSharingType(ty);
    }
}

// grand central dispatch (libSystem): run a function on the main thread from any thread
unsafe extern "C" {
    static _dispatch_main_q: c_void;
    fn dispatch_async_f(queue: *const c_void, context: *mut c_void, work: extern "C" fn(*mut c_void));
}

static DOCK_HIDDEN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

extern "C" fn apply_dock(_ctx: *mut c_void) {
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let app = NSApplication::sharedApplication(mtm);
    if DOCK_HIDDEN.load(Ordering::Relaxed) {
        let _ = app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
    } else {
        let _ = app.setActivationPolicy(NSApplicationActivationPolicy::Regular);
    }
}

/// hide (or restore) the dock icon. called from the engine thread (hotkey works while the game is
/// focused), so the appkit call is handed to the main thread.
pub fn set_taskbar_hidden(hidden: bool) {
    DOCK_HIDDEN.store(hidden, Ordering::Relaxed);
    unsafe {
        dispatch_async_f(
            &raw const _dispatch_main_q,
            std::ptr::null_mut(),
            apply_dock,
        );
    }
}

/// hide the whole app (close-to-menu-bar). main thread only.
pub fn hide_app() {
    use objc2::MainThreadMarker;
    use objc2_app_kit::NSApplication;
    if let Some(mtm) = MainThreadMarker::new() {
        NSApplication::sharedApplication(mtm).hide(None);
    }
}

/// bring the app back from the menu bar. main thread only.
#[allow(deprecated)]
pub fn show_app() {
    use objc2::MainThreadMarker;
    use objc2_app_kit::NSApplication;
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let app = NSApplication::sharedApplication(mtm);
    app.unhide(None);
    app.activateIgnoringOtherApps(true);
    for win in app.windows().iter() {
        if win.standardWindowButton(objc2_app_kit::NSWindowButton::CloseButton).is_some() {
            win.makeKeyAndOrderFront(None);
        }
    }
}

/// quit from the menu bar. never leave an injected mouse button stuck down.
pub fn quit_now() -> ! {
    click_up(true);
    click_up(false);
    std::process::exit(0);
}

/// move the native traffic lights so their centre sits `center_y` points below the top edge of the
/// window and the close button starts `left` points from the left edge (same trick electron uses
/// for `trafficLightPosition`). macos re-lays out the title bar on focus/resize changes, so call
/// this every frame; it only touches the views when something actually moved. main thread only.
pub fn place_traffic_lights(center_y: f64, left: f64) {
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSApplication, NSWindowButton};

    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    for win in NSApplication::sharedApplication(mtm).windows().iter() {
        let (Some(close), Some(mini), Some(zoom)) = (
            win.standardWindowButton(NSWindowButton::CloseButton),
            win.standardWindowButton(NSWindowButton::MiniaturizeButton),
            win.standardWindowButton(NSWindowButton::ZoomButton),
        ) else {
            continue;
        };
        // button -> titlebar view -> titlebar container view
        let Some(container) = (unsafe { close.superview() }).and_then(|v| unsafe { v.superview() })
        else {
            continue;
        };
        let btn_h = close.frame().size.height;
        let mut spacing = mini.frame().origin.x - close.frame().origin.x;
        if spacing <= 0.0 {
            spacing = 20.0;
        }
        // grow the container to 2 * center_y so the buttons, centred inside it, land on center_y
        let h = (center_y * 2.0).max(btn_h);
        let mut cf = container.frame();
        let top = win.frame().size.height - h;
        if (cf.size.height - h).abs() > 0.5 || (cf.origin.y - top).abs() > 0.5 {
            cf.size.height = h;
            cf.origin.y = top;
            container.setFrame(cf);
        }
        for (i, b) in [&close, &mini, &zoom].into_iter().enumerate() {
            let mut o = b.frame().origin;
            let (x, y) = (left + i as f64 * spacing, (h - btn_h) / 2.0);
            if (o.x - x).abs() > 0.5 || (o.y - y).abs() > 0.5 {
                o.x = x;
                o.y = y;
                b.setFrameOrigin(o);
            }
        }
    }
}
