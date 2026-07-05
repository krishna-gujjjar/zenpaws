#![cfg(target_os = "macos")]

use std::ffi::c_void;
use tauri::Emitter;

// Event tap placement
const KCG_HID_EVENT_TAP: u32 = 0;
const KCG_HEAD_INSERT_EVENT_TAP: u32 = 0;
const KCG_EVENT_TAP_OPTION_LISTEN_ONLY: u32 = 1;

// Event masks
const KCG_EVENT_KEY_DOWN: u32 = 10;
const KCG_EVENT_SCROLL_WHEEL: u32 = 22;
const KCG_EVENT_TAP_DISABLED_BY_TIMEOUT: u32 = 0xFFFFFFFE;
const KCG_EVENT_TAP_DISABLED_BY_USER_INPUT: u32 = 0xFFFFFFFF;

const KCG_EVENT_MASK: u64 = (1 << KCG_EVENT_KEY_DOWN) | (1 << KCG_EVENT_SCROLL_WHEEL);

// CGEventField for scroll delta
const KCG_SCROLL_WHEEL_EVENT_DELTA_AXIS1: u32 = 11;

type CGEventRef = *const c_void;
type CGEventTapRef = *const c_void;

type CGEventTapCallBack =
    unsafe extern "C" fn(CGEventTapRef, u32, CGEventRef, *mut c_void) -> CGEventRef;

// Shared state between callback and main loop
struct TapState {
    handle: tauri::AppHandle,
    tap: CGEventTapRef,
}

// Make it sendable across threads
unsafe impl Send for TapState {}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventTapCreate(
        tap: u32,
        place: u32,
        options: u32,
        events_of_interest: u64,
        callback: CGEventTapCallBack,
        user_info: *mut c_void,
    ) -> CGEventTapRef;

    fn CGEventTapEnable(tap: CGEventTapRef, enable: bool);

    fn CGEventGetIntegerValueField(event: CGEventRef, field: u32) -> i64;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFMachPortCreateRunLoopSource(
        allocator: *const c_void,
        port: *const c_void,
        order: i64,
    ) -> *const c_void;
    fn CFRunLoopGetCurrent() -> *const c_void;
    fn CFRunLoopAddSource(rl: *const c_void, source: *const c_void, mode: *const c_void);
    fn CFRunLoopRun();
    static kCFRunLoopCommonModes: *const c_void;
}

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
}

unsafe extern "C" fn tap_callback(
    tap_ref: CGEventTapRef,
    event_type: u32,
    event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef {
    if user_info.is_null() {
        return event;
    }

    let state = &*(user_info as *const TapState);

    match event_type {
        // Re-enable tap if macOS disabled it due to timeout
        KCG_EVENT_TAP_DISABLED_BY_TIMEOUT | KCG_EVENT_TAP_DISABLED_BY_USER_INPUT => {
            eprintln!("[Comnyang] Event tap was disabled, re-enabling...");
            CGEventTapEnable(state.tap, true);
        }

        KCG_EVENT_KEY_DOWN => {
            let _ = state.handle.emit("key-typed", ());
        }

        KCG_EVENT_SCROLL_WHEEL => {
            let delta = CGEventGetIntegerValueField(event, KCG_SCROLL_WHEEL_EVENT_DELTA_AXIS1);
            if delta != 0 {
                let _ = state.handle.emit("scroll-event", delta);
            }
        }

        _ => {}
    }

    event
}

pub fn start(app_handle: tauri::AppHandle) {
    std::thread::spawn(move || unsafe {
        // Check Accessibility
        if !AXIsProcessTrusted() {
            eprintln!(
                "[Comnyang] Accessibility not granted.\n\
                 System Settings → Privacy & Security → Accessibility\n\
                 → Add terminal → toggle ON → fully quit & reopen terminal"
            );
        }

        // Create tap first (we need the ref for TapState)
        // Use a placeholder, fill in after creation
        let placeholder = Box::new(TapState {
            handle: app_handle.clone(),
            tap: std::ptr::null(),
        });
        let ptr = Box::into_raw(placeholder) as *mut c_void;

        let tap = CGEventTapCreate(
            KCG_HID_EVENT_TAP,
            KCG_HEAD_INSERT_EVENT_TAP,
            KCG_EVENT_TAP_OPTION_LISTEN_ONLY,
            KCG_EVENT_MASK,
            tap_callback,
            ptr,
        );

        if tap.is_null() {
            eprintln!(
                "[Comnyang] CGEventTap failed — Accessibility not granted or denied.\n\
                 System Settings → Privacy & Security → Accessibility\n\
                 → Add your terminal → toggle ON → fully quit & reopen terminal"
            );
            drop(Box::from_raw(ptr as *mut TapState));
            return;
        }

        // Now store the real tap ref in state so callback can re-enable it
        let state = &mut *(ptr as *mut TapState);
        state.tap = tap;

        // Explicitly enable the tap
        CGEventTapEnable(tap, true);

        eprintln!("[Comnyang] Global keyboard + scroll listener started.");

        let source = CFMachPortCreateRunLoopSource(std::ptr::null(), tap, 0);

        if source.is_null() {
            eprintln!("[Comnyang] Failed to create run loop source.");
            drop(Box::from_raw(ptr as *mut TapState));
            return;
        }

        let run_loop = CFRunLoopGetCurrent();
        CFRunLoopAddSource(run_loop, source, kCFRunLoopCommonModes);

        // This blocks — runs the event tap loop on this thread
        CFRunLoopRun();
    });
}
