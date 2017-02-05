extern crate winapi;
extern crate user32;
extern crate kernel32;

use self::winapi::*;
use std::mem;
use std::ptr;

pub use self::extra::*;
mod extra;

pub use self::timer::*;
mod timer;

pub use self::ntdll::*;
mod ntdll;

#[cfg(target_pointer_width = "32")]
pub const POINTER_SIZE: usize = 4;
#[cfg(target_pointer_width = "64")]
pub const POINTER_SIZE: usize = 8;

// pub unsafe extern "system" fn GetForegroundWindow() -> HWND
pub fn get_foreground_window() -> windef::HWND {
    unsafe { user32::GetForegroundWindow() }
}

// pub unsafe extern "system" fn GetWindowThreadProcessId(hWnd: HWND, lpdwProcessId: LPDWORD) -> DWORD
pub fn get_window_process_id(hwnd: windef::HWND) -> minwindef::DWORD {
    let mut process_id: minwindef::DWORD = 0;
    unsafe {
        user32::GetWindowThreadProcessId(hwnd, &mut process_id);
    }
    process_id
}

// pub unsafe extern "system" fn OpenProcess(dwDesiredAccess: DWORD, bInheritHandle: BOOL, dwProcessId: DWORD) -> HANDLE
pub fn open_process(process_id: minwindef::DWORD) -> winnt::HANDLE {
    unsafe {
        kernel32::OpenProcess(winnt::PROCESS_QUERY_INFORMATION | winnt::PROCESS_VM_READ,
                              minwindef::FALSE,
                              process_id)
    }
}

// pub unsafe extern "system" fn GetWindowTextW(hWnd: HWND, lpString: LPWSTR, nMaxCount: c_int) -> c_int
pub fn get_window_text(hwnd: windef::HWND) -> String {
    let max_char_count: usize = 300;
    let mut buffer: Vec<winnt::WCHAR> = Vec::with_capacity(max_char_count);
    unsafe {
        let char_count = user32::GetWindowTextW(hwnd,
                                                buffer.as_mut_ptr() as winnt::LPWSTR,
                                                max_char_count as c_int) as usize;
        if char_count == 0 {
            return String::new();
        }
        buffer.set_len(char_count);
    }
    String::from_utf16_lossy(&buffer)
}

// pub unsafe extern "system" fn ReadProcessMemory(hProcess: HANDLE, lpBaseAddress: LPCVOID, lpBuffer: LPVOID, nSize: SIZE_T, lpNumberOfBytesRead: *mut SIZE_T) -> BOOL
pub fn read_process_memory_raw(process_handle: winnt::HANDLE, base_address: minwindef::LPCVOID, buffer: minwindef::LPVOID, size: usize) -> bool {
    unsafe {
        let mut bytes_read: SIZE_T = 0;
        let result = kernel32::ReadProcessMemory(process_handle,
                                                 base_address,
                                                 buffer as minwindef::LPVOID,
                                                 size as basetsd::SIZE_T,
                                                 &mut bytes_read);
        result != 0
    }
}

pub fn read_process_memory<T>(process_handle: winnt::HANDLE, base_address: minwindef::LPCVOID, buffer: *mut T) -> bool {
    read_process_memory_raw(process_handle,
                            base_address,
                            buffer as minwindef::LPVOID,
                            mem::size_of::<T>())
}

// pub unsafe extern "system" fn IsWow64Process(hProcess: HANDLE, Wow64Process: PBOOL) -> BOOL
pub fn is_wow64_process(process_handle: HANDLE) -> bool {
    let mut result: minwindef::BOOL = minwindef::FALSE;
    unsafe {
        kernel32::IsWow64Process(process_handle, &mut result);
    }
    result != minwindef::FALSE
}

// pub unsafe extern "system" fn IsImmersiveProcess(hProcess: HANDLE) -> BOOL
pub fn is_immersive_process(process_handle: HANDLE) -> bool {
    unsafe { user32::IsImmersiveProcess(process_handle) != minwindef::FALSE }
}

// pub unsafe extern "system" fn CloseHandle(hObject: HANDLE) -> BOOL
pub fn close_handle(handle: HANDLE) {
    unsafe {
        kernel32::CloseHandle(handle);
    }
}

