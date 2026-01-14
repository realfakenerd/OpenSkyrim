use bevy::prelude::*;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn hello_world_system() {
    println!("Hello from Bevy!");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Basic Bevy verification
    App::new()
        .add_systems(Update, hello_world_system)
        .run_once();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
