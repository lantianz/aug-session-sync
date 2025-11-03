use reqwest::Client;
use std::time::Duration;

/// 创建HTTP客户端（支持 cookies）
pub fn create_client() -> Result<Client, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .cookie_store(true)  // 启用 cookie 存储
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;

    Ok(client)
}

/// 从URL获取文本内容
#[tauri::command]
pub async fn fetch_text_from_url(url: String) -> Result<String, String> {
    let client = create_client()?;
    
    let response = client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("HTTP错误: {}", response.status()));
    }
    
    let text = response
        .text()
        .await
        .map_err(|e| format!("读取内容失败: {}", e))?;
    
    Ok(text)
}

