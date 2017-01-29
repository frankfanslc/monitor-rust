extern crate winapi;
extern crate user32;
extern crate kernel32;

use self::winapi::*;
use std::mem;

pub use self::extra::*;
mod extra;

// pub unsafe extern "system" fn GetForegroundWindow() -> HWND
pub fn get_foreground_window() -> windef::HWND {
    unsafe { user32::GetForegroundWindow() }
}

// pub unsafe extern "system" fn GetWindowThreadProcessId(hWnd: HWND, lpdwProcessId: LPDWORD) -> DWORD
pub fn get_window_process_id(hwnd: windef::HWND) -> minwindef::DWORD {
    let mut process_id : minwindef::DWORD = 0;
    unsafe { user32::GetWindowThreadProcessId(hwnd, &mut process_id); }
    process_id
}

// pub unsafe extern "system" fn OpenProcess(dwDesiredAccess: DWORD, bInheritHandle: BOOL, dwProcessId: DWORD) -> HANDLE
pub fn open_process(process_id: DWORD) -> winnt::HANDLE {
    unsafe { kernel32::OpenProcess(winnt::PROCESS_QUERY_INFORMATION | winnt::PROCESS_VM_READ, minwindef::FALSE, process_id) }
}

// pub unsafe extern "system" fn GetWindowTextW(hWnd: HWND, lpString: LPWSTR, nMaxCount: c_int) -> c_int
pub fn get_window_text(hwnd: windef::HWND) -> String {
    let max_char_count : usize = 300;
    let mut buffer : Vec<winnt::WCHAR> = Vec::with_capacity(max_char_count);
    unsafe {
        let char_count = user32::GetWindowTextW(hwnd, buffer.as_mut_ptr() as winnt::LPWSTR, max_char_count as c_int) as usize;
        if char_count == 0 {
            return String::new();
        }
        buffer.set_len(char_count);
    }
    String::from_utf16_lossy(&buffer)
}

// pub unsafe extern "system" fn ReadProcessMemory(hProcess: HANDLE, lpBaseAddress: LPCVOID, lpBuffer: LPVOID, nSize: SIZE_T, lpNumberOfBytesRead: *mut SIZE_T) -> BOOL
pub fn read_process_memory_raw (
            process_handle: winnt::HANDLE,
            base_address: minwindef::LPCVOID,
            buffer: minwindef::LPVOID,
            size: usize)
            -> bool {
    unsafe {
        let mut bytes_read: SIZE_T = 0;
        let result = kernel32::ReadProcessMemory(process_handle, base_address,
                    buffer as minwindef::LPVOID, size as basetsd::SIZE_T, &mut bytes_read);
        result != 0
    }
}

pub fn read_process_memory<T> (
            process_handle: winnt::HANDLE,
            base_address: minwindef::LPCVOID,
            buffer: *mut T)
            -> bool {
    read_process_memory_raw(process_handle, base_address, buffer as minwindef::LPVOID, mem::size_of::<T>())
}

// pub unsafe extern "system" fn IsWow64Process(hProcess: HANDLE, Wow64Process: PBOOL) -> BOOL
pub fn is_wow64_process(process_handle: HANDLE) -> bool {
    let mut result: minwindef::BOOL = minwindef::FALSE;
    unsafe { kernel32::IsWow64Process(process_handle, &mut result); }
    result != minwindef::FALSE
}

// pub unsafe extern "system" fn IsImmersiveProcess(hProcess: HANDLE) -> BOOL
pub fn is_immersive_process(process_handle: HANDLE) -> bool {
    unsafe {
        user32::IsImmersiveProcess(process_handle) != minwindef::FALSE
    }
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
    unsafe {
        user32::EnumChildWindows(parent_window, callback, lparam) != minwindef::FALSE
    }
}
