use std::{thread, time};

mod win32helper;
mod ntdll;

fn main() {
    thread::sleep(time::Duration::from_secs(3));

    let mut window_handle = win32helper::get_foreground_window();
    let mut process_id = win32helper::get_window_process_id(window_handle);
    let mut process_handle = win32helper::open_process(process_id);

    if win32helper::is_immersive_process(process_handle) {
        win32helper::close_handle(process_handle);
        win32helper::get_universal_app(&mut window_handle, &mut process_id);
        process_handle = win32helper::open_process(process_id);
    }

    let window_text = win32helper::get_window_text(window_handle);
    let command_line = win32helper::get_process_command_line(process_handle);

    println!("hwnd  : {:?}", window_handle);
    println!("pid   : {:?}", process_id);
    println!("text  : {}",   window_text);
    println!("cmdln : {}",   command_line);
}
