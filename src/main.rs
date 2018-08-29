mod win32helper;

mod mainframe;
mod logger;
use self::mainframe::*;
use self::logger::*;

use std::io::prelude::*;
use std::net;
use std::thread;
use std::process::Command;

const CHECK_INTERNVAL_IN_SECONDS: u32 = 10;
const FLUSH_INTERVAL_IN_MINUTES: u32 = 15;
const LISTENING_PORT: u16 = 50080;

fn main() {
    let logger = Logger::new(CHECK_INTERNVAL_IN_SECONDS, FLUSH_INTERVAL_IN_MINUTES);
    set_logger(|| Box::new(logger));

    start_web_server();

    setup_periodic_callback(CHECK_INTERNVAL_IN_SECONDS, get_foreground_app);
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

fn start_web_server() {
    thread::spawn(|| {
        let ip_address = get_local_ip();
        println!("Listen on {}:{}", &ip_address, LISTENING_PORT);
        let listener = net::TcpListener::bind((ip_address.as_str(), LISTENING_PORT)).unwrap();
        for stream in listener.incoming() {
            handle_connection(stream.unwrap());
        }
    });
}

fn handle_connection(mut stream: net::TcpStream) {
    let mut buffer = [0; 512];

    stream.read(&mut buffer).unwrap();

    let header = "HTTP/1.1 200 OK\r\n\r\n";
    let (window_title, command_line) = get_last_entry();
    let response = format!("{}<p/>{}<p/>{}", header, window_title, command_line);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

pub fn get_local_ip() -> String {
    //
    // parse output from "netsh.exe interface ipv4 show addresses"
    //
    //     DHCP enabled:    Yes
    //     IP Address:      192.168.1.100
    //
    let output = Command::new("netsh")
        .arg("interface")
        .arg("ipv4")
        .arg("show")
        .arg("addresses")
        .output()
        .expect("failed to execute netsh");
    let stdout = String::from_utf8(output.stdout).unwrap();

    let lines: Vec<&str> = stdout.split("\r\n").collect();
    let mut found_dhcp = false;
    for line in lines {
        if found_dhcp && line.contains("IP Address") {
            let parts: Vec<&str> = line.split(":").collect();
            return parts[1].trim_left().to_string();
        }
        if line.contains("DHCP enabled") && line.contains("Yes") {
            found_dhcp = true;
        }
    }
    "".to_string()
}