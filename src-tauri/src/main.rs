#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

extern crate benfred_read_process_memory as read_process_memory;

mod bindings {
  windows::include_bindings!();
}

use bindings::{
  Windows::Win32::Foundation::{BOOL, HANDLE, HINSTANCE},
  Windows::Win32::System::ProcessStatus::K32EnumProcessModules,
  Windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
};

use read_process_memory::{copy_address, Pid, ProcessHandle};
use std::{convert::TryInto, io, env, mem::MaybeUninit, ops::{Deref}, sync::{Arc, Mutex}, thread, time::{Duration, Instant}};
use sysinfo::{ProcessExt, System, SystemExt};

fn main() {
  let args: Vec<String> = env::args().collect();
  let mut background = false;
  println!("{:?}", args);
  for arg in args {
    if arg == "--background" {
      background = true;
      break;
    }
  }

  let scanner = Arc::new(Mutex::new(Scanner::new()));
  if background {
    println!("Started background?");
  } else {
    scanner_attach_thread(scanner.clone());
    tauri::Builder::default()
      .setup(|_| Ok(()))
      .manage(scanner)
      .invoke_handler(tauri::generate_handler![get_gil])
      .run(tauri::generate_context!())
      .expect("error while running tauri application");
  }
}

#[tauri::command]
fn get_gil(state: tauri::State<Arc<Mutex<Scanner>>>) -> u32{
  println!("CALLED FROM JS!");
  return state.deref().deref().lock().expect("Scanner like has to be here bro").get_gil();
}

// Start the thread responsible for attempting to attach to the game
// process. Thread dies after game process is found.
fn scanner_attach_thread(scanner: Arc<Mutex<Scanner>>) {
  let scanner_cpy = scanner.clone();
  std::thread::spawn(move || {
    let mut sys = System::new_all();
    let mut run = true;
    let mut found = false;
    let process_scan_interval = Duration::from_secs(3);
    // Main background scanning loop
    while run == true {
      sys.refresh_processes();
      // Search for game process
      if !found {
        let start = Instant::now();
        for (pid, process) in sys.processes() {
          if process.name() == "ffxiv_dx11.exe" {
            found = true;
            let handle = Scanner::get_handle(*pid as u32).expect("Not sure how this happened.");
            let ba = Scanner::get_module(handle).expect("Module not found.") as usize;
            println!("Found game process, exiting scanning thread.");
            scanner_cpy
              .deref()
              .lock()
              .expect("This should always have a value, even if its just default")
              .base_address = ba;
            scanner_cpy
              .deref()
              .lock()
              .expect("This should always have a value, even if its just default")
              .process_id = *pid;
            run = false;
            break;
          }
        }
        let runtime = start.elapsed();
        // If there is time left in the scheduler, sleep thread
        if let Some(remaining) = process_scan_interval.checked_sub(runtime) {
          thread::sleep(remaining);
        }
      }
    }
  });
}


// Game scanning struct. Implements methods for reading values from game memory.
struct Scanner {
  gil_offsets: [usize; 3],
  process_id: usize,   // Base process id
  base_address: usize, // Base address
}

impl Scanner {
  fn new() -> Scanner {
    let scanner = Scanner {
      process_id: 1,
      base_address: 1,
      gil_offsets: [0x01DD4358, 0x78, 0xC]
    };
    return scanner;
  }

  // Get the players current Gil
  fn get_gil(&self) -> u32 {
    // Static pointer
    let bytes =
      Scanner::read_memory(self.process_id as Pid, self.base_address + self.gil_offsets[0], 8).unwrap();
    let str: String = hex::encode(bytes);
    let address: usize = usize::from_str_radix(&str, 16).unwrap().try_into().unwrap();
    // First offset
    let bytes: Vec<u8> = Scanner::read_memory(self.process_id as Pid, address + self.gil_offsets[1], 8).unwrap();
    let str: String = hex::encode(bytes);
    let address: usize = usize::from_str_radix(&str, 16).unwrap().try_into().unwrap();
    // Final offset
    let bytes: Vec<u8> = Scanner::read_memory(self.process_id as Pid, address + self.gil_offsets[2], 4).unwrap();
    let gil: u32 = u32::from_be_bytes(bytes.try_into().expect("Should always have a value"));
    return gil;
  }

  // Read an array of bytes from a memory location
  fn read_memory(pid: Pid, address: usize, size: usize) -> io::Result<Vec<u8>> {
    let handle: ProcessHandle = pid.try_into()?;
    let mut _bytes = copy_address(address, size, &handle)?;
    _bytes.reverse(); // Flip the bytes so they are easier to work with (little endian to big)
    Ok(_bytes)
  }

  // Get a win32 handle to a process by process ID
  fn get_handle(pid: u32) -> io::Result<HANDLE> {
    let handle: HANDLE;
    unsafe {
      handle = OpenProcess(
        PROCESS_VM_READ | PROCESS_QUERY_INFORMATION,
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