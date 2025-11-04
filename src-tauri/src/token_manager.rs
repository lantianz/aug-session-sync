use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PortalInfo {
    pub credits_balance: Option<i32>,
    pub expiry_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenRecord {
    pub id: String,
    pub tenant_url: String,
    pub access_token: String,
    pub created_at: String,
    pub updated_at: String,
    pub portal_url: Option<String>,
    pub ban_status: String,
    pub portal_info: Option<PortalInfo>,
    pub email_note: Option<String>,
    pub tag_name: Option<String>,
    pub tag_color: Option<String>,
    pub auth_session: String,
    pub suspensions: Option<String>,
    pub skip_check: bool,
    pub balance_color_mode: Option<String>,
}

// 远端 API 返回的 Token 数据结构（字段可选）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemoteTokenRecord {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub tenant_url: Option<String>,
    #[serde(default)]
    pub access_token: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub portal_url: Option<String>,
    #[serde(default)]
    pub ban_status: Option<String>,
    #[serde(default)]
    pub portal_info: Option<PortalInfo>,
    #[serde(default)]
    pub email_note: Option<String>,
    #[serde(default)]
    pub tag_name: Option<String>,
    #[serde(default)]
    pub tag_color: Option<String>,
    #[serde(default)]
    pub auth_session: Option<String>,
    #[serde(default)]
    pub suspensions: Option<String>,
    #[serde(default)]
    pub skip_check: Option<bool>,
    #[serde(default)]
    pub balance_color_mode: Option<String>,

    // 兼容其他可能的字段名
    #[serde(default, alias = "tenantUrl")]
    pub tenant_url_alt: Option<String>,
    #[serde(default, alias = "accessToken")]
    pub access_token_alt: Option<String>,
    #[serde(default, alias = "authSession")]
    pub auth_session_alt: Option<String>,
    #[serde(default, alias = "emailNote")]
    pub email_note_alt: Option<String>,
    #[serde(default, alias = "banStatus")]
    pub ban_status_alt: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoteApiResponse {
    pub status: i32,
    pub data: Vec<RemoteTokenRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportResult {
    pub imported: usize,
    pub skipped: usize,
}

impl RemoteTokenRecord {
    /// 转换为本地 TokenRecord 格式，并填充缺失字段的默认值
    /// 必需字段：id, auth_session, created_at
    /// 其他字段：填充默认值或 null
    fn to_local_token(&self) -> Result<TokenRecord, String> {
        println!("    📝 开始转换远端数据为本地格式...");

        // ========== 第一步：提取远端 API 返回的必需字段 ==========

        // 必需字段 1: id
        let id = self.id.clone()
            .ok_or("缺少必需字段: id")?;
        println!("      ✓ id: {}", id);

        // 必需字段 2: auth_session
        let auth_session = self.auth_session.clone()
            .or_else(|| self.auth_session_alt.clone())
            .ok_or("缺少必需字段: auth_session")?;
        println!("      ✓ auth_session: {}", if auth_session.len() > 20 { &auth_session[..20] } else { &auth_session });

        // 必需字段 3: created_at
        let created_at = self.created_at.clone()
            .ok_or("缺少必需字段: created_at")?;
        println!("      ✓ created_at: {}", created_at);

        // ========== 第二步：提取可选字段（如果远端提供） ==========

        // 提取可选字段
        let tenant_url = self.tenant_url.clone()
            .or_else(|| self.tenant_url_alt.clone());
        if let Some(ref url) = tenant_url {
            println!("      ✓ tenant_url: {}", url);
        }

        let access_token = self.access_token.clone()
            .or_else(|| self.access_token_alt.clone());
        if let Some(ref token) = access_token {
            println!("      ✓ access_token: {}", if token.len() > 20 { &token[..20] } else { token });
        }

        let email_note = self.email_note.clone()
            .or_else(|| self.email_note_alt.clone());
        if let Some(ref email) = email_note {
            println!("      ✓ email_note: {}", email);
        }

        let portal_info = self.portal_info.clone();
        if let Some(ref info) = portal_info {
            println!("      ✓ portal_info: credits={:?}, expiry={:?}",
                info.credits_balance, info.expiry_date);
        }

        let ban_status = self.ban_status.clone()
            .or_else(|| self.ban_status_alt.clone());
        if let Some(ref status) = ban_status {
            println!("      ✓ ban_status: {}", status);
        }

        // ========== 第三步：填充缺失字段的默认值 ==========
        println!("    🔧 填充缺失字段的默认值或 null...");

        // tenant_url: 默认空字符串
        let tenant_url = tenant_url.unwrap_or_else(|| {
            println!("      → tenant_url: \"\" (默认空字符串)");
            String::new()
        });

        // access_token: 默认空字符串
        let access_token = access_token.unwrap_or_else(|| {
            println!("      → access_token: \"\" (默认空字符串)");
            String::new()
        });

        // updated_at: 使用 created_at 的值
        let updated_at = self.updated_at.clone().unwrap_or_else(|| {
            println!("      → updated_at: {} (使用 created_at 的值)", created_at);
            created_at.clone()
        });

        // ban_status: 默认 "ACTIVE"
        let ban_status = ban_status.unwrap_or_else(|| {
            println!("      → ban_status: \"ACTIVE\" (默认值)");
            "ACTIVE".to_string()
        });

        // skip_check: 默认 false
        let skip_check = self.skip_check.unwrap_or_else(|| {
            println!("      → skip_check: false (默认值)");
            false
        });

        // 其他字段保持 null
        if self.portal_url.is_none() {
            println!("      → portal_url: null");
        }
        if email_note.is_none() {
            println!("      → email_note: null");
        }
        if portal_info.is_none() {
            println!("      → portal_info: null");
        }
        if self.tag_name.is_none() {
            println!("      → tag_name: null");
        }
        if self.tag_color.is_none() {
            println!("      → tag_color: null");
        }
        if self.suspensions.is_none() {
            println!("      → suspensions: null");
        }
        if self.balance_color_mode.is_none() {
            println!("      → balance_color_mode: null");
        }

        println!("    ✅ 转换完成");

        Ok(TokenRecord {
            id,
            tenant_url,
            access_token,
            created_at,
            updated_at,
            portal_url: self.portal_url.clone(),
            ban_status,
            portal_info,
            email_note,
            tag_name: self.tag_name.clone(),
            tag_color: self.tag_color.clone(),
            auth_session,
            suspensions: self.suspensions.clone(),
            skip_check,
            balance_color_mode: self.balance_color_mode.clone(),
        })
    }
}

/// 获取 tokens.json 文件路径
/// 路径: %APPDATA%\com.lantianzhi.aug-session-sync\tokens.json
fn get_tokens_file_path() -> Result<PathBuf, String> {
    use std::env;

    // 获取 APPDATA 环境变量
    let app_data = env::var("APPDATA")
        .or_else(|_| env::var("HOME").map(|home| format!("{}/.config", home)))
        .map_err(|e| format!("获取应用数据目录失败: {}", e))?;

    // 构建应用数据目录路径
    let app_dir = PathBuf::from(app_data).join("com.lantianzhi.aug-session-sync");

    // 确保目录存在
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir)
            .map_err(|e| format!("创建应用数据目录失败: {}", e))?;
    }

    Ok(app_dir.join("tokens.json"))
}

