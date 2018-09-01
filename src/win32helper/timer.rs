extern crate winapi;

use self::winapi::{
        shared::minwindef,
        um::winnt,
        um::winbase};

use std::mem;
use std::thread;

use super::*;

pub trait TimerTrait {
    fn timer_func(&mut self);
}

#[derive(Debug)]
pub struct PeriodicTimer<T>
    where T: TimerTrait {

    period_in_second: u32,
    timer: T,
    timer_handle: winnt::HANDLE,
    start_event: winnt::HANDLE,
    running: bool,
}

impl<T> PeriodicTimer<T>
    where T: TimerTrait {

    pub fn new(period_in_second: u32, t: T) -> Self {

        let manual_reset = true;
        let auto_reset = false;
        let initial_state_signalled = true;
        // let initial_state_not_signalled = false;

        let timer = PeriodicTimer {
            period_in_second: period_in_second,
            timer: t,
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
            let this_ptr: *mut PeriodicTimer<T> = mem::transmute(raw_ptr);
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

    fn i64_to_large_integer(i: i64) -> winnt::LARGE_INTEGER {
        unsafe {
            let mut large_integer: winnt::LARGE_INTEGER = mem::zeroed();
            *large_integer.QuadPart_mut() = i;
            large_integer
        }
    }

    fn start_for_real(&mut self) {
        let due_time: winnt::LARGE_INTEGER = PeriodicTimer::<T>::i64_to_large_integer(-1); // trigger immediately
        let resume_system = false;
        let raw_ptr: *mut PeriodicTimer<T> = self;
        if set_waitable_timer(self.timer_handle,
                              &due_time,
                              PeriodicTimer::<T>::seconds_to_millisecond(self.period_in_second) as winnt::LONG,
                              Some(PeriodicTimer::<T>::apc_routine),
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
        let this_ptr: *mut PeriodicTimer<T> = context as *mut PeriodicTimer<T>;
        (*this_ptr).timer.timer_func();
    }
}
