extern crate winapi;

use self::winapi::{
    ctypes, shared::basetsd, shared::minwindef, shared::windef, um::winnt, um::winuser,
};

use std::mem;
use std::ptr;

use super::ntdll;
use super::*;

pub fn get_process_peb_address(process_handle: winnt::HANDLE) -> winnt::PVOID {
    let mut basic_info: ntdll::PROCESS_BASIC_INFORMATION = unsafe { mem::zeroed() };
    if ntdll::nt_query_information_process::<ntdll::PROCESS_BASIC_INFORMATION>(
        process_handle,
        ntdll::PROCESSINFOCLASS::ProcessBasicInformation,
        &mut basic_info,
    ) {
        basic_info.PebBaseAddress
    } else {
        ptr::null_mut()
    }
}

pub fn get_process_peb_address_wow32(process_handle: winnt::HANDLE) -> winnt::PVOID {
    let mut peb_address: winnt::PVOID = ptr::null_mut();
    if ntdll::nt_query_information_process::<winnt::PVOID>(
        process_handle,
        ntdll::PROCESSINFOCLASS::ProcessWow64Information,
        &mut peb_address,
    ) {
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
    if !read_process_memory::<ntdll::PROCESS_ENVIRONMENT_BLOCK>(
        process_handle,
        peb_address,
        &mut peb,
    ) {
        return empty;
    }

    let mut process_parameters: ntdll::RTL_USER_PROCESS_PARAMETERS = unsafe { mem::zeroed() };
    if !read_process_memory::<ntdll::RTL_USER_PROCESS_PARAMETERS>(
        process_handle,
        peb.ProcessParameters,
        &mut process_parameters,
    ) {
        return empty;
    }

    let byte_count = process_parameters.CommandLine.Length as usize;
    let char_count = byte_count / 2;
    let mut buffer: Vec<winnt::WCHAR> = Vec::with_capacity(char_count);
    unsafe {
        buffer.set_len(char_count);
    }
    if !read_process_memory_raw(
        process_handle,
        process_parameters.CommandLine.Buffer as minwindef::LPCVOID,
        buffer.as_mut_ptr() as minwindef::LPVOID,
        byte_count,
    ) {
        return empty;
    }
    String::from_utf16_lossy(&buffer)
}

pub fn get_process_command_line_32(process_handle: winnt::HANDLE) -> String {
    let empty = String::new();

    let peb_address = get_process_peb_address_wow32(process_handle);
    let mut peb: ntdll::PROCESS_ENVIRONMENT_BLOCK_32 = unsafe { mem::zeroed() };
    if !read_process_memory::<ntdll::PROCESS_ENVIRONMENT_BLOCK_32>(
        process_handle,
        peb_address,
        &mut peb,
    ) {
        return empty;
    }

    let mut process_parameters: ntdll::RTL_USER_PROCESS_PARAMETERS_32 = unsafe { mem::zeroed() };
    if !read_process_memory::<ntdll::RTL_USER_PROCESS_PARAMETERS_32>(
        process_handle,
        peb.ProcessParameters as minwindef::LPCVOID,
        &mut process_parameters,
    ) {
        return empty;
    }

    let byte_count = process_parameters.CommandLine.Length as usize;
    let char_count = byte_count / 2;
    let mut buffer: Vec<winnt::WCHAR> = Vec::with_capacity(char_count);
    unsafe {
        buffer.set_len(char_count);
    }
    if !read_process_memory_raw(
        process_handle,
        process_parameters.CommandLine.Buffer as minwindef::LPCVOID,
        buffer.as_mut_ptr() as minwindef::LPVOID,
        byte_count,
    ) {
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

    unsafe extern "system" fn enum_universal_app_callback(
        child_window: windef::HWND,
        lparam: minwindef::LPARAM,
    ) -> minwindef::BOOL {
        let parameter = &mut *(lparam as minwindef::LPVOID as *mut EnumUniversaApplParameter);
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
    enum_child_windows(
        *window_handle,
        Some(enum_universal_app_callback),
        address as minwindef::LPARAM,
    );

    if parameter.child_process == 0 {
        println!(
            "Error trying to find the universal app for host window {:?}",
            get_window_text(*window_handle)
        );
        return;
    }

    *window_handle = parameter.child_window;
    *process_id = parameter.child_process;
}

// fn wnd_proc(hwnd: windef::HWND, msg: minwindef::UINT, wparam: minwindef::WPARAM, lparam: LPARAM) -> minwindef::LRESULT
pub trait WindowTrait {
    fn wnd_proc(
        &mut self,
        hwnd: windef::HWND,
        msg: minwindef::UINT,
        wparam: minwindef::WPARAM,
        lparam: minwindef::LPARAM,
    ) -> minwindef::LRESULT;
}

// pub unsafe extern "system" fn RegisterClassW(lpWndClass: *const WNDCLASSW) -> ATOM
// pub unsafe extern "system" fn CreateWindowExW(dwExStyle: DWORD, lpClassName: LPCWSTR, lpWindowName: LPCWSTR, dwStyle: DWORD, x: c_int, y: c_int, nWidth: c_int, nHeight: c_int,
//                                               hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: LPVOID) -> HWND
pub fn create_window<W>(
    window: &mut W,
    class_name: &str,
    window_name: &str,
    style: minwindef::DWORD,
    instance_handle: minwindef::HINSTANCE,
    wnd_extra: ctypes::c_int,
) -> windef::HWND
where
    W: WindowTrait,
{
    unsafe extern "system" fn static_wnd_proc<W>(
        hwnd: windef::HWND,
        msg: minwindef::UINT,
        wparam: minwindef::WPARAM,
        lparam: minwindef::LPARAM,
    ) -> minwindef::LRESULT
    where
        W: WindowTrait,
    {
        if msg == winuser::WM_NCCREATE {
            let create_structure = lparam as *const winuser::CREATESTRUCTW;
            let window = (*create_structure).lpCreateParams;
            set_window_extra(
                hwnd,
                WINDOW_EXTRA_SLOT_WND_PROC,
                window as basetsd::LONG_PTR,
            ); // lparam was passed from CreateWindow() call, as self pointer to WindowTrait
        }

        let window_extra = get_window_extra(hwnd, WINDOW_EXTRA_SLOT_WND_PROC);
        if window_extra == 0 {
            return def_window_proc(hwnd, msg, wparam, lparam);
        }

        let window: &mut W = mem::transmute(window_extra);
        window.wnd_proc(hwnd, msg, wparam, lparam)
    }

    let class_name_vec = to_wide_chars(class_name);
    let window_name_vec = to_wide_chars(window_name);

    let mut wnd_class: winuser::WNDCLASSW = unsafe { mem::zeroed() };
    wnd_class.lpfnWndProc = Some(static_wnd_proc::<W>);
    wnd_class.hInstance = instance_handle;
    wnd_class.hbrBackground = winuser::COLOR_BACKGROUND as windef::HBRUSH;
    wnd_class.lpszClassName = class_name_vec.as_ptr();
    wnd_class.cbWndExtra = wnd_extra + WINDOW_EXTRA_SLOT_USER;

    unsafe {
        if winuser::RegisterClassW(&wnd_class) == 0 {
            return ptr::null_mut();
        }

        winuser::CreateWindowExW(
            0,
            class_name_vec.as_ptr(),
            window_name_vec.as_ptr(),
            style,
            winuser::CW_USEDEFAULT,
            winuser::CW_USEDEFAULT,
            winuser::CW_USEDEFAULT,
            winuser::CW_USEDEFAULT,
            ptr::null_mut(), // hWndParent
            ptr::null_mut(), // hMenu
            instance_handle,
            window as *mut W as minwindef::LPVOID,
        ) // Passed to WM_NCCREATE as CREATESTRUCT.lpCreateParams
    }
}
