use serde_json::Value;
use std::fs;
use std::path::Path;

/// 读取JSON文件内容
#[tauri::command]
pub async fn read_json_file(file_path: String) -> Result<String, String> {
    let path = Path::new(&file_path);
    
    // 如果文件不存在,返回空数组
    if !path.exists() {
        return Ok("[]".to_string());
    }
    
    let content = fs::read_to_string(path)
        .map_err(|e| format!("读取文件失败: {}", e))?;
    
    // 验证是否为有效的JSON
    serde_json::from_str::<Value>(&content)
        .map_err(|e| format!("JSON格式无效: {}", e))?;
    
    Ok(content)
}

/// 增量写入JSON文件
#[tauri::command]
pub async fn append_to_json_file(
    file_path: String,
    new_data: Value,
) -> Result<(), String> {
    let path = Path::new(&file_path);
    
    // 确保父目录存在
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("创建目录失败: {}", e))?;
    }
    
    // 读取现有数据
    let content = if path.exists() {
        fs::read_to_string(path)
            .map_err(|e| format!("读取文件失败: {}", e))?
    } else {
        "[]".to_string()
    };
    
    // 解析为数组
    let mut array: Vec<Value> = serde_json::from_str(&content)
        .unwrap_or_else(|_| vec![]);
    
    // 添加新数据
    array.push(new_data);
    
    // 序列化为格式化的JSON
    let json_str = serde_json::to_string_pretty(&array)
        .map_err(|e| format!("序列化失败: {}", e))?;
    
    // 写入文件
    fs::write(path, json_str)
        .map_err(|e| format!("写入文件失败: {}", e))?;
    
    Ok(())
}

/// 获取默认文件路径
/// 固定路径: %APPDATA%\com.cubezhao.atm\tokens.json
#[tauri::command]
pub async fn get_default_file_path() -> Result<String, String> {
    use std::env;
    use std::path::PathBuf;

    // 获取 APPDATA 环境变量
    let app_data = env::var("APPDATA")
        .or_else(|_| env::var("HOME").map(|home| format!("{}/.config", home)))
        .map_err(|e| format!("获取应用数据目录失败: {}", e))?;

    // 构建固定路径: %APPDATA%\com.cubezhao.atm\tokens.json
    let file_path = PathBuf::from(app_data)
        .join("com.cubezhao.atm")
        .join("tokens.json");

    Ok(file_path.to_string_lossy().to_string())
}

/// 选择文件
#[tauri::command]
pub async fn select_file(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::{DialogExt, FilePath};
    
    let file_path = app.dialog()
        .file()
        .set_title("选择JSON文件")
        .add_filter("JSON文件", &["json"])
        .blocking_pick_file();
    
    match file_path {
        Some(FilePath::Path(path)) => Ok(Some(path.to_string_lossy().to_string())),
        Some(FilePath::Url(_)) => Err("不支持URL路径".to_string()),
        None => Ok(None),
    }
}

