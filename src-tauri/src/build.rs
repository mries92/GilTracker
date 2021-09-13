fn main() {
  tauri_build::build();
  windows::build! {
    Windows::Win32::System::Diagnostics::Debug::{GetLastError},
    Windows::Win32::System::ProcessStatus::{K32EnumProcessModules},
    Windows::Win32::System::Threading::{OpenProcess},
    Windows::Win32::Foundation::{BOOL, HINSTANCE, HWND, HANDLE},
    Windows::Win32::UI::WindowsAndMessaging::{FindWindowA},
  };
}
