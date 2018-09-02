extern crate winapi;

use get_foreground_app;
use CHECK_INTERNVAL_IN_SECONDS;

use self::winapi::{ctypes, shared::minwindef, shared::windef, um::winnt, um::winuser};

use super::win32helper;
use std::mem;
use std::ptr;

struct MyTimer {
    hwnd: Option<windef::HWND>,
    running: bool,
    period_in_second: u32,
    id: minwindef::UINT,
}

impl MyTimer {
    pub fn start(&mut self) {
        win32helper::set_timer(self.hwnd.unwrap(), self.id, self.period_in_second * 1000);
        self.running = true;
    }

    pub fn stop(&mut self) {
        win32helper::kill_timer(self.hwnd.unwrap(), self.id);
        self.running = false;
    }

    pub fn is_running(&self) -> bool {
        self.running
    }
}

pub struct MainFrame {
    hwnd: Option<windef::HWND>,
    timer: MyTimer,
}

impl win32helper::WindowTrait for MainFrame {
    fn wnd_proc(
        &mut self,
        hwnd: windef::HWND,
        msg: minwindef::UINT,
        wparam: minwindef::WPARAM,
        lparam: minwindef::LPARAM,
    ) -> minwindef::LRESULT {
        match msg {
            winuser::WM_TIMER => {
                get_foreground_app();
            }
            winuser::WM_POWERBROADCAST => {
                if wparam == win32helper::PBT_POWERSETTINGCHANGE && lparam != 0 {
                    let setting: &win32helper::POWERBROADCAST_SETTING =
                        unsafe { mem::transmute(lparam) };
                    self.power_event(setting);
                }
            }
            winuser::WM_WTSSESSION_CHANGE => {
                self.logon_event(wparam);
            }
            winuser::WM_DESTROY => {
                win32helper::post_quit_message(0);
            }
            _ => {}
        }
        win32helper::def_window_proc(hwnd, msg, wparam, lparam)
    }
}

impl MainFrame {
    pub fn run() {
        let timer = MyTimer {
            hwnd: None,
            running: false,
            period_in_second: CHECK_INTERNVAL_IN_SECONDS,
            id: 1, // non-zero
        };

        let mut frame = MainFrame {
            hwnd: None,
            timer: timer,
        };

        let hwnd = frame.create_window();
        frame.hwnd = Some(hwnd);
        frame.timer.hwnd = Some(hwnd);

        frame.register_notification(hwnd);
        frame.timer.start();

        win32helper::message_loop();
    }

    pub fn create_window(&mut self) -> windef::HWND {
        let instance_handle = win32helper::get_current_instance();
        let class_name = "{8677407E-01E9-4D3E-8BF5-F9082CE08AEB}";
        let window_name = "Monitor";
        let wnd_extra: ctypes::c_int = 0;

        let hwnd = win32helper::create_window::<MainFrame>(
            self,
            class_name,
            window_name,
            winuser::WS_OVERLAPPEDWINDOW, // | winuser::WS_VISIBLE,
            instance_handle,
            wnd_extra,
        );
        hwnd
    }

    pub fn register_notification(&mut self, hwnd: windef::HWND) {
        let power_settings = [
            win32helper::GUID_SESSION_USER_PRESENCE,
            win32helper::GUID_SESSION_DISPLAY_STATUS,
        ];
        for setting in power_settings.iter() {
            if win32helper::register_power_setting_notification(
                hwnd as winnt::HANDLE,
                setting,
                win32helper::DEVICE_NOTIFY_WINDOW_HANDLE,
            ) == ptr::null_mut()
            {
                println!(
                    "register_power_setting_notification failed with {:?}",
                    win32helper::get_last_error()
                );
                return;
            }
        }

        if !win32helper::wts_register_session_notification(
            hwnd,
            win32helper::NOTIFY_FOR_THIS_SESSION,
        ) {
            println!(
                "wts_register_session_notification failed with {:?}",
                win32helper::get_last_error()
            );
            return;
        }
    }

    fn power_event(&mut self, setting: &win32helper::POWERBROADCAST_SETTING) {
        if win32helper::is_equal_guid(
            &setting.power_setting,
            &win32helper::GUID_SESSION_USER_PRESENCE,
        ) {
            let power_user_present = 0;
            let power_user_inactive = 2;

            let data = setting.data;
            if self.timer.is_running() && data == power_user_inactive {
                self.timer.stop();
            } else if !self.timer.is_running() && data == power_user_present {
                self.timer.start();
            }
        } else if win32helper::is_equal_guid(
            &setting.power_setting,
            &win32helper::GUID_SESSION_DISPLAY_STATUS,
        ) {
            let display_off = 0;
            let display_on = 1;

            let data = setting.data;
            if self.timer.is_running() && data == display_off {
                self.timer.stop();
            } else if !self.timer.is_running() && data == display_on {
                self.timer.start();
            }
        }
    }

    fn logon_event(&mut self, data: minwindef::WPARAM) {
        if self.timer.is_running() && data == win32helper::WTS_SESSION_LOCK {
            self.timer.stop();
        } else if !self.timer.is_running() && data == win32helper::WTS_SESSION_UNLOCK {
            self.timer.start();
        }
    }
}
