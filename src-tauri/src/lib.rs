// 模块声明
mod http_client;
mod file_ops;
mod config;
mod augment_oauth;
mod token_manager;

// 导入命令
use http_client::fetch_text_from_url;
use file_ops::{read_json_file, append_to_json_file, get_default_file_path, select_file};
use config::{load_config, save_config};
use augment_oauth::extract_token_from_session;
use token_manager::{read_tokens, write_tokens, add_token, import_from_remote, delete_token, update_token};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenFromSessionResponse {
    pub access_token: String,
    pub tenant_url: String,
    pub email: Option<String>,
    pub credits_balance: Option<i32>,
    pub expiry_date: Option<String>,
}

/// 从 session 提取 token 的 Tauri 命令
#[tauri::command]
async fn parse_session(session: String) -> Result<TokenFromSessionResponse, String> {
    println!("收到 parse_session 命令");

    let token_response = extract_token_from_session(&session).await?;

    Ok(TokenFromSessionResponse {
        access_token: token_response.access_token,
        tenant_url: token_response.tenant_url,
        email: token_response.email,
        credits_balance: token_response.credits_balance,
        expiry_date: token_response.expiry_date,
    })
}

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
            save_config,
            parse_session,
            read_tokens,
            write_tokens,
            add_token,
            import_from_remote,
            delete_token,
            update_token
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
