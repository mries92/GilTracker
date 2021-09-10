#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

fn main() {
  tauri::Builder::default()
    .on_page_load(|_, _| {
    })
    .setup(|_| {
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}