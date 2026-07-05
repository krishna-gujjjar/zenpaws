#![cfg(target_os = "macos")]

use std::ffi::c_void;
use std::sync::{Arc, Mutex};
use tauri::Emitter;

const KCG_SESSION_EVENT_TAP: u32 = 1;
const KCG_HEAD_INSERT_EVENT_TAP: u32 = 0;
const KCG_EVENT_TAP_OPTION_LISTEN_ONLY: u32 = 1;

const KCG_EVENT_KEY_DOWN: u32 = 10;
const KCG_EVENT_SCROLL_WHEEL: u32 = 22;
const KCG_EVENT_TAP_DISABLED_BY_TIMEOUT: u32 = 0xFFFFFFFE;
const KCG_EVENT_TAP_DISABLED_BY_USER_INPUT: u32 = 0xFFFFFFFF;

const KCG_EVENT_MASK: u64 = (1 << KCG_EVENT_KEY_DOWN) | (1 << KCG_EVENT_SCROLL_WHEEL);
const KCG_SCROLL_WHEEL_EVENT_DELTA_AXIS1: u32 = 11;

type CGEventRef = *const c_void;
type CGEventTapRef = *const c_void;

type CGEventTapCallBack =
    unsafe extern "C" fn(CGEventTapRef, u32, CGEventRef, *mut c_void) -> CGEventRef;

pub struct TapState {
    pub handle: tauri::AppHandle,
    pub tap: Arc<Mutex<CGEventTapRef>>,
}

unsafe impl Send for TapState {}
unsafe impl Sync for TapState {}

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

    // Get the MAIN run loop — not current thread's
    fn CFRunLoopGetMain() -> *const c_void;

    fn CFRunLoopAddSource(rl: *const c_void, source: *const c_void, mode: *const c_void);
    static kCFRunLoopCommonModes: *const c_void;
    static kCFBooleanTrue: *const c_void;
    fn CFDictionaryCreate(
        allocator: *const c_void,
        keys: *const *const c_void,
        values: *const *const c_void,
        num_values: i64,
        key_call_backs: *const c_void,
        value_call_backs: *const c_void,
    ) -> *const c_void;
    static kCFTypeDictionaryKeyCallBacks: c_void;
    static kCFTypeDictionaryValueCallBacks: c_void;
    fn CFRelease(cf: *const c_void);
}

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
    fn AXIsProcessTrustedWithOptions(options: *const c_void) -> bool;
    static kAXTrustedCheckOptionPrompt: *const c_void;
}

unsafe extern "C" fn tap_callback(
    _tap_ref: CGEventTapRef,
    event_type: u32,
    event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef {
    if user_info.is_null() {
        return event;
    }

    let state = &*(user_info as *const TapState);

    match event_type {
        KCG_EVENT_TAP_DISABLED_BY_TIMEOUT | KCG_EVENT_TAP_DISABLED_BY_USER_INPUT => {
            let tap_ptr = *state.tap.lock().unwrap();
            if !tap_ptr.is_null() {
                CGEventTapEnable(tap_ptr, true);
            }
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
    // No thread spawn — register directly on calling context
    // source is added to MAIN run loop so it fires on macOS main thread
    unsafe {
        if !AXIsProcessTrusted() {
            eprintln!(
                "[Comnyang] Accessibility not granted. Requesting macOS prompt...\n\
                 System Settings → Privacy & Security → Accessibility\n\
                 → Add terminal/app → toggle ON → fully quit & reopen"
            );
            let keys = [kAXTrustedCheckOptionPrompt];
            let values = [kCFBooleanTrue];
            let dict = CFDictionaryCreate(
                std::ptr::null(),
                keys.as_ptr() as *const *const c_void,
                values.as_ptr() as *const *const c_void,
                1,
                &kCFTypeDictionaryKeyCallBacks as *const _ as *const c_void,
                &kCFTypeDictionaryValueCallBacks as *const _ as *const c_void,
            );
            if !dict.is_null() {
                AXIsProcessTrustedWithOptions(dict);
                CFRelease(dict);
            }
        }

        let tap_arc: Arc<Mutex<CGEventTapRef>> = Arc::new(Mutex::new(std::ptr::null()));

        let state = Box::new(TapState {
            handle: app_handle,
            tap: Arc::clone(&tap_arc),
        });
        let ptr = Box::into_raw(state) as *mut c_void;

        let tap = CGEventTapCreate(
            KCG_SESSION_EVENT_TAP,
            KCG_HEAD_INSERT_EVENT_TAP,
            KCG_EVENT_TAP_OPTION_LISTEN_ONLY,
            KCG_EVENT_MASK,
            tap_callback,
            ptr,
        );

        if tap.is_null() {
            eprintln!(
                "[Comnyang] CGEventTap failed.\n\
                 System Settings → Privacy & Security → Accessibility\n\
                 → Add terminal → toggle ON → fully quit & reopen"
            );
            drop(Box::from_raw(ptr as *mut TapState));
            return;
        }

        *tap_arc.lock().unwrap() = tap;
        CGEventTapEnable(tap, true);

        let source = CFMachPortCreateRunLoopSource(std::ptr::null(), tap, 0);

        if source.is_null() {
            eprintln!("[Comnyang] Failed to create run loop source.");
            drop(Box::from_raw(ptr as *mut TapState));
            return;
        }

        // CRITICAL FIX: attach to MAIN run loop, not current thread
        let main_run_loop = CFRunLoopGetMain();
        CFRunLoopAddSource(main_run_loop, source, kCFRunLoopCommonModes);

        eprintln!("[Comnyang] Global keyboard + scroll listener started on main run loop.");
    }
}
