extern crate winapi;

use self::winapi::*;
use super::win32helper;
use std::ptr;
use std::mem;

pub fn setup_periodic_callback(period_in_second: u32, callback: win32helper::TimerRoutine, context: win32helper::TimerContext) {

    if win32helper::is_app_already_runniing("Local\\{AB2F0A5E-FAA2-4664-B3C2-25D3984F0A20}") {
        return;
    }

    let console_result = win32helper::alloc_console();
    println!("alloc_console: {:?}", console_result);

    let mut timer = win32helper::PeriodicTimer::new(period_in_second, callback, context);

    let hwnd = timer.create_window();
    timer.register_notification(hwnd);
    timer.start_wait();

    win32helper::message_loop();
}

impl win32helper::PeriodicTimer {
    pub fn create_window(&mut self) -> windef::HWND {
        let instance_handle = win32helper::get_current_instance();
        let class_name = "{8677407E-01E9-4D3E-8BF5-F9082CE08AEB}";
        let window_name = "Monitor";
        let wnd_extra: c_int = win32helper::POINTER_SIZE as c_int; // reserve a space for self pointer

        let hwnd = win32helper::create_window(class_name,
                                              window_name,
                                              win32helper::PeriodicTimer::raw_wnd_proc,
                                              winuser::WS_OVERLAPPEDWINDOW | winuser::WS_VISIBLE,
                                              instance_handle,
                                              wnd_extra);

        if !win32helper::set_window_long_ptr(hwnd,
                                             0,
                                             self as *mut win32helper::PeriodicTimer as basetsd::LONG_PTR) {
            println!("set_window_long_ptr failed with {:?}",
                     win32helper::get_last_error());
        }
        hwnd
    }

    pub fn register_notification(&mut self, hwnd: windef::HWND) {
        let power_settings = [win32helper::GUID_SESSION_USER_PRESENCE, win32helper::GUID_SESSION_DISPLAY_STATUS];
        for setting in power_settings.iter() {
            if win32helper::register_power_setting_notification(hwnd as winnt::HANDLE,
                                                                setting,
                                                                win32helper::DEVICE_NOTIFY_WINDOW_HANDLE) == ptr::null_mut() {
                println!("register_power_setting_notification failed with {:?}",
                         win32helper::get_last_error());
                return;
            }
        }

        if !win32helper::wts_register_session_notification(hwnd, win32helper::NOTIFY_FOR_THIS_SESSION) {
            println!("wts_register_session_notification failed with {:?}",
                     win32helper::get_last_error());
            return;
        }
    }

    pub fn raw_wnd_proc(hwnd: windef::HWND, msg: minwindef::UINT, wparam: minwindef::WPARAM, lparam: minwindef::LPARAM) -> minwindef::LRESULT {
        let raw_ptr = win32helper::get_window_long_ptr(hwnd, 0);
        if raw_ptr != 0 {
            let timer = unsafe { &mut *(raw_ptr as *mut win32helper::PeriodicTimer) };
            timer.wnd_proc(hwnd, msg, wparam, lparam);
        }

        match msg {
            winuser::WM_DESTROY => {
                win32helper::post_quit_message(0);
                0
            }
            _ => win32helper::def_window_proc(hwnd, msg, wparam, lparam),
        }
    }

    fn wnd_proc(&mut self, _: windef::HWND, msg: minwindef::UINT, wparam: minwindef::WPARAM, lparam: minwindef::LPARAM) {
        match msg {
            winuser::WM_POWERBROADCAST => {
                if wparam == win32helper::PBT_POWERSETTINGCHANGE && lparam != 0 {
                    let setting: &win32helper::POWERBROADCAST_SETTING = unsafe { mem::transmute(lparam) };
                    self.power_event(setting);
                }
            }
            winuser::WM_WTSSESSION_CHANGE => {
                self.logon_event(wparam);
            }
            _ => {}
        }
    }

    fn power_event(&mut self, setting: &win32helper::POWERBROADCAST_SETTING) {
        if win32helper::is_equal_guid(&setting.power_setting,
                                      &win32helper::GUID_SESSION_USER_PRESENCE) {
            let power_user_present = 0;
            let power_user_inactive = 2;

            let data = setting.data;
            if self.is_running() && data == power_user_inactive {
                self.stop();
            } else if !self.is_running() && data == power_user_present {
                self.start();
            }

        } else if win32helper::is_equal_guid(&setting.power_setting,
                                             &win32helper::GUID_SESSION_DISPLAY_STATUS) {
            let display_off = 0;
            let display_on = 1;

            let data = setting.data;
            if self.is_running() && data == display_off {
                self.stop();
            } else if !self.is_running() && data == display_on {
                self.start();
            }
        }
    }

    fn logon_event(&mut self, data: minwindef::WPARAM) {
        if self.is_running() && data == win32helper::WTS_SESSION_LOCK {
            self.stop();
        } else if !self.is_running() && data == win32helper::WTS_SESSION_UNLOCK {
            self.start();
        }
    }
}
