#![windows_subsystem = "windows"]

mod win32helper;

mod logger;
mod mainframe;
use self::logger::*;
use self::mainframe::*;

use std::io::prelude::*;
use std::net;
use std::thread;

pub const CHECK_INTERNVAL_IN_SECONDS: u32 = 10;
pub const FLUSH_INTERVAL_IN_MINUTES: u32 = 15;
const LISTENING_PORT: u16 = 50080;

fn main() {
    if win32helper::is_app_already_runniing("Local\\{AB2F0A5E-FAA2-4664-B3C2-25D3984F0A20}") {
        return;
    }

    // let console_result = win32helper::alloc_console();
    // println!("alloc_console: {:?}", console_result);

    let logger = Logger::new(CHECK_INTERNVAL_IN_SECONDS, FLUSH_INTERVAL_IN_MINUTES);
    set_logger(|| Box::new(logger));

    start_web_server();

    MainFrame::new();

    win32helper::message_loop();
}

pub fn get_foreground_app() {
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

fn start_web_server() {
    thread::spawn(|| {
        let ip_address = win32helper::get_local_ip();
        println!("Listen on {}:{}", &ip_address, LISTENING_PORT);
        let listener = net::TcpListener::bind((ip_address.as_str(), LISTENING_PORT)).unwrap();
        for stream in listener.incoming() {
            match stream {
                Ok(t) => handle_connection(t),
                Err(_) => return,
            }
        }
    });
}

fn handle_connection(mut stream: net::TcpStream) {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();
    let get = b"GET / HTTP/1.1\r\n";
    if !buffer.starts_with(get) {
        return;
    }

    let http_header = "HTTP/1.1 200 OK\r\n\r\n";
    let html_head = "<head><title>Status</title><style>body{font-size:50px}</style></head>";
    let (window_title, command_line) = get_last_entry();
    let response = format!(
        "{}{}<p/>{}<p/>{}",
        http_header, html_head, window_title, command_line
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