// type WNDENUMPROC = Option<unsafe  extern "system" fn(HWND, LPARAM) -> BOOL>;
// pub unsafe extern "system" fn EnumChildWindows(hwndParent: HWND, lpEnumFunc: WNDENUMPROC, lpParam: LPARAM) -> BOOL
pub fn enum_child_windows(parent_window: windef::HWND, callback: winuser::WNDENUMPROC, lparam: minwindef::LPARAM) -> bool {
    unsafe { user32::EnumChildWindows(parent_window, callback, lparam) != minwindef::FALSE }
}

pub fn to_wide_chars(s: &str) -> Vec<u16> {
    let mut v: Vec<u16> = s.encode_utf16().collect();
    v.push(0);
    v
}

// the first few slots in WNDCLASS.cbWndExtra is reserved for win32helper
pub const WINDOW_EXTRA_SLOT_WND_PROC: c_int = 0;
pub const WINDOW_EXTRA_SLOT_USER: c_int = POINTER_SIZE as c_int;

pub fn get_window_extra(hwnd: windef::HWND, index: c_int) -> basetsd::LONG_PTR {
    unsafe { user32::GetWindowLongPtrW(hwnd, index) }
}

pub fn set_window_extra(hwnd: windef::HWND, index: c_int, value: basetsd::LONG_PTR) -> basetsd::LONG_PTR {
    unsafe { user32::SetWindowLongPtrW(hwnd, index, value) }
}

fn window_extra_real_index(index: c_int) -> c_int {
    if index < 0 {
        index
    } else {
        index + WINDOW_EXTRA_SLOT_USER
    }
}

// pub unsafe extern "system" fn GetWindowLongPtrW(hWnd: HWND, nIndex: c_int) -> LONG_PTR
pub fn get_window_long_ptr(hwnd: windef::HWND, index: c_int) -> basetsd::LONG_PTR {
    get_window_extra(hwnd, window_extra_real_index(index))
}

// pub unsafe extern "system" fn SetWindowLongPtrW(hWnd: HWND, nIndex: c_int, dwNewLong: LONG_PTR) -> LONG_PTR
pub fn set_window_long_ptr(hwnd: windef::HWND, index: c_int, value: basetsd::LONG_PTR) -> bool {
    set_last_error(0);
    let result = set_window_extra(hwnd, window_extra_real_index(index), value);
    if result == 0 {
        let last_error = get_last_error();
        if last_error != 0 {
            return false;
        }
    }
    return true;
}

// pub unsafe extern "system" fn GetModuleHandleW(lpModuleName: LPCWSTR) -> HMODULE
pub fn get_module_handle(module_name: winnt::LPCWSTR) -> minwindef::HMODULE {
    unsafe { kernel32::GetModuleHandleW(module_name) }
}

pub fn get_current_instance() -> minwindef::HINSTANCE {
    get_module_handle(ptr::null()) as minwindef::HINSTANCE
}

// pub unsafe extern "system" fn PostQuitMessage(nExitCode: c_int)
pub fn post_quit_message(exit_code: c_int) {
    unsafe { user32::PostQuitMessage(exit_code) }
}

// pub unsafe extern "system" fn DefWindowProcW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT
pub fn def_window_proc(hwnd: windef::HWND, msg: minwindef::UINT, wparam: minwindef::WPARAM, lparam: LPARAM) -> minwindef::LRESULT {
    unsafe { user32::DefWindowProcW(hwnd, msg, wparam, lparam) }
}

// pub unsafe extern "system" fn GetMessageW(lpMsg: LPMSG, hWnd: HWND, wMsgFilterMin: UINT, wMsgFilterMax: UINT) -> BOOL
// pub unsafe extern "system" fn DispatchMessageW(lpmsg: *const MSG) -> LRESULT
pub fn message_loop() {
    unsafe {
        let mut msg: winuser::MSG = mem::zeroed();
        while user32::GetMessageW(&mut msg, ptr::null_mut(), 0, 0) != minwindef::FALSE {
            user32::DispatchMessageW(&msg);
        }
    }
}

// pub unsafe extern "system" fn AllocConsole() -> BOOL
pub fn alloc_console() -> bool {
    unsafe { kernel32::AllocConsole() != minwindef::FALSE }
}

fn to_winapi_bool(x: bool) -> minwindef::BOOL {
    if x { minwindef::TRUE } else { minwindef::FALSE }
}

