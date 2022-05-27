use windows::{Win32::Foundation::*, Win32::System::Diagnostics::ToolHelp::*};

use crate::mem::{Constructor, Process, SnapshotHandle};

pub struct Module {
    pub name: String,
    pub base: usize,
    pub size: usize,
    pub data: Vec<u8>,
}

impl Constructor for MODULEENTRY32 {
    // Create a new instance of `MODULEENTRY32W`
    fn new() -> Self {
        let mut module: MODULEENTRY32 = unsafe { std::mem::zeroed() };
        module.dwSize = std::mem::size_of::<MODULEENTRY32>() as u32;
        module
    }
}

impl Module {}

pub fn get(name: &str, process: &Process) -> Option<Module> {
    let snapshot = SnapshotHandle::new(process.id, TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32)?;
    let mut me = MODULEENTRY32::new();

    if !unsafe { Module32First(*snapshot, &mut me).as_bool() } {
        return None;
    }

    let (_, s, _) = unsafe { name.as_bytes().align_to::<CHAR>() };

    while unsafe { Module32Next(*snapshot, &mut me).as_bool() } {
        if me.szModule.starts_with(s) {
            let mut i = Module {
                name: name.to_string(),
                base: me.modBaseAddr as usize,
                size: me.modBaseSize as usize,
                data: vec![0u8; me.modBaseSize as usize],
            };

            if process.read_ptr(i.base, i.data.as_mut_ptr(), i.size) {
                return Some(i);
            } else {
                return None;
            }
        }

        me.szModule
            .iter_mut()
            .take_while(|c| **c != CHAR(0))
            .for_each(|c| *c = CHAR(0));
    }

    None
}
