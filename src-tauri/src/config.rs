use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::env;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub url: String,
    pub file_path: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            file_path: String::new(),
        }
    }
}

/// 获取配置文件路径
fn get_config_path() -> Result<PathBuf, String> {
    // 获取 APPDATA 环境变量
    let app_data = env::var("APPDATA")
        .or_else(|_| env::var("HOME").map(|home| format!("{}/.config", home)))
        .map_err(|e| format!("获取应用数据目录失败: {}", e))?;

    // 构建配置目录路径: %APPDATA%\com.cubezhao.aug-session-sync\
    let config_dir = PathBuf::from(app_data).join("com.cubezhao.aug-session-sync");

    // 确保目录存在
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)
            .map_err(|e| format!("创建应用数据目录失败: {}", e))?;
    }

    Ok(config_dir.join("config.json"))
}

/// 加载配置
#[tauri::command]
pub fn load_config() -> Result<AppConfig, String> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        // 配置文件不存在，返回默认配置
        return Ok(AppConfig::default());
    }

    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;

    let config: AppConfig = serde_json::from_str(&content)
        .map_err(|e| format!("解析配置文件失败: {}", e))?;

    Ok(config)
}

/// 保存配置
#[tauri::command]
pub fn save_config(config: AppConfig) -> Result<(), String> {
    let config_path = get_config_path()?;

    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;

    fs::write(&config_path, content)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;

    Ok(())
}

