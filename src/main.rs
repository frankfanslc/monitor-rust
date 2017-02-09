mod win32helper;

mod mainframe;
mod logger;
use self::mainframe::*;
use self::logger::*;
use std::ptr;

const CHECK_INTERNVAL_IN_SECONDS: u32 = 10;
const FLUSH_INTERVAL_IN_MINUTES: u32 = 15;

fn main() {
    let logger = Logger::new(CHECK_INTERNVAL_IN_SECONDS, FLUSH_INTERVAL_IN_MINUTES);
    set_logger(|| Box::new(logger));

    setup_periodic_callback(CHECK_INTERNVAL_IN_SECONDS,
                            timer_callback,
                            ptr::null_mut() as win32helper::TimerContext);
}

fn timer_callback(_: win32helper::TimerContext) {
    get_foreground_app();
}

fn get_foreground_app() {
    let mut window_handle = win32helper::get_foreground_window();
    let window_text = win32helper::get_window_text(window_handle);
    let mut process_id = win32helper::get_window_process_id(window_handle);
    let mut process_handle = win32helper::open_process(process_id);

    if win32helper::is_immersive_process(process_handle) {
        win32helper::close_handle(process_handle);
        win32helper::get_universal_app(&mut window_handle, &mut process_id);
        process_handle = win32helper::open_process(process_id);

        // Note: I'm not refreshing get_window_text() for universal app here, because if we do so,
        // the result seems always to be a static string like "Microsoft Edge", which is not useful.
        // Besides, previous result from the parent app already reads as "CreateFileW - Google Search
        // and 2 more pages ‎- Microsoft Edge", and I'd rather keep that.
    }

    let command_line = win32helper::get_process_command_line(process_handle);

    win32helper::output_timestamp();
    println!();
    println!("hwnd  : {:?}", window_handle);
    println!("pid   : {:?}", process_id);
    println!("text  : {}", window_text);
    println!("cmdln : {}", command_line);
    println!();

    log(window_text, command_line);
}
