#![cfg(target_os = "windows")]

use std::ffi::c_void;
use std::thread;
use tauri::Emitter;

const WH_KEYBOARD_LL: i32 = 13;
const WH_MOUSE_LL: i32 = 14;
const WM_KEYDOWN: usize = 0x0100;
const WM_SYSKEYDOWN: usize = 0x0104;
const WM_MOUSEWHEEL: usize = 0x020A;

#[repr(C)]
struct MSLLHOOKSTRUCT {
    pt_x: i32,
    pt_y: i32,
    mouse_data: u32,
    flags: u32,
    time: u32,
    dw_extra_info: usize,
}

#[link(name = "user32")]
extern "system" {
    fn SetWindowsHookExW(
        idHook: i32,
        lpfn: unsafe extern "system" fn(i32, usize, isize) -> isize,
        hMod: *mut c_void,
        dwThreadId: u32,
    ) -> *mut c_void;
    fn CallNextHookEx(hhk: *mut c_void, nCode: i32, wParam: usize, lParam: isize) -> isize;
    fn GetMessageW(
        lpMsg: *mut c_void,
        hWnd: *mut c_void,
        wMsgFilterMin: u32,
        wMsgFilterMax: u32,
    ) -> i32;
    fn GetModuleHandleW(lpModuleName: *const u16) -> *mut c_void;
}

static mut APP_HANDLE: Option<tauri::AppHandle> = None;

unsafe extern "system" fn low_level_keyboard_proc(
    n_code: i32,
    w_param: usize,
    l_param: isize,
) -> isize {
    if n_code >= 0 && (w_param == WM_KEYDOWN || w_param == WM_SYSKEYDOWN) {
        if let Some(ref handle) = APP_HANDLE {
            let _ = handle.emit("key-typed", ());
        }
    }
    CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param)
}

unsafe extern "system" fn low_level_mouse_proc(
    n_code: i32,
    w_param: usize,
    l_param: isize,
) -> isize {
    if n_code >= 0 && w_param == WM_MOUSEWHEEL {
        let hook_struct = &*(l_param as *const MSLLHOOKSTRUCT);
        let delta = (hook_struct.mouse_data >> 16) as i16 as i32;
        if delta != 0 && delta.abs() >= 10 {
            if let Some(ref handle) = APP_HANDLE {
                let _ = handle.emit("scroll-event", delta / 10);
            }
        }
    }
    CallNextHookEx(std::ptr::null_mut(), n_code, w_param, l_param)
}

pub fn start(app_handle: tauri::AppHandle) {
    unsafe {
        APP_HANDLE = Some(app_handle);
    }
    thread::spawn(|| unsafe {
        let h_mod = GetModuleHandleW(std::ptr::null());
        SetWindowsHookExW(WH_KEYBOARD_LL, low_level_keyboard_proc, h_mod, 0);
        SetWindowsHookExW(WH_MOUSE_LL, low_level_mouse_proc, h_mod, 0);
        let mut msg = [0u8; 48];
        while GetMessageW(msg.as_mut_ptr() as *mut c_void, std::ptr::null_mut(), 0, 0) > 0 {}
    });
}