// pub unsafe extern "system" fn CreateWaitableTimerW(lpTimerAttributes: LPSECURITY_ATTRIBUTES, bManualReset: BOOL, lpTimerName: LPCWSTR) -> HANDLE
pub fn create_waitable_timer(manual_reset: bool) -> winnt::HANDLE {
    unsafe { kernel32::CreateWaitableTimerW(ptr::null_mut(), to_winapi_bool(manual_reset), ptr::null()) }
}

// type PTIMERAPCROUTINE = Option<unsafe extern "system" fn(lpArgToCompletionRoutine: LPVOID, dwTimerLowValue: DWORD, dwTimerHighValue: DWORD)>;
// pub unsafe extern "system" fn SetWaitableTimer(hTimer: HANDLE, lpDueTime: *const LARGE_INTEGER, lPeriod: LONG,
//                                                pfnCompletionRoutine: PTIMERAPCROUTINE, lpArgToCompletionRoutine: LPVOID, fResume: BOOL) -> BOOL
pub fn set_waitable_timer(timer_handle: winnt::HANDLE,
                          due_time: *const winnt::LARGE_INTEGER,
                          period: winnt::LONG,
                          callback: synchapi::PTIMERAPCROUTINE,
                          callback_context: minwindef::LPVOID,
                          resume_system: bool)
                          -> bool {
    unsafe {
        kernel32::SetWaitableTimer(timer_handle,
                                   due_time,
                                   period,
                                   callback,
                                   callback_context,
                                   to_winapi_bool(resume_system)) != minwindef::FALSE
    }
}

// pub unsafe extern "system" fn CancelWaitableTimer(hTimer: HANDLE) -> BOOL
pub fn cancel_waitable_timer(timer_handle: winnt::HANDLE) -> bool {
    unsafe { kernel32::CancelWaitableTimer(timer_handle) != minwindef::FALSE }
}

// pub unsafe extern "system" fn CreateEventW(lpEventAttributes: LPSECURITY_ATTRIBUTES, bManualReset: BOOL, bInitialState: BOOL, lpName: LPCWSTR) -> HANDLE
pub fn create_event(manual_reset: bool, initial_state: bool) -> winnt::HANDLE {
    unsafe {
        kernel32::CreateEventW(ptr::null_mut(),
                               to_winapi_bool(manual_reset),
                               to_winapi_bool(initial_state),
                               ptr::null())
    }
}

// pub unsafe extern "system" fn SetEvent(hEvent: HANDLE) -> BOOL
pub fn set_event(event_handle: winnt::HANDLE) {
    unsafe { kernel32::SetEvent(event_handle) };
}

// pub unsafe extern "system" fn WaitForSingleObjectEx(hHandle: HANDLE, dwMilliseconds: DWORD, bAlertable: BOOL) -> DWORD
pub fn wait_for_single_object_ex(handle: winnt::HANDLE, milliseconds: minwindef::DWORD) -> bool {
    unsafe { kernel32::WaitForSingleObjectEx(handle, milliseconds, minwindef::TRUE) == winbase::WAIT_OBJECT_0 }
}

// pub unsafe extern "system" fn CreateMutexW(lpMutexAttributes: LPSECURITY_ATTRIBUTES, bInitialOwner: BOOL, lpName: LPCWSTR) -> HANDLE
pub fn create_mutex(initial_owner: bool, name: &str) -> winnt::HANDLE {
    let name_vec = to_wide_chars(name);
    unsafe {
        kernel32::CreateMutexW(ptr::null_mut(),
                               to_winapi_bool(initial_owner),
                               name_vec.as_ptr())
    }
}

// pub unsafe extern "system" fn GetLastError() -> DWORD
pub fn get_last_error() -> minwindef::DWORD {
    unsafe { kernel32::GetLastError() }
}

// pub unsafe extern "system" fn SetLastError(dwErrCode: DWORD)
pub fn set_last_error(value: minwindef::DWORD) {
    unsafe { kernel32::SetLastError(value) };
}

pub fn is_app_already_runniing(name: &str) -> bool {
    let handle = create_mutex(false, name);

    // - if the function fails, the return value is NULL.
    // - if the named mutex already exists before this function call, the return value is a non-null handle to the existing object,
    //   GetLastError returns ERROR_ALREADY_EXISTS, bInitialOwner is ignored.
    (handle == ptr::null_mut() || get_last_error() == winerror::ERROR_ALREADY_EXISTS)
}

