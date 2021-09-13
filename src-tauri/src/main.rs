#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

extern crate benfred_read_process_memory as read_process_memory;

mod bindings {
  windows::include_bindings!();
}

use bindings::{
  Windows::Win32::System::Diagnostics::Debug::{GetLastError},
  Windows::Win32::System::ProcessStatus::{K32EnumProcessModules},
  Windows::Win32::System::Threading::{OpenProcess, PROCESS_ALL_ACCESS, PROCESS_VM_READ, PROCESS_QUERY_INFORMATION},
  Windows::Win32::Foundation::{BOOL, HINSTANCE, HWND, HANDLE},
  Windows::Win32::UI::WindowsAndMessaging::{FindWindowA},
};

use read_process_memory::{copy_address, CopyAddress, Pid, ProcessHandle};
use std::{
  convert::TryInto,
  io, thread,
  time::{Duration, Instant},
  mem::{MaybeUninit}
};
use sysinfo::{ProcessExt, System, SystemExt};

fn main() {
  tauri::Builder::default()
    .setup(|_| {
      let _scheduler = thread::spawn(|| {
        let mut sys = System::new_all();
        let run = true;
        let mut found = false;
        let mut base_offset: usize = 0;
        let mut process_id: usize = 1 as usize;
        let process_scan_interval = Duration::from_secs(3);
        let _currency_scan_interval = Duration::from_secs(10);
        // Main background scanning loop
        while run == true {
          sys.refresh_processes();
          // Search for game process
          if !found {
            let start = Instant::now();
            for (pid, process) in sys.processes() {
              if process.name() == "ffxiv_dx11.exe" {
                found = true;
                process_id = *pid;
                base_offset = process.memory() as usize;
                println!(
                  "Found game process... ID: {}, Offset: {}",
                  process_id, base_offset
                );
                break;
              }
            }
            let runtime = start.elapsed();
            // If there is time left in the scheduler, sleep thread
            if let Some(remaining) = process_scan_interval.checked_sub(runtime) {
              thread::sleep(remaining);
            }
          }
          // Scan for values
          else {
            let handle = get_handle(process_id as u32).expect("Not sure how this happened.");
            let module : u64 = get_module(handle).expect("Module not found.");
            println!("Module Address: {}", module);

            let start = Instant::now();

            // Base
            let bytes = read_memory(process_id as Pid, module as usize + 0x01DD4358, 8).unwrap();
            let str: String = hex::encode(bytes);
            println!("Static offset address: 0x{}", str);
            let address: usize = usize::from_str_radix(&str, 16).unwrap().try_into().unwrap();

            // First offset
            let bytes: Vec<u8> = read_memory(process_id as Pid, address + 0x78, 8).unwrap();
            let str: String = hex::encode(bytes);
            let address: usize = usize::from_str_radix(&str, 16).unwrap().try_into().unwrap();

            // Final offset
            let bytes: Vec<u8> = read_memory(process_id as Pid, address + 0xC, 4).unwrap();
            let gil: u32 = u32::from_be_bytes(bytes.try_into().expect("Should always have a value"));
            println!("Gil: {}\n\n", gil);

            let runtime = start.elapsed();
            if let Some(remaining) = process_scan_interval.checked_sub(runtime) {
              thread::sleep(remaining);
            }
          }
        }
      });
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
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
  let mut handle: HANDLE;
  unsafe {
    handle = OpenProcess(PROCESS_VM_READ | PROCESS_QUERY_INFORMATION, BOOL::from(false), pid);
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
  Ok((module_id))
}
