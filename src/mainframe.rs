extern crate winapi;

use self::winapi::*;
use super::win32helper;

use std::mem;
use std::ptr;

pub fn winmain() {
    let instance_handle = win32helper::get_module_handle(ptr::null()) as minwindef::HINSTANCE;

    unsafe extern "system" fn wnd_proc(hwnd: windef::HWND, msg: minwindef::UINT, wparam: minwindef::WPARAM, lparam: LPARAM) -> minwindef::LRESULT {
        match msg {
            winuser::WM_DESTROY => {
                win32helper::post_quit_message(0);
                0
            }
            _ => win32helper::def_window_proc(hwnd, msg, wparam, lparam),
        }
    }

    let window_class_name = win32helper::to_wide_chars("{8677407E-01E9-4D3E-8BF5-F9082CE08AEB}");
    let window_title = win32helper::to_wide_chars("Monitor");

    let mut wnd_class: winuser::WNDCLASSW = unsafe { mem::zeroed() };
    wnd_class.lpfnWndProc = Some(wnd_proc);
    wnd_class.hInstance = instance_handle;
    wnd_class.hbrBackground = winuser::COLOR_BACKGROUND as windef::HBRUSH;
    wnd_class.lpszClassName = window_class_name.as_ptr();

    if !win32helper::register_class(&wnd_class) {
        return;
    }

    let hwnd = win32helper::create_window(wnd_class.lpszClassName,
                                          window_title.as_ptr(),
                                          winuser::WS_OVERLAPPEDWINDOW | winuser::WS_VISIBLE,
                                          instance_handle);
    if hwnd.is_null() {
        return;
    }

    win32helper::message_loop();
}
