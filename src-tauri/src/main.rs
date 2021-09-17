#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod game_scanner;
mod file_manager;
mod error;

use std::env;
use game_scanner::{ScanError, Scanner};
use file_manager::{FileManager};

fn main() {
  let args: Vec<String> = env::args().collect();
  let mut background = false;
  for arg in args {
    if arg == "--background" {
      background = true;
      break;
    }
  }

  let scanner = game_scanner::Scanner::new();
  if background {
    println!("Started background?");
  } else {
    tauri::Builder::default()
      .setup(|_| Ok(()))
      .manage(scanner)
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