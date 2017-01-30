extern crate winapi;
extern crate user32;
extern crate kernel32;

use self::winapi::*;
use ntdll;
use std::mem;
use std::ptr;

use super::*;

pub fn get_process_peb_address(process_handle: winnt::HANDLE) -> winnt::PVOID {
    let mut basic_info: ntdll::PROCESS_BASIC_INFORMATION = unsafe { mem::zeroed() };
    if ntdll::nt_query_information_process::<ntdll::PROCESS_BASIC_INFORMATION>(process_handle,
                                                                               ntdll::PROCESSINFOCLASS::ProcessBasicInformation,
                                                                               &mut basic_info) {
        basic_info.PebBaseAddress
    } else {
        ptr::null_mut()
    }
}

pub fn get_process_peb_address_wow32(process_handle: winnt::HANDLE) -> winnt::PVOID {
    let mut peb_address: winnt::PVOID = ptr::null_mut();
    if ntdll::nt_query_information_process::<winnt::PVOID>(process_handle,
                                                           ntdll::PROCESSINFOCLASS::ProcessWow64Information,
                                                           &mut peb_address) {
        peb_address
    } else {
        ptr::null_mut()
    }
}

pub fn get_process_command_line(process_handle: winnt::HANDLE) -> String {
    if is_wow64_process(process_handle) {
        return get_process_command_line_32(process_handle);
    }
    let empty = String::new();

    let peb_address = get_process_peb_address(process_handle);
    let mut peb: ntdll::PROCESS_ENVIRONMENT_BLOCK = unsafe { mem::zeroed() };
    if !read_process_memory::<ntdll::PROCESS_ENVIRONMENT_BLOCK>(process_handle, peb_address, &mut peb) {
        return empty;
    }

    let mut process_parameters: ntdll::RTL_USER_PROCESS_PARAMETERS = unsafe { mem::zeroed() };
    if !read_process_memory::<ntdll::RTL_USER_PROCESS_PARAMETERS>(process_handle,
                                                                  peb.ProcessParameters,
                                                                  &mut process_parameters) {
        return empty;
    }

    let byte_count = process_parameters.CommandLine.Length as usize;
    let char_count = byte_count / 2;
    let mut buffer: Vec<winnt::WCHAR> = Vec::with_capacity(char_count);
    unsafe {
        buffer.set_len(char_count);
    }
    if !read_process_memory_raw(process_handle,
                                process_parameters.CommandLine.Buffer as minwindef::LPCVOID,
                                buffer.as_mut_ptr() as minwindef::LPVOID,
                                byte_count) {
        return empty;
    }
    String::from_utf16_lossy(&buffer)
}

pub fn get_process_command_line_32(process_handle: winnt::HANDLE) -> String {
    let empty = String::new();

    let peb_address = get_process_peb_address_wow32(process_handle);
    let mut peb: ntdll::PROCESS_ENVIRONMENT_BLOCK_32 = unsafe { mem::zeroed() };
    if !read_process_memory::<ntdll::PROCESS_ENVIRONMENT_BLOCK_32>(process_handle, peb_address, &mut peb) {
        return empty;
    }

    let mut process_parameters: ntdll::RTL_USER_PROCESS_PARAMETERS_32 = unsafe { mem::zeroed() };
    if !read_process_memory::<ntdll::RTL_USER_PROCESS_PARAMETERS_32>(process_handle,
                                                                     peb.ProcessParameters as minwindef::LPCVOID,
                                                                     &mut process_parameters) {
        return empty;
    }

    let byte_count = process_parameters.CommandLine.Length as usize;
    let char_count = byte_count / 2;
    let mut buffer: Vec<winnt::WCHAR> = Vec::with_capacity(char_count);
    unsafe {
        buffer.set_len(char_count);
    }
    if !read_process_memory_raw(process_handle,
                                process_parameters.CommandLine.Buffer as minwindef::LPCVOID,
                                buffer.as_mut_ptr() as minwindef::LPVOID,
                                byte_count) {
        return empty;
    }
    String::from_utf16_lossy(&buffer)
}

pub fn get_universal_app(window_handle: &mut windef::HWND, process_id: &mut minwindef::DWORD) {

    struct EnumUniversaApplParameter {
        window_handle: windef::HWND,
        process_id: minwindef::DWORD,
        child_window: windef::HWND,
        child_process: minwindef::DWORD,
    }

    unsafe extern "system" fn enum_universal_app_callback(child_window: windef::HWND, lparam: minwindef::LPARAM) -> minwindef::BOOL {
        let mut parameter = &mut *(lparam as minwindef::LPVOID as *mut EnumUniversaApplParameter);
        let child_process = get_window_process_id(child_window);
        if child_process != 0 && child_process != parameter.process_id {
            parameter.child_window = child_window;
            parameter.child_process = child_process;
            return minwindef::FALSE;
        }
        return minwindef::TRUE;
    }

    let mut parameter: EnumUniversaApplParameter = unsafe { mem::zeroed() };
    parameter.window_handle = *window_handle;
    parameter.process_id = *process_id;

    let address: *mut EnumUniversaApplParameter = &mut parameter;
    enum_child_windows(*window_handle,
                       Some(enum_universal_app_callback),
                       address as minwindef::LPARAM);

    if parameter.child_process == 0 {
        panic!("Error trying to find the universal app for host window {:?}",
               get_window_text(*window_handle));
    }

    *window_handle = parameter.child_window;
    *process_id = parameter.child_process;
}
