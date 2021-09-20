// ----- Imports -----
use std::{
  convert::TryInto,
  sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc, Mutex,
  },
  thread,
  time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};
use sysinfo::{ProcessExt, System, SystemExt};
use tauri::Manager;
use thiserror::Error;

use crate::{file_manager::FileManager, memory_scanner};
use crate::{WaitForSingleObject, HANDLE, INFINITE};
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
#[derive(Deserialize, Serialize, Debug)]
pub struct ScanResult {
  pub gil: u32,
  pub mgp: u32,
  pub company_seals: u32,
  pub timestamp: u64,
}

impl ScanResult {
  fn new() -> ScanResult {
    let instance = ScanResult {
        gil: 0,
        mgp: 0,
        company_seals: 0,
        timestamp: 0,
    };
    return instance;
  }
}

/// Game scanning struct. Implements methods for reading values from game memory.
pub struct Scanner {
  gil_offsets: [usize; 3],
  attached: Arc<AtomicBool>,
  process_id: Arc<AtomicUsize>,      // Base process id
  base_address: Arc<AtomicUsize>,    // Base address
  app: Arc<Mutex<tauri::AppHandle>>, // Reference to the base application
}

#[derive(Clone)]
pub struct ScanEvent {
  code: String,
  description: String,
}

impl serde::Serialize for ScanEvent {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    use serde::ser::SerializeStruct;
    let mut state = serializer.serialize_struct("ScanEvent", 2)?;
    state.serialize_field("code", &self.code)?;
    state.serialize_field("description", &self.description)?;
    state.end()
  }
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

  /// Used to get attached status. Used once from front end when DOM load is complete.
  pub fn attached(&self) -> bool {
    return self.attached.load(Ordering::Relaxed);
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
      let process_scan_interval = Duration::from_secs(5);
      let mut handle: HANDLE = HANDLE(0);
      loop {
        // Enumerate processes and look for handle
        let start = Instant::now();
        sys.refresh_all();
        for (pid, process) in sys.processes() {
          if process.name() == "ffxiv_dx11.exe" {
            handle = memory_scanner::get_handle(*pid).expect("Not sure how this happened.");
            let ba = memory_scanner::get_module(handle).expect("Module not found.") as usize;
            base_address.store(ba, Ordering::Relaxed);
            process_id.store(*pid, Ordering::Relaxed);
            attached.store(true, Ordering::Relaxed);
            app
              .lock()
              .expect("App has to exist.")
              .emit_all(
                "ScanEvent",
                ScanEvent {
                  code: "GameConnected".to_string(),
                  description: "Game client found.".to_string(),
                },
              )
              .unwrap();
            break;
          }
        }
        if !handle.is_null() {
          unsafe {
            WaitForSingleObject(handle, INFINITE);
            app
              .lock()
              .expect("App has to exist.")
              .emit_all(
                "ScanEvent",
                ScanEvent {
                  code: "GameDisconnected".to_string(),
                  description: "Game client lost.".to_string(),
                },
              )
              .unwrap();
            attached.store(false, Ordering::Relaxed);
            handle = HANDLE(0); // Drop the existing handle
            sys.refresh_all(); // Gotta do the refresh here unfortunately to keep it thread safe
          }
        }
        let runtime = start.elapsed();
        if let Some(remaining) = process_scan_interval.checked_sub(runtime) {
          thread::sleep(remaining);
        }
      }
    });
  }

  pub fn get_currency(&self) -> Result<ScanResult, ScanError> {
    let mut result = ScanResult::new();
    result.gil = self.get_gil()?;
    result.timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
    Ok(result)
  }

  // Get the players current Gil
  fn get_gil(&self) -> Result<u32, ScanError> {
    let id = self.process_id.load(Ordering::Relaxed) as u32;
    if !self.attached.load(Ordering::Relaxed) {
      return Err(ScanError::NotAttached);
    }
    // Static pointer
    let base = self.base_address.as_ref();

    // Parse the bytes into a hex string
    let bytes =
      memory_scanner::read_memory(id, base.load(Ordering::Relaxed) + self.gil_offsets[0], 8)?;
    let str: String = hex::encode(bytes);
    let address = usize::from_str_radix(&str, 16).unwrap();

    // First offset
    let bytes = memory_scanner::read_memory(id, address + self.gil_offsets[1], 8)?;
    let str: String = hex::encode(bytes);
    let address: usize = usize::from_str_radix(&str, 16).unwrap().try_into().unwrap();

    // Final offset
    let bytes = memory_scanner::read_memory(id, address + self.gil_offsets[2], 4)?;
    let gil = u32::from_be_bytes(bytes.try_into().expect("Should always have a value"));

    Ok(gil)
  }
}
