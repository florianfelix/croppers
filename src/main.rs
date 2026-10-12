#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let mut app = croppers::Croppers::new();
    let _ = app.run();
}