/// 读取 tokens.json 文件
#[tauri::command]
pub async fn read_tokens() -> Result<Vec<TokenRecord>, String> {
    let file_path = get_tokens_file_path()?;
    
    // 如果文件不存在，创建空数组文件
    if !file_path.exists() {
        fs::write(&file_path, "[]")
            .map_err(|e| format!("创建 tokens.json 失败: {}", e))?;
        return Ok(vec![]);
    }
    
    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("读取 tokens.json 失败: {}", e))?;
    
    let tokens: Vec<TokenRecord> = serde_json::from_str(&content)
        .map_err(|e| format!("解析 tokens.json 失败: {}", e))?;
    
    Ok(tokens)
}

/// 写入 tokens.json 文件
#[tauri::command]
pub async fn write_tokens(tokens: Vec<TokenRecord>) -> Result<(), String> {
    let file_path = get_tokens_file_path()?;
    
    let json_string = serde_json::to_string_pretty(&tokens)
        .map_err(|e| format!("序列化 tokens 失败: {}", e))?;
    
    fs::write(&file_path, json_string)
        .map_err(|e| format!("写入 tokens.json 失败: {}", e))?;
    
    Ok(())
}

/// 添加单个 token 记录
#[tauri::command]
pub async fn add_token(token: TokenRecord) -> Result<(), String> {
    let mut tokens = read_tokens().await?;
    
    // 检查是否已存在相同的 auth_session
    if tokens.iter().any(|t| t.auth_session == token.auth_session) {
        return Err("该 Session 已存在".to_string());
    }
    
    tokens.push(token);
    write_tokens(tokens).await?;
    
    Ok(())
}

