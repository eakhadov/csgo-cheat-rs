use std::{cell::RefCell, collections::HashMap, ffi, mem, ptr, rc::Rc};

use windows::{
    Win32::Foundation::*,
    Win32::System::{
        Diagnostics::{Debug::*, ToolHelp::*},
        Threading::*,
    },
};

use crate::mem::{Constructor, SnapshotHandle, ZwReadVirtualMemory, ZwWriteVirtualMemory};

impl Constructor for PROCESSENTRY32 {
    // Create a new instance of `PROCESSENTRY32`
    fn new() -> Self {
        let mut pe: PROCESSENTRY32 = unsafe { mem::zeroed() };
        pe.dwSize = mem::size_of::<PROCESSENTRY32W>() as u32;
        pe
    }
}

pub struct Process {
    // Process id.
    pub id: u32,

    // Process bitness.
    pub is_wow64: bool,

    // Process `HANDLE`.
    handle: HANDLE,

    // List of modules.
    modules: RefCell<HashMap<String, Rc<super::module::Module>>>,
}

impl Process {
    #[allow(dead_code)]
    pub fn read<T>(&self, address: usize) -> Option<T> {
        let mut buffer = unsafe { mem::zeroed::<T>() };

        debug!("(ZwReadVirtualMemory) - address: 0x{:X}", address);
        match unsafe {
            ZwReadVirtualMemory(
                self.handle,
                address as *const ffi::c_void,
                &mut buffer as *mut T as *mut ffi::c_void,
                mem::size_of::<T>(),
                ptr::null_mut(),
            )
            .is_ok()
        } {
            false => None,
            _ => Some(buffer),
        }
    }

    #[allow(dead_code)]
    pub fn read_ptr<T>(&self, address: usize, buffer: *mut T, count: usize) -> bool {
        debug!("(ZwReadVirtualMemory (ptr)) - address: 0x{:X}", address);
        unsafe {
            ZwReadVirtualMemory(
                self.handle,
                address as *const ffi::c_void,
                buffer as *mut T as *mut ffi::c_void,
                mem::size_of::<T>() as usize * count,
                ptr::null_mut(),
            )
            .is_ok()
        }
    }

    #[allow(dead_code)]
    pub fn write<T>(&self, address: usize, buffer: &T) -> bool {
        debug!("(ZwWriteVirtualMemory) - address: 0x{:X}", address);
        unsafe {
            ZwWriteVirtualMemory(
                self.handle,
                address as *mut ffi::c_void,
                buffer as *const T as *const ffi::c_void,
                mem::size_of::<T>(),
                ptr::null_mut(),
            )
            .is_ok()
        }
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        if !self.handle.is_invalid() {
            debug!("(Drop Process Handle) - handle: {:?}", self.handle);
            unsafe { CloseHandle(self.handle) };
        }
    }
}

impl Process {
    pub fn get_module(&self, name: &str) -> Option<Rc<super::module::Module>> {
        let mut b = self.modules.borrow_mut();
        if b.contains_key(name) {
            return b.get(name).cloned();
        }

        super::module::get(name, self).and_then(|m| b.insert(name.to_string(), Rc::new(m)));
        b.get(name).cloned()
    }
}

pub fn from_pid(pid: u32) -> Option<Process> {
    let handle = match unsafe { OpenProcess(PROCESS_ALL_ACCESS, false, pid) } {
        Ok(h) => h,
        Err(_) => return None,
    };

    let mut tmp: BOOL = BOOL(0);
    if unsafe { IsWow64Process(handle, &mut tmp).as_bool() } == false {
        warn!("Could not determine process bitness: IsWow64Process returned an error!");
        return None;
    }

    let is_wow64 = tmp.as_bool();
    debug!("(OpenProcess) - PID: {}, is_wow64: {}", pid, is_wow64);

    Some(Process {
        id: pid,
        is_wow64,
        handle,
        modules: RefCell::new(HashMap::new()),
    })
}

pub fn from_name(name: &str) -> Option<Process> {
    let snapshot = SnapshotHandle::new(0, TH32CS_SNAPPROCESS)?;
    let mut pe = PROCESSENTRY32::new();

    if !unsafe { Process32First(*snapshot, &mut pe).as_bool() } {
        return None;
    }

    let (_, s, _) = unsafe { name.as_bytes().align_to::<CHAR>() };

    while unsafe { Process32Next(*snapshot, &mut pe).as_bool() } {
        if pe.szExeFile.starts_with(s) {
            return from_pid(pe.th32ProcessID);
        }

        pe.szExeFile
            .iter_mut()
            .take_while(|c| **c != CHAR(0))
            .for_each(|c| *c = CHAR(0));
    }

    None
}
