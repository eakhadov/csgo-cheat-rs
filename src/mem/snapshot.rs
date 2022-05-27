use std::ops::Deref;

use windows::{Win32::Foundation::*, Win32::System::Diagnostics::ToolHelp::*};

pub struct SnapshotHandle {
    pub handle: HANDLE,
}

impl SnapshotHandle {
    pub fn new(pid: u32, flags: CREATE_TOOLHELP_SNAPSHOT_FLAGS) -> Option<Self> {
        match unsafe { CreateToolhelp32Snapshot(flags, pid).ok() } {
            Some(handle) => Some(SnapshotHandle { handle }),
            _ => None,
        }
    }
}

impl Drop for SnapshotHandle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.handle);
        }
    }
}

impl Deref for SnapshotHandle {
    type Target = HANDLE;
    fn deref(&self) -> &HANDLE {
        &self.handle
    }
}
