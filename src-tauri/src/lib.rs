mod file_access;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            file_access::commands::open_markdown_file,
            file_access::commands::open_markdown_folder,
            file_access::commands::read_markdown_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
