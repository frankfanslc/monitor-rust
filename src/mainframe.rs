extern crate winapi;

use self::winapi::*;
use super::win32helper;

pub fn winmain() {

    fn wnd_proc(hwnd: windef::HWND, msg: minwindef::UINT, wparam: minwindef::WPARAM, lparam: LPARAM) -> minwindef::LRESULT {
        match msg {
            winuser::WM_DESTROY => {
                win32helper::post_quit_message(0);
                0
            }
            _ => win32helper::def_window_proc(hwnd, msg, wparam, lparam),
        }
    }

    let instance_handle = win32helper::get_current_instance();
    let class_name = "{8677407E-01E9-4D3E-8BF5-F9082CE08AEB}";
    let window_name = "Monitor";
    let wnd_extra: c_int = win32helper::POINTER_SIZE as c_int;

    let hwnd = win32helper::create_window(class_name,
                                          window_name,
                                          wnd_proc,
                                          winuser::WS_OVERLAPPEDWINDOW | winuser::WS_VISIBLE,
                                          instance_handle,
                                          wnd_extra);
    if hwnd.is_null() {
        return;
    }

    win32helper::set_window_long_ptr(hwnd, 0, hwnd as basetsd::LONG_PTR);
    win32helper::get_window_long_ptr(hwnd, 0);

    win32helper::message_loop();
}
