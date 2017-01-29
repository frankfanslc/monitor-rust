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

// pub unsafe extern "system" fn NtQueryInformationProcess(ProcessHandle: HANDLE, ProcessInformationClass: PROCESSINFOCLASS,
//      ProcessInformation: PVOID, ProcessInformationLength: ULONG, ReturnLength: &mut ULONG) -> NTSTATUS;
pub fn nt_query_information_process<T> (
            process_handle: winnt::HANDLE, 
            information_class: ntdll::PROCESSINFOCLASS,
            buffer: *mut T)
            -> bool {
    let mut return_length: minwindef::ULONG = 0;
    unsafe {
        let status = ntdll::NtQueryInformationProcess(process_handle, information_class,
                buffer as minwindef::LPVOID, mem::size_of::<T>() as minwindef::ULONG, &mut return_length);
        ntdll::NT_SUCCESS(status)
    }
}

pub fn get_process_peb_address(process_handle: winnt::HANDLE) -> winnt::PVOID {
    let mut basic_info: ntdll::PROCESS_BASIC_INFORMATION = unsafe { mem::zeroed() };
    if nt_query_information_process::<ntdll::PROCESS_BASIC_INFORMATION>(
                process_handle, ntdll::PROCESSINFOCLASS::ProcessBasicInformation, &mut basic_info) {
        basic_info.PebBaseAddress
    } else {
        ptr::null_mut()
    }
}

pub fn get_process_peb_address_wow32(process_handle: winnt::HANDLE) -> winnt::PVOID {
    let mut peb_address: winnt::PVOID = ptr::null_mut();
    if nt_query_information_process::<winnt::PVOID>(
                process_handle, ntdll::PROCESSINFOCLASS::ProcessWow64Information, &mut peb_address) {
        peb_address
    } else {
        ptr::null_mut()
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
    if is_wow64_process(process_handle) {
        return get_process_command_line_32(process_handle);
    }
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
    let mut buffer : Vec<winnt::WCHAR> = Vec::with_capacity(char_count);
    unsafe { buffer.set_len(char_count); }
    if !read_process_memory_raw(process_handle, process_parameters.CommandLine.Buffer as minwindef::LPCVOID,
                buffer.as_mut_ptr() as minwindef::LPVOID, char_count * 2) {
        return empty;
    }
    String::from_utf16_lossy(&buffer)
}

// pub unsafe extern "system" fn IsWow64Process(hProcess: HANDLE, Wow64Process: PBOOL) -> BOOL
pub fn is_wow64_process(process_handle: HANDLE) -> bool {
    let mut result: minwindef::BOOL = minwindef::FALSE;
    unsafe { kernel32::IsWow64Process(process_handle, &mut result); }
    result != minwindef::FALSE
}

pub fn get_process_command_line_32(process_handle: winnt::HANDLE) -> String {
    let empty = String::new();

    let peb_address = get_process_peb_address_wow32(process_handle);
    println!("peb   : {:?}", peb_address); 

    let mut peb: ntdll::PROCESS_ENVIRONMENT_BLOCK_32 = unsafe { mem::zeroed() };
    if !read_process_memory::<ntdll::PROCESS_ENVIRONMENT_BLOCK_32>(process_handle, peb_address, &mut peb) {
        return empty;
    }
    println!("param : {:?}", peb.ProcessParameters);

    let mut process_parameters: ntdll::RTL_USER_PROCESS_PARAMETERS_32 = unsafe { mem::zeroed() };
    if !read_process_memory::<ntdll::RTL_USER_PROCESS_PARAMETERS_32>(process_handle, peb.ProcessParameters as minwindef::LPCVOID, &mut process_parameters) {
        return empty;
    }
    println!("cmdln : {:?}", process_parameters.CommandLine.Buffer);

    let char_count = process_parameters.CommandLine.Length as usize;
    let mut buffer : Vec<winnt::WCHAR> = Vec::with_capacity(char_count);
    unsafe { buffer.set_len(char_count); }
    if !read_process_memory_raw(process_handle, process_parameters.CommandLine.Buffer as minwindef::LPCVOID,
                buffer.as_mut_ptr() as minwindef::LPVOID, char_count * 2) {
        return empty;
    }
    String::from_utf16_lossy(&buffer)
}