// wparam for WM_POWERBROADCAST
pub const PBT_POWERSETTINGCHANGE: minwindef::WPARAM = 0x0218;

#[repr(C)]
pub struct POWERBROADCAST_SETTING {
    pub power_setting: guiddef::GUID,
    pub length: minwindef::DWORD,
    pub data: minwindef::DWORD, // [u8; *],
}

// wparam for WM_WTSSESSION_CHANGE
pub const WTS_SESSION_LOCK: minwindef::WPARAM = 7;
pub const WTS_SESSION_UNLOCK: minwindef::WPARAM = 8;

// DEFINE_GUID!(GUID_SESSION_USER_PRESENCE,  0x3c0f4548, 0xc03f, 0x4c4d, 0xb9, 0xf2, 0x23, 0x7e, 0xde, 0x68, 0x63, 0x76);
pub const GUID_SESSION_USER_PRESENCE: GUID = GUID {
    Data1: 0x3c0f4548,
    Data2: 0xc03f,
    Data3: 0x4c4d,
    Data4: [0xb9, 0xf2, 0x23, 0x7e, 0xde, 0x68, 0x63, 0x76],
};

// DEFINE_GUID!(GUID_SESSION_DISPLAY_STATUS, 0x2b84c20e, 0xad23, 0x4ddf, 0x93, 0xdb, 0x05, 0xff, 0xbd, 0x7e, 0xfc, 0xa5);
pub const GUID_SESSION_DISPLAY_STATUS: GUID = GUID {
    Data1: 0x2b84c20e,
    Data2: 0xad23,
    Data3: 0x4ddf,
    Data4: [0x93, 0xdb, 0x05, 0xff, 0xbd, 0x7e, 0xfc, 0xa5],
};

pub fn is_equal_guid(x: &GUID, y: &GUID) -> bool {
    x.Data1 == y.Data1 && x.Data2 == y.Data2 && x.Data3 == y.Data3 && x.Data4 == y.Data4
}

// HPOWERNOTIFY WINAPI RegisterPowerSettingNotification(
//   _In_ HANDLE  hRecipient,
//   _In_ LPCGUID PowerSettingGuid,
//   _In_ DWORD   Flags
// );

pub type HPOWERNOTIFY = winnt::HANDLE;

#[allow(non_snake_case)]
#[link(name = "user32")]
extern "system" {
    pub fn RegisterPowerSettingNotification(hRecipient: winnt::HANDLE, PowerSettingGuid: &GUID, Flags: minwindef::DWORD) -> HPOWERNOTIFY;
}

pub fn register_power_setting_notification(recipient: winnt::HANDLE, setting: &GUID, flags: minwindef::DWORD) -> HPOWERNOTIFY {
    unsafe { RegisterPowerSettingNotification(recipient, setting, flags) }
}

// flags for register_power_setting_notification
pub const DEVICE_NOTIFY_WINDOW_HANDLE: minwindef::DWORD = 0;
// pub const DEVICE_NOTIFY_SERVICE_HANDLE: minwindef::DWORD = 1;

// BOOL WTSRegisterSessionNotification(
//   _In_ HWND  hWnd,
//   _In_ DWORD dwFlags
// );
#[allow(non_snake_case)]
#[link(name = "wtsapi32")]
extern "system" {
    pub fn WTSRegisterSessionNotification(hWnd: windef::HWND, dwFlags: minwindef::DWORD) -> minwindef::BOOL;
}

pub fn wts_register_session_notification(hwnd: windef::HWND, flags: minwindef::DWORD) -> bool {
    unsafe { WTSRegisterSessionNotification(hwnd, flags) != minwindef::FALSE }
}

// flags for wts_register_session_notification
pub const NOTIFY_FOR_THIS_SESSION: minwindef::DWORD = 0;
// pub const NOTIFY_FOR_ALL_SESSIONS: minwindef::DWORD = 1;

// pub unsafe extern "system" fn GetLocalTime(lpSystemTime: LPSYSTEMTIME)
pub fn get_local_time() -> minwinbase::SYSTEMTIME {
    unsafe {
        let mut now: minwinbase::SYSTEMTIME = mem::zeroed();
        kernel32::GetLocalTime(&mut now);
        return now;
    }
}
