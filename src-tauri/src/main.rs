#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

extern crate benfred_read_process_memory as read_process_memory;

use read_process_memory::{copy_address, CopyAddress, Pid, ProcessHandle};
use std::{
  convert::TryInto,
  io, thread,
  time::{Duration, Instant},
};
use sysinfo::{ProcessExt, System, SystemExt};

fn main() {
  tauri::Builder::default()
    .setup(|_| {
      let _scheduler = thread::spawn(|| {
        let mut sys = System::new_all();
        let run = true;
        let mut found = false;
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
                println!("Found game process: {}", process_id);
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
            let start = Instant::now();
            let bytes = read_memory(process_id as Pid, 0x200F3161800 + 0x78, 8).unwrap();
            let str : String = hex::encode(bytes);
            println!("Wallet address: 0x{}", str);
            let address : usize = usize::from_str_radix(&str, 16).unwrap().try_into().unwrap();
            let bytes : Vec<u8> = read_memory(process_id as Pid, address + 0xC, 4).unwrap();
            let str : String = hex::encode(bytes);
            let gil : i32 = i32::from_str_radix(&str, 16).unwrap().try_into().unwrap();
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

fn read_memory(pid: Pid, address: usize, size: usize) -> io::Result<Vec<u8>> {
  let handle: ProcessHandle = pid.try_into()?;
  println!("Reading value at: 0x{}", address);
  let mut _bytes = copy_address(address, size, &handle)?;
  _bytes.reverse(); // Flip the bytes so they are easier to work with (little endian to big)
  println!("Read {} bytes", size);
  Ok(_bytes)
}