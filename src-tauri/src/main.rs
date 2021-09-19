#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod bindings {
  windows::include_bindings!();
}

use bindings::{
  Windows::Win32::Foundation::{BOOL, HINSTANCE, HANDLE},
  Windows::Win32::System::ProcessStatus::K32EnumProcessModules,
  Windows::Win32::System::Threading::{
    OpenProcess, WaitForSingleObject, PROCESS_QUERY_INFORMATION, PROCESS_SYNCHRONIZE,
    PROCESS_VM_READ,
  },
  Windows::Win32::System::WindowsProgramming::INFINITE,
};



mod error;
mod file_manager;
mod game_scanner;
mod memory_scanner;

use file_manager::FileManager;
use game_scanner::{ScanError, ScanResult, Scanner};
use std::env;
use tauri::Manager;

fn main() {
  let args: Vec<String> = env::args().collect();
  let mut background = false;
  for arg in args {
    if arg == "--background" {
      background = true;
      break;
    }
  }

  if background {
    println!("Started background?");
  } else {
    tauri::Builder::default()
      .setup(|app| {
        let scanner = Scanner::new(app.handle());
        app.manage(scanner);
        Ok(())
      })
      .invoke_handler(tauri::generate_handler![get_gil, is_attached, load_from_disk])
      .run(tauri::generate_context!())
      .expect("error while running tauri application");
  }
}

/**
Get a boolean indicating whether the game is attached or not.

Used one time on initial DOM load to get backend status.

Events used afterwards.
*/
#[tauri::command]
fn is_attached(scanner: tauri::State<Scanner>) -> bool {
  return scanner.attached();
}

#[tauri::command]
fn get_gil(scanner: tauri::State<Scanner>) -> Result<u32, ScanError> {
  scanner.get_gil()
}

#[tauri::command]
fn load_from_disk() -> Vec<ScanResult> {
  let data = FileManager::read_data_from_disk();
  return data;
}