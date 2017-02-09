extern crate winapi;

use self::winapi::*;
use super::win32helper;
use std::fs::File;
use std::mem;

#[derive(Debug)]
struct Entry {
    timestamp: minwinbase::SYSTEMTIME,
    duration_in_seconds: u32,
    window_tile: String,
    command_line: String,
}

#[derive(Debug)]
pub struct Logger {
    file: File,
    interval_in_seconds: u32,
    max_entries_before_flush: u32,
    count: u32,
    last_entry: Entry,
    entries: Vec<Entry>,
}

impl Logger {
    pub fn new(interval_in_seconds: u32, flush_interval_in_minutes: u32) -> Logger {

        use std::env;
        use std::fs::OpenOptions;
        use std::os::windows::fs::OpenOptionsExt;

        let file_name = env::var("LOCALAPPDATA").unwrap() + "\\findapp-monitor.log";

        let last_entry = Entry {
            timestamp: win32helper::get_local_time(),
            duration_in_seconds: 0,
            window_tile: String::new(),
            command_line: String::new(),
        };

        let max_entries_before_flush = flush_interval_in_minutes * 60 / interval_in_seconds;

        Logger {
            file: OpenOptions::new()
                .append(true)
                .share_mode(winnt::FILE_SHARE_READ)
                .open(file_name)
                .unwrap(),
            interval_in_seconds: interval_in_seconds,
            max_entries_before_flush: max_entries_before_flush,
            count: 0,
            last_entry: last_entry,
            entries: Vec::<Entry>::with_capacity(max_entries_before_flush as usize),
        }
    }

    pub fn add_entry(&mut self, window_tile: String, command_line: String) {
        self.count += 1;
        if self.count >= self.max_entries_before_flush {
            self.flush();
        }
        let entry = Entry {
            timestamp: win32helper::get_local_time(),
            duration_in_seconds: self.interval_in_seconds,
            window_tile: window_tile.to_owned(),
            command_line: command_line.to_owned(),
        };
        if self.last_entry.duration_in_seconds == 0 {
            self.last_entry = entry;
            return;
        }
        if self.last_entry.window_tile == window_tile && self.last_entry.command_line == command_line {
            self.last_entry.duration_in_seconds += self.interval_in_seconds;
            return;
        }
        self.entries.push(mem::replace(&mut self.last_entry, entry));
    }

    fn flush(&mut self) {
        use std::io::Write;

        let entry = Entry {
            timestamp: win32helper::get_local_time(),
            duration_in_seconds: 0,
            window_tile: String::new(),
            command_line: String::new(),
        };
        self.entries.push(mem::replace(&mut self.last_entry, entry));

        for entry in &self.entries {
            let now = entry.timestamp;
            writeln!(self.file,
                     "{}-{}-{} {}:{}:{}, {}, {}, {}",
                     now.wYear,
                     now.wMonth,
                     now.wDay,
                     now.wHour,
                     now.wMinute,
                     now.wSecond,
                     entry.duration_in_seconds,
                     entry.command_line,
                     entry.window_tile)
                .unwrap();
        }

        self.entries.clear();
        self.count = 0;
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        self.flush();
    }
}
