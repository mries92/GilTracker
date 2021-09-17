// ----- Imports -----
use std::{
  convert::TryInto,
  io,
  mem::MaybeUninit,
  sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex,
  },
  thread,
  time::{Duration, Instant},
};

mod bindings {
  windows::include_bindings!();
}

use bindings::{
  Windows::Win32::Foundation::{BOOL, HANDLE, HINSTANCE},
  Windows::Win32::System::ProcessStatus::K32EnumProcessModules,
  Windows::Win32::System::Threading::{
    OpenProcess, WaitForSingleObject, PROCESS_QUERY_INFORMATION, PROCESS_SYNCHRONIZE,
    PROCESS_VM_READ,
  },
  Windows::Win32::System::WindowsProgramming::INFINITE,
};

use benfred_read_process_memory::{copy_address, Pid, ProcessHandle};
use serde::Deserialize;
use sysinfo::{ProcessExt, System, SystemExt};
use tauri::Manager;
use thiserror::Error;
// ----- End Imports -----


#[derive(Error, Debug)]
pub enum ScanError {
  #[error("Could not convert process id to valid handle")]
  HandleConversionError,
  #[error("Not attached to the game process")]
  NotAttached,
  #[error("Failed to read memory at requested location")]
  MemoryReadError,
}

impl ScanError {
  const fn code(&self) -> &'static str {
    match self {
      ScanError::HandleConversionError => "HandleConversionError",
      ScanError::NotAttached => "NotAttached",
      ScanError::MemoryReadError => "MemoryReadError",
    }
  }
}

impl serde::Serialize for ScanError {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    use serde::ser::SerializeStruct;
    let mut state = serializer.serialize_struct("Error", 2)?;
    state.serialize_field("code", &self.code())?;
    state.serialize_field("description", &self.to_string())?;
    state.end()
  }
}

/// Holds the results of a scan
#[derive(Deserialize, Debug)]
pub struct ScanResult {
  value: u32,
  timestamp: u64,
}

/// Game scanning struct. Implements methods for reading values from game memory.
pub struct Scanner {
  gil_offsets: [usize; 3],
  attached: Arc<AtomicBool>,
  process_id: Arc<AtomicUsize>,      // Base process id
  base_address: Arc<AtomicUsize>,    // Base address
  app: Arc<Mutex<tauri::AppHandle>>, // Reference to the base application
}

impl Scanner {
  pub fn new(app: tauri::AppHandle) -> Scanner {
    let scanner = Scanner {
      app: Arc::new(Mutex::new(app)),
      attached: Arc::new(AtomicBool::new(false)),
      process_id: Arc::new(AtomicUsize::new(1)),
      base_address: Arc::new(AtomicUsize::new(1)),
      gil_offsets: [0x01DD4358, 0x78, 0xC],
    };
    scanner.start_scan();
    return scanner;
  }

  /**
  Starts the asynchronous scanning thread.

  ### Note
  This thread loops infinitely. Once process is found, thread is halted
  until process is closed, then scanning resumes.
  */
  fn start_scan(&self) {
    self.attached.store(false, Ordering::Relaxed);
    let base_address = self.base_address.clone();
    let process_id = self.process_id.clone();
    let attached = self.attached.clone();
    let app = self.app.clone();
    std::thread::spawn(move || {
      let mut sys = System::new_all();
      let process_scan_interval = Duration::from_secs(2);
      let mut handle: HANDLE = HANDLE(0);
      loop {
        // Enumerate processes and look for handle
        let start = Instant::now();
        sys.refresh_processes();
        for (pid, process) in sys.processes() {
          if process.name() == "ffxiv_dx11.exe" {
            handle = Scanner::get_handle(*pid as u32).expect("Not sure how this happened.");
            let ba = Scanner::get_module(handle).expect("Module not found.") as usize;
            base_address.store(ba, Ordering::Relaxed);
            process_id.store(*pid, Ordering::Relaxed);
            attached.store(true, Ordering::Relaxed);
            break;
          }
        }
        if !handle.is_null() {
          unsafe {
            WaitForSingleObject(handle, INFINITE);
            app
              .lock()
              .expect("App has to exist.")
              .emit_all("test", {})
              .unwrap();
            attached.store(false, Ordering::Relaxed);
            handle = HANDLE(0); // Drop the existing handle
          }
        } else {
          let runtime = start.elapsed();
          if let Some(remaining) = process_scan_interval.checked_sub(runtime) {
            thread::sleep(remaining);
          }
        }
      }
    });
  }

  // Get the players current Gil
  pub fn get_gil(&self) -> Result<u32, ScanError> {
    if !self.attached.load(Ordering::Relaxed) {
      return Err(ScanError::NotAttached);
    }
    // Error handling must be done for each read. The game process could have closed in between calls.
    // Static pointer
    let base = self.base_address.as_ref();
    let bytes = self.read_memory(base.load(Ordering::Relaxed) + self.gil_offsets[0], 8);
    let bytes = match bytes {
      Ok(bytes) => bytes,
      Err(err) => {
        // Start scanning for the game process again
        self.start_scan();
        return Err(err);
      }
    };
    // Parse the bytes into a hex string
    let str: String = hex::encode(bytes);
    // If at this point, this should succeed. Still, // TODO add handling with custom error type
    let address = usize::from_str_radix(&str, 16).unwrap();

    // First offset
    let bytes = self.read_memory(address + self.gil_offsets[1], 8);
    let bytes = match bytes {
      Ok(bytes) => bytes,
      Err(err) => {
        // Start scanning for the game process again
        self.start_scan();
        return Err(err);
      }
    };
    let str: String = hex::encode(bytes);
    let address: usize = usize::from_str_radix(&str, 16).unwrap().try_into().unwrap();

    // Final offset
    let bytes = self.read_memory(address + self.gil_offsets[2], 4);
    let bytes = match bytes {
      Ok(bytes) => bytes,
      Err(err) => {
        // Start scanning for the game process again
        self.start_scan();
        return Err(err);
      }
    };
    let gil = u32::from_be_bytes(bytes.try_into().expect("Should always have a value"));
    Ok(gil)
  }

  /// Read an array of bytes from game memory
  fn read_memory(&self, address: usize, size: usize) -> Result<Vec<u8>, ScanError> {
    let id = self.process_id.load(Ordering::Relaxed) as Pid;
    let handle: Result<ProcessHandle, _> = id.try_into();
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
  fn get_handle(pid: u32) -> io::Result<HANDLE> {
    let handle: HANDLE;
    unsafe {
      handle = OpenProcess(
        PROCESS_VM_READ | PROCESS_QUERY_INFORMATION | PROCESS_SYNCHRONIZE,
        BOOL::from(false),
        pid,
      );
    }
    Ok(handle)
  }

  // Get the base module of a process by process ID
  fn get_module(handle: HANDLE) -> io::Result<u64> {
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
}
