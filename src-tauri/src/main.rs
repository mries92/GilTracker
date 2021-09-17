#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod error;
mod file_manager;
mod game_scanner;

use file_manager::FileManager;
use game_scanner::{ScanError, Scanner};
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
      .invoke_handler(tauri::generate_handler![get_gil, read_data_from_file])
      .run(tauri::generate_context!())
      .expect("error while running tauri application");
  }
}

#[tauri::command]
fn get_gil(scanner: tauri::State<Scanner>) -> Result<u32, ScanError> {
  scanner.get_gil()
}

/// Read the existing data we have already captured
#[tauri::command]
fn read_data_from_file() {
  FileManager::read_data_from_disk();
}
