mod module;
mod process;
mod snapshot;

pub use crate::mem::module::*;
pub use crate::mem::process::*;
pub use crate::mem::snapshot::*;

pub trait Constructor {
    fn new() -> Self;
}

use windows::Win32::Foundation::{HANDLE, NTSTATUS};

#[link(name = "ntdll")]
extern "system" {
    pub fn ZwReadVirtualMemory(
        hprocess: HANDLE,
        lpbaseaddress: *const ::core::ffi::c_void,
        lpbuffer: *mut ::core::ffi::c_void,
        nsize: usize,
        lpnumberofbytesread: *mut usize,
    ) -> NTSTATUS;
    pub fn ZwWriteVirtualMemory(
        ProcessHandle: HANDLE,
        BaseAddress: *mut ::core::ffi::c_void,
        Buffer: *const ::core::ffi::c_void,
        BufferSize: usize,
        NumberOfBytesWritten: *mut usize,
    ) -> NTSTATUS;
}
