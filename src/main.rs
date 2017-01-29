// extern crate winapi;
// use winapi::*;

mod win32helper;
mod ntdll;

fn main() {
    println!("Hello, world!");

    let window_handle = win32helper::get_foreground_window();
    // let process_id = 
    win32helper::get_window_process_id(window_handle);
    let process_id = 15620; // firefox
    let process_handle = win32helper::open_process(process_id);
    let window_text = win32helper::get_window_text(window_handle);
    let command_line = win32helper::get_process_command_line(process_handle);

    println!("hwnd  : {:?}", window_handle);
    println!("pid   : {:?}", process_id);
    println!("hProc : {:?}", process_handle);
    println!("text  : {}",   window_text);
    println!("cmdln : {}",   command_line);
}
