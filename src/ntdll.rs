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
