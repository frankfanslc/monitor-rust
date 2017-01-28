#![allow(non_snake_case)]

extern crate winapi;
extern crate user32;
extern crate kernel32;

use self::winapi::*;
use std::mem;

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

/*
 *
 */
pub fn NT_SUCCESS(status: ntdef::NTSTATUS) -> bool {
    status >= 0
}

#[derive(Debug)]
#[repr(C)]
pub enum PROCESSINFOCLASS {
    ProcessBasicInformation = 0,
}

// typedef NTSTATUS(NTAPI* type_NtQueryInformationProcess)(
//         _In_ HANDLE ProcessHandle,
//         _In_ PROCESSINFOCLASS ProcessInformationClass,
//         _Out_ PVOID ProcessInformation,
//         _In_ ULONG ProcessInformationLength,
//         _Out_opt_ PULONG ReturnLength);

#[link(name = "ntdll")]
extern "system" {
    pub fn NtQueryInformationProcess(
        ProcessHandle: winnt::HANDLE,
        ProcessInformationClass: PROCESSINFOCLASS,
        ProcessInformation: winnt::PVOID,
        ProcessInformationLength: minwindef::ULONG,
        ReturnLength: &mut minwindef::ULONG)
        -> ntdef::NTSTATUS;
}


#[repr(C)]
pub struct PROCESS_BASIC_INFORMATION {
    Reserved1: winnt::PVOID,
    PebBaseAddress: winnt::PVOID, // PPEB,
    Reserved2: [winnt::PVOID; 2],
    UniqueProcessId: basetsd::ULONG_PTR,
    Reserved3: winnt::PVOID,
}

pub fn get_process_peb_address(process_handle: winnt::HANDLE) {
    // let mut basic_information: PROCESS_BASIC_INFORMATION;
    let buffer_size = mem::size_of::<PROCESS_BASIC_INFORMATION>();
    let mut buffer: Vec<u8> = Vec::with_capacity(buffer_size);
    let mut return_length: minwindef::ULONG = 0;

    unsafe {
        let status = NtQueryInformationProcess(process_handle, PROCESSINFOCLASS::ProcessBasicInformation,
                buffer.as_mut_ptr() as winnt::PVOID, buffer_size as minwindef::ULONG, &mut return_length);
        if !NT_SUCCESS(status) {
            return;
        }
        let basic_info = & *(buffer.as_ptr() as *const PROCESS_BASIC_INFORMATION);
        println!("peb {:?}", basic_info.PebBaseAddress);
    }
}