/// 从远端 API 导入 tokens
#[tauri::command]
pub async fn import_from_remote(api_url: String) -> Result<ImportResult, String> {
    println!("=== 后端：开始从远端 API 导入 ===");
    println!("API 地址: {}", api_url);

    // 调用远端 API
    println!("步骤1: 创建 HTTP 客户端...");
    let client = crate::http_client::create_client()?;

    println!("步骤2: 发送 GET 请求...");
    let response = client
        .get(&api_url)
        .send()
        .await
        .map_err(|e| format!("请求远端 API 失败: {}", e))?;

    println!("步骤3: 接收响应");
    println!("  - HTTP 状态码: {}", response.status());
    println!("  - 响应头:");
    for (key, value) in response.headers() {
        println!("    {}: {:?}", key, value);
    }

    if !response.status().is_success() {
        println!("=== 后端：导入失败（HTTP 错误）===");
        return Err(format!("远端 API 返回错误: {}", response.status()));
    }

    println!("步骤4: 解析 JSON 响应体...");
    let response_text = response.text().await
        .map_err(|e| format!("读取响应体失败: {}", e))?;
    println!("  - 响应体长度: {} 字节", response_text.len());
    println!("  - 响应体内容（前 500 字符）: {}",
        if response_text.len() > 500 { &response_text[..500] } else { &response_text });

    let api_response: RemoteApiResponse = serde_json::from_str(&response_text)
        .map_err(|e| {
            println!("  ❌ JSON 解析失败: {}", e);
            println!("  完整响应体: {}", response_text);
            format!("解析远端 API 响应失败: {}", e)
        })?;

    println!("步骤5: 验证响应数据");
    println!("  - status 字段: {}", api_response.status);
    println!("  - data 数组长度: {}", api_response.data.len());

    // 打印第一条记录的结构（用于调试）
    if !api_response.data.is_empty() {
        println!("  - 第一条记录示例:");
        println!("    {:?}", api_response.data[0]);
    }

    // 检查 status 字段
    if api_response.status != 1 {
        println!("=== 后端：导入失败（status != 1）===");
        return Err("远端 API 返回失败状态".to_string());
    }

    println!("远端 API 返回 {} 条记录", api_response.data.len());
    
    // 读取本地 tokens
    println!("步骤6: 读取本地 tokens...");
    let mut local_tokens = read_tokens().await?;
    println!("  - 本地现有记录数: {}", local_tokens.len());

    // 转换并合并数据（填充默认值 + 去重）
    println!("步骤7: 转换远端数据并填充默认值...");
    println!("  说明: 远端 API 只返回核心字段，本地会自动填充缺失字段的默认值");
    println!("");

    let mut imported = 0;
    let mut skipped = 0;
    let mut conversion_errors = 0;

    for (index, remote_token) in api_response.data.iter().enumerate() {
        println!("  📦 处理第 {} 条记录", index + 1);

        // 转换为本地格式（提取远端字段 + 填充默认值）
        let local_token = match remote_token.to_local_token() {
            Ok(token) => {
                println!("    ✅ 转换成功");
                token
            },
            Err(e) => {
                conversion_errors += 1;
                println!("    ❌ 转换失败: {}", e);
                println!("    原始数据: {:?}", remote_token);
                println!("");
                continue;
            }
        };

        // 检查是否重复（基于 auth_session）
        if local_tokens.iter().any(|t| t.auth_session == local_token.auth_session) {
            skipped += 1;
            println!("    ⏭️  跳过重复记录 (auth_session 已存在)");
            println!("    邮箱: {}", local_token.email_note.as_deref().unwrap_or("未知"));
        } else {
            println!("    ✅ 添加到本地数据库");
            println!("    邮箱: {}", local_token.email_note.as_deref().unwrap_or("未知"));
            local_tokens.push(local_token);
            imported += 1;
        }
        println!("");
    }

    if conversion_errors > 0 {
        println!("  ⚠️  转换错误统计: {} 条记录无法转换（缺少必需字段）", conversion_errors);
        println!("");
    }

    // 写入本地文件
    println!("步骤8: 写入本地文件...");
    write_tokens(local_tokens).await?;

    println!("=== 后端：导入完成 ===");
    println!("  - 成功导入: {} 条", imported);
    println!("  - 跳过重复: {} 条", skipped);

    Ok(ImportResult { imported, skipped })
}

/// 删除 token 记录
#[tauri::command]
pub async fn delete_token(id: String) -> Result<(), String> {
    let mut tokens = read_tokens().await?;
    tokens.retain(|t| t.id != id);
    write_tokens(tokens).await?;
    Ok(())
}

/// 更新 token 记录
#[tauri::command]
pub async fn update_token(token: TokenRecord) -> Result<(), String> {
    let mut tokens = read_tokens().await?;
    
    if let Some(index) = tokens.iter().position(|t| t.id == token.id) {
        tokens[index] = token;
        write_tokens(tokens).await?;
        Ok(())
    } else {
        Err("未找到指定的 Token 记录".to_string())
    }
}

