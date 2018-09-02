#![allow(non_snake_case)]

extern crate winapi;

use self::winapi::{shared::basetsd, shared::minwindef, shared::ntdef, um::winnt};

use std::mem;

pub fn NT_SUCCESS(status: ntdef::NTSTATUS) -> bool {
    status >= 0
}

#[derive(Debug)]
#[repr(C)]
pub enum PROCESSINFOCLASS {
    ProcessBasicInformation = 0,
    ProcessWow64Information = 26,
}

// typedef NTSTATUS(NTAPI* type_NtQueryInformationProcess)(
//         _In_ HANDLE ProcessHandle,
//         _In_ PROCESSINFOCLASS ProcessInformationClass,
//         _Out_ PVOID ProcessInformation,
//         _In_ ULONG ProcessInformationLength,
//         _Out_opt_ PULONG ReturnLength);

#[link(name = "ntdll")]
extern "system" {
    pub fn NtQueryInformationProcess(
        ProcessHandle: winnt::HANDLE,
        ProcessInformationClass: PROCESSINFOCLASS,
        ProcessInformation: winnt::PVOID,
        ProcessInformationLength: minwindef::ULONG,
        ReturnLength: &mut minwindef::ULONG,
    ) -> ntdef::NTSTATUS;
}

// pub unsafe extern "system" fn NtQueryInformationProcess(ProcessHandle: HANDLE, ProcessInformationClass: PROCESSINFOCLASS,
//      ProcessInformation: PVOID, ProcessInformationLength: ULONG, ReturnLength: &mut ULONG) -> NTSTATUS;
pub fn nt_query_information_process<T>(
    process_handle: winnt::HANDLE,
    information_class: PROCESSINFOCLASS,
    buffer: *mut T,
) -> bool {
    let mut return_length: minwindef::ULONG = 0;
    unsafe {
        let status = NtQueryInformationProcess(
            process_handle,
            information_class,
            buffer as minwindef::LPVOID,
            mem::size_of::<T>() as minwindef::ULONG,
            &mut return_length,
        );
        NT_SUCCESS(status)
    }
}

#[repr(C)]
pub struct PROCESS_BASIC_INFORMATION {
    Reserved1: winnt::PVOID,
    pub PebBaseAddress: winnt::PVOID, // PPEB,
    Reserved2: [winnt::PVOID; 2],
    pub UniqueProcessId: basetsd::ULONG_PTR,
    Reserved3: winnt::PVOID,
}

#[repr(C)]
pub struct PROCESS_ENVIRONMENT_BLOCK {
    Reserved1: [u8; 2],
    pub BeingDebugged: u8,
    Reserved2: u8,
    Reserved3: [winnt::PVOID; 2],
    pub Ldr: winnt::PVOID,
    pub ProcessParameters: winnt::PVOID,
}

type POINTER32 = u32;

#[repr(C)]
pub struct PROCESS_ENVIRONMENT_BLOCK_32 {
    Reserved1: [u8; 2],
    pub BeingDebugged: u8,
    Reserved2: u8,
    Reserved3: [POINTER32; 2],
    pub Ldr: POINTER32,
    pub ProcessParameters: POINTER32,
}

// #[repr(C)]
// pub struct UNICODE_STRING {
//     Length: minwindef::USHORT,
//     MaximumLength: minwindef::USHORT,
//     Buffer: winnt::PVOID,
// }

#[repr(C)]
pub struct UNICODE_STRING_32 {
    pub Length: minwindef::USHORT,
    pub MaximumLength: minwindef::USHORT,
    pub Buffer: POINTER32,
}

#[repr(C)]
pub struct RTL_USER_PROCESS_PARAMETERS {
    Reserved1: [u8; 16],
    Reserved2: [winnt::PVOID; 10],
    pub ImagePathName: ntdef::UNICODE_STRING,
    pub CommandLine: ntdef::UNICODE_STRING,
}

#[repr(C)]
pub struct RTL_USER_PROCESS_PARAMETERS_32 {
    Reserved1: [u8; 16],
    Reserved2: [POINTER32; 10],
    pub ImagePathName: UNICODE_STRING_32,
    pub CommandLine: UNICODE_STRING_32,
}
