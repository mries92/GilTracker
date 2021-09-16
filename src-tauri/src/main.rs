#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod game_scanner;

use std::env;
use game_scanner::{ScanError, Scanner};

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
      .invoke_handler(tauri::generate_handler![get_gil])
      .run(tauri::generate_context!())
      .expect("error while running tauri application");
  }
}

#[tauri::command]
fn get_gil(state: tauri::State<Scanner>) -> Result<u32, ScanError> {
  state.get_gil()
}