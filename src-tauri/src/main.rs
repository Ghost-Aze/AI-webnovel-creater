#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(feature = "tauri-app")]
fn main() {
    webnovel_ai_studio_lib::run();
}

#[cfg(not(feature = "tauri-app"))]
fn main() {
    eprintln!("Tauri runtime disabled; use --features tauri-app to launch the application.");
}
