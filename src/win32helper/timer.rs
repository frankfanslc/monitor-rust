extern crate winapi;
extern crate user32;
extern crate kernel32;

use self::winapi::*;
use std::mem;
use std::thread;

use super::*;

pub type TimerContext = minwindef::LPCVOID;
pub type TimerRoutine = fn(context: TimerContext);

#[derive(Debug)]
pub struct PeriodicTimer {
    period_in_second: u32,
    routine: TimerRoutine,
    context: TimerContext,
    timer_handle: winnt::HANDLE,
    start_event: winnt::HANDLE,
    running: bool,
}

impl PeriodicTimer {
    pub fn new(period_in_second: u32, routine: TimerRoutine, context: TimerContext) -> PeriodicTimer {

        let manual_reset = true;
        let auto_reset = false;
        let initial_state_signalled = true;
        // let initial_state_not_signalled = false;

        let timer = PeriodicTimer {
            period_in_second: period_in_second,
            routine: routine,
            context: context,
            timer_handle: create_waitable_timer(manual_reset),
            start_event: create_event(auto_reset, initial_state_signalled),
            running: false,
        };
        timer
    }

    pub fn start_wait(&self) {
        // Cannot pass PeriodicTimer structure to thread::spawn directly, as it will fail with:
        // error[E0277] the trait `std::marker::Sync` is not implemented for `*const std::os::raw::c_void`
        let raw_ptr: usize = unsafe { mem::transmute(self) };
        thread::spawn(move || unsafe {
            let this_ptr: *mut PeriodicTimer = mem::transmute(raw_ptr);
            loop {
                if wait_for_single_object_ex((*this_ptr).start_event, winbase::INFINITE) {
                    (*this_ptr).start_for_real();
                }
            }
        });
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    fn seconds_to_millisecond(n: u32) -> u32 {
        n * 1000
    }

    pub fn start(&self) {
        set_event(self.start_event);
    }

    fn start_for_real(&mut self) {
        let due_time: winnt::LARGE_INTEGER = -1; // trigger immediately
        let resume_system = false;
        let raw_ptr: *mut PeriodicTimer = self;
        if set_waitable_timer(self.timer_handle,
                              &due_time,
                              PeriodicTimer::seconds_to_millisecond(self.period_in_second) as winnt::LONG,
                              Some(PeriodicTimer::apc_routine),
                              raw_ptr as minwindef::LPVOID,
                              resume_system) {
            self.running = true;

            output_timestamp();
            println!("Timer started");
            println!();
        }
    }

    pub fn stop(&mut self) {
        if !cancel_waitable_timer(self.timer_handle) {
            return;
        }
        self.running = false;

        output_timestamp();
        println!("Timer stopped");
        println!();
    }

    // type PTIMERAPCROUTINE = Option<unsafe extern "system" fn(lpArgToCompletionRoutine: LPVOID, dwTimerLowValue: DWORD, dwTimerHighValue: DWORD)>;
    unsafe extern "system" fn apc_routine(context: minwindef::LPVOID, _: minwindef::DWORD, _: minwindef::DWORD) {
        let this_ptr: *mut PeriodicTimer = context as *mut PeriodicTimer;
        ((*this_ptr).routine)((*this_ptr).context);
    }
}
