// 模块声明
mod http_client;
mod file_ops;
mod config;

// 导入命令
use http_client::fetch_text_from_url;
use file_ops::{read_json_file, append_to_json_file, get_default_file_path, select_file};
use config::{load_config, save_config};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            fetch_text_from_url,
            read_json_file,
            append_to_json_file,
            get_default_file_path,
            select_file,
            load_config,
            save_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
