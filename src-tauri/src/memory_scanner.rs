use std::{convert::TryInto, io, mem::MaybeUninit};
use benfred_read_process_memory::{copy_address, Pid, ProcessHandle};

use crate::{PROCESS_VM_READ, PROCESS_QUERY_INFORMATION, PROCESS_SYNCHRONIZE, BOOL, HINSTANCE, HANDLE, K32EnumProcessModules, OpenProcess};
use crate::game_scanner::ScanError;

/// Read an array of bytes from game memory
pub fn read_memory(process_id: u32, address: usize, size: usize) -> Result<Vec<u8>, ScanError> {
  let handle: Result<ProcessHandle, _> = (process_id as Pid).try_into();
  let handle = match handle {
    Ok(handle) => handle,
    Err(_) => return Err(ScanError::HandleConversionError),
  };
  let bytes = copy_address(address, size, &handle);
  let mut bytes = match bytes {
    Ok(bytes) => bytes,
    Err(_) => return Err(ScanError::MemoryReadError),
  };
  bytes.reverse(); // Little endian to big
  Ok(bytes)
}

// Get a win32 handle to a process by process ID
pub fn get_handle(pid: usize) -> io::Result<HANDLE> {
  let mut handle: HANDLE = HANDLE(0);
  unsafe {
    handle = OpenProcess(
      PROCESS_VM_READ | PROCESS_QUERY_INFORMATION | PROCESS_SYNCHRONIZE,
      BOOL::from(false),
      pid as u32,
    );
  }
  Ok(handle)
}

// Get the base module of a process by process ID
pub fn get_module(handle: HANDLE) -> io::Result<u64> {
  let mut hmods;
  unsafe {
    hmods = MaybeUninit::<[MaybeUninit<HINSTANCE>; 1024]>::uninit().assume_init();
  }
  let ptr = hmods.as_mut_ptr() as *mut HINSTANCE;
  let mut cbneeded: u32 = 0;
  let mut module_id: u64 = 0;
  unsafe {
    if K32EnumProcessModules(handle, ptr, 1024, &mut cbneeded) == BOOL::from(true) {
      // Base module is always first
      module_id = hmods[0].assume_init().0 as u64;
    }
  }
  Ok(module_id)
}
