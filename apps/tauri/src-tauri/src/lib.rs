mod menu;

#[tauri::command]
fn platform_ping() -> &'static str {
    "treemaker-tauri"
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            menu::setup_menu(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![platform_ping])
        .run(tauri::generate_context!())
        .expect("error while running TreeMaker");
}
