mod file_access;
mod resource_usage;

use tauri::{Manager, PhysicalPosition, PhysicalSize, WebviewWindow};

/// Each side of the main window as a fraction of the monitor's work area.
/// 0.7 × 0.7 is roughly half of the usable screen area.
const WINDOW_SCALE: f64 = 0.7;
/// Never open smaller than the original fixed size (logical pixels).
const MIN_WINDOW_SIZE: (f64, f64) = (800.0, 600.0);

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Sizes the window relative to the monitor's work area (which excludes the
/// macOS Dock/menu bar and the Windows taskbar) and centers it within it.
fn size_and_center(window: &WebviewWindow) -> tauri::Result<()> {
    let monitor = match window.current_monitor()? {
        Some(monitor) => monitor,
        None => match window.primary_monitor()? {
            Some(monitor) => monitor,
            None => return Ok(()),
        },
    };

    let area = monitor.work_area();
    let scale = monitor.scale_factor();
    let fit = |available: u32, min_logical: f64| -> u32 {
        let target = (available as f64 * WINDOW_SCALE).max(min_logical * scale);
        target.min(available as f64).round() as u32
    };

    let width = fit(area.size.width, MIN_WINDOW_SIZE.0);
    let height = fit(area.size.height, MIN_WINDOW_SIZE.1);
    let x = area.position.x + ((area.size.width - width) / 2) as i32;
    let y = area.position.y + ((area.size.height - height) / 2) as i32;

    window.set_size(PhysicalSize::new(width, height))?;
    window.set_position(PhysicalPosition::new(x, y))?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(resource_usage::ResourceMonitor::new())
        .setup(|app| {
            // The window starts hidden (see tauri.conf.json) so it doesn't
            // visibly jump from its default size to the computed one.
            if let Some(window) = app.get_webview_window("main") {
                if let Err(error) = size_and_center(&window) {
                    eprintln!("couldn't size the main window: {error}");
                }
                window.show()?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            file_access::commands::open_markdown_file,
            file_access::commands::open_markdown_folder,
            file_access::commands::read_markdown_file,
            file_access::commands::read_recent_files,
            file_access::commands::write_recent_files,
            resource_usage::read_resource_usage,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
