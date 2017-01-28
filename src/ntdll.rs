#![allow(non_snake_case)]

extern crate winapi;
use self::winapi::*;

pub fn NT_SUCCESS(status: ntdef::NTSTATUS) -> bool {
    status >= 0
}

#[derive(Debug)]
#[repr(C)]
pub enum PROCESSINFOCLASS {
    ProcessBasicInformation = 0,
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
        ReturnLength: &mut minwindef::ULONG)
        -> ntdef::NTSTATUS;
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
type PEB = PROCESS_ENVIRONMENT_BLOCK;

// #[repr(C)]
// pub struct UNICODE_STRING {
//     Length: minwindef::USHORT,
//     MaximumLength: minwindef::USHORT,
//     Buffer: winnt::PVOID,
// }

#[repr(C)]
pub struct RTL_USER_PROCESS_PARAMETERS
{
    Reserved1: [u8; 16],
    Reserved2: [winnt::PVOID; 10],
    pub ImagePathName: UNICODE_STRING,
    pub CommandLine: UNICODE_STRING,
}
