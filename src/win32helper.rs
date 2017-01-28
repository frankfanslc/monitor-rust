extern crate winapi;
extern crate user32;
extern crate kernel32;

use self::winapi::*;
use ntdll;
use std::mem;
use std::ptr;

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

pub fn get_process_peb_address(process_handle: winnt::HANDLE) -> winnt::PVOID {
    let buffer_size = mem::size_of::<ntdll::PROCESS_BASIC_INFORMATION>();
    let mut buffer: Vec<u8> = Vec::with_capacity(buffer_size);
    let mut return_length: minwindef::ULONG = 0;

    unsafe {
        let status = ntdll::NtQueryInformationProcess(process_handle, ntdll::PROCESSINFOCLASS::ProcessBasicInformation,
                buffer.as_mut_ptr() as winnt::PVOID, buffer_size as minwindef::ULONG, &mut return_length);
        if !ntdll::NT_SUCCESS(status) {
            return ptr::null_mut();
        }
        let basic_info = & *(buffer.as_ptr() as *const ntdll::PROCESS_BASIC_INFORMATION);
        basic_info.PebBaseAddress
    }
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

pub fn get_process_command_line(process_handle: winnt::HANDLE) -> String {
    let empty = String::new();

    let peb_address = get_process_peb_address(process_handle);
    println!("peb   : {:?}", peb_address); 

    let mut peb: ntdll::PROCESS_ENVIRONMENT_BLOCK = unsafe { mem::zeroed() };
    if !read_process_memory::<ntdll::PROCESS_ENVIRONMENT_BLOCK>(process_handle, peb_address, &mut peb) {
        return empty;
    }
    println!("param : {:?}", peb.ProcessParameters);

    let mut process_parameters: ntdll::RTL_USER_PROCESS_PARAMETERS = unsafe { mem::zeroed() };
    if !read_process_memory::<ntdll::RTL_USER_PROCESS_PARAMETERS>(process_handle, peb.ProcessParameters, &mut process_parameters) {
        return empty;
    }
    println!("cmdln : {:?}", process_parameters.CommandLine.Buffer);

    let char_count = process_parameters.CommandLine.Length as usize;
    let mut buffer : Vec<winnt::WCHAR> = Vec::new();
    buffer.resize(char_count, 0);
    // let mut buffer : Vec<winnt::WCHAR> = Vec::with_capacity(char_count);
    // unsafe { buffer.set_len(char_count); }
    if !read_process_memory_raw(process_handle, process_parameters.CommandLine.Buffer as minwindef::LPCVOID,
                buffer.as_mut_ptr() as minwindef::LPVOID, char_count * 2) {
        return empty;
    }
    String::from_utf16_lossy(&buffer)
}
