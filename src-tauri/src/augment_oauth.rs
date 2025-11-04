use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use regex::Regex;

const CLIENT_ID: &str = "v";
const AUTH_BASE_URL: &str = "https://auth.augmentcode.com";

#[derive(Debug, Serialize, Deserialize)]
pub struct AugmentTokenResponse {
    pub access_token: String,
    pub tenant_url: String,
    pub email: Option<String>,
    pub credits_balance: Option<i32>,
    pub expiry_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenApiResponse {
    pub access_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelsResponse {
    pub user: UserData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserData {
    pub id: String,
    pub email: String,
    pub tenant_id: String,
    pub tenant_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreditInfoResponse {
    pub usage_units_remaining: f64,
    pub usage_units_total_current_billing_cycle: f64,
    pub usage_units_total_additional: f64,
    pub is_credit_balance_low: bool,
    pub display_info: Option<serde_json::Value>,
    pub refreshed_at: String,
    pub included_usage_units_per_billing_cycle: f64,
    pub current_billing_cycle_end_date_iso: String,
    pub credit_details: Option<Vec<serde_json::Value>>,
    pub usage_units_total: f64,
}

/// 从 auth session 中提取 access token
pub async fn extract_token_from_session(session: &str) -> Result<AugmentTokenResponse, String> {
    println!("=== 开始从 Session 提取 Token ===");
    println!("Session (masked): {}...{}", &session[..10.min(session.len())], if session.len() > 10 { &session[session.len()-10..] } else { "" });

    // 步骤1: 生成 PKCE 参数
    println!("步骤1: 生成 PKCE 参数");
    let code_verifier = generate_random_string(32);
    let code_challenge = base64_url_encode(&sha256_hash(code_verifier.as_bytes()));
    let state = generate_random_string(42);
    let client_id = CLIENT_ID;
    println!("  - code_verifier 长度: {}", code_verifier.len());
    println!("  - code_challenge: {}", code_challenge);
    println!("  - state 长度: {}", state.len());

    // 步骤2: 构建 terms-accept URL
    println!("步骤2: 构建 terms-accept URL");
    let terms_url = format!(
        "{}/terms-accept?response_type=code&code_challenge={}&client_id={}&state={}&prompt=login",
        AUTH_BASE_URL, code_challenge, client_id, state
    );
    println!("  - URL: {}", terms_url);

    // 步骤3: 使用 session cookie 访问 terms-accept 页面
    println!("步骤3: 访问 terms-accept 页面");
    let client = crate::http_client::create_client()?;
    let html_response = client
        .get(&terms_url)
        .header("Cookie", format!("session={}", session))
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .send()
        .await
        .map_err(|e| format!("访问 terms-accept 页面失败: {}", e))?;

    println!("  - HTTP 状态码: {}", html_response.status());

    let html = html_response.text().await
        .map_err(|e| format!("读取 HTML 响应失败: {}", e))?;
    
    println!("  - HTML 长度: {} 字符", html.len());

    // 步骤4: 从 HTML 中提取授权码、state 和 tenant_url
    println!("步骤4: 从 HTML 提取授权码和 tenant_url");
    let code_regex = Regex::new(r#"code:\s*"([^"]+)""#).unwrap();
    let state_regex = Regex::new(r#"state:\s*"([^"]+)""#).unwrap();
    let tenant_url_regex = Regex::new(r#"tenant_url:\s*"([^"]+)""#).unwrap();

    let code = code_regex.captures(&html)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str())
        .ok_or("SESSION_ERROR_OR_ACCOUNT_BANNED: 无法提取授权码")?;

    let parsed_state = state_regex.captures(&html)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str())
        .ok_or("SESSION_ERROR_OR_ACCOUNT_BANNED: 无法提取 state")?;

    let tenant_url = tenant_url_regex.captures(&html)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str())
        .ok_or("SESSION_ERROR_OR_ACCOUNT_BANNED: 无法提取 tenant_url")?;

    println!("  - 授权码 (masked): {}...", &code[..10.min(code.len())]);
    println!("  - State (masked): {}...", &parsed_state[..10.min(parsed_state.len())]);
    println!("  - Tenant URL: {}", tenant_url);

    // 步骤5: 使用授权码交换 access token
    println!("步骤5: 交换 access token");
    let token_url = format!("{}token", tenant_url);
    let token_payload = serde_json::json!({
        "grant_type": "authorization_code",
        "client_id": client_id,
        "code_verifier": code_verifier,
        "redirect_uri": "",
        "code": code
    });

    println!("  - Token URL: {}", token_url);

    let token_response = client
        .post(&token_url)
        .header("Content-Type", "application/json")
        .json(&token_payload)
        .send()
        .await
        .map_err(|e| format!("交换 token 失败: {}", e))?;

    println!("  - HTTP 状态码: {}", token_response.status());

    let token_data: TokenApiResponse = token_response.json().await
        .map_err(|e| format!("解析 token 响应失败: {}", e))?;

    println!("  - Access token (masked): {}...", &token_data.access_token[..20.min(token_data.access_token.len())]);

    // 步骤6: 并行获取用户邮箱和积分信息
    println!("步骤6: 并行获取用户信息和积分信息");
    let token = token_data.access_token.clone();
    let tenant_url_clone = tenant_url.to_string();

    let (email_result, credit_result) = tokio::join!(
        get_models(&token, &tenant_url_clone),
        get_credit_info(&token, &tenant_url_clone)
    );

    // 步骤7: 处理邮箱结果
    println!("步骤7: 处理用户邮箱");
    let email = match email_result {
        Ok(models_response) => {
            println!("  - 邮箱: {}", models_response.user.email);
            Some(models_response.user.email)
        },
        Err(err) => {
            println!("  - 获取邮箱失败: {}", err);
            None
        }
    };

    // 步骤8: 处理积分信息结果
    println!("步骤8: 处理积分信息");
    let (credits_balance, expiry_date) = match credit_result {
        Ok(credit_info) => {
            let balance = credit_info.usage_units_remaining.floor() as i32;
            println!("  - 积分余额: {}", balance);
            println!("  - 过期时间: {}", credit_info.current_billing_cycle_end_date_iso);
            (
                Some(balance),
                Some(credit_info.current_billing_cycle_end_date_iso),
            )
        },
        Err(err) => {
            println!("  - 获取积分信息失败: {}", err);
            (None, None)
        }
    };

    println!("=== Token 提取成功 ===");

    // 步骤9: 返回完整的 token 响应
    Ok(AugmentTokenResponse {
        access_token: token_data.access_token,
        tenant_url: tenant_url.to_string(),
        email,
        credits_balance,
        expiry_date,
    })
}

/// 获取用户邮箱
pub async fn get_models(token: &str, tenant_url: &str) -> Result<ModelsResponse, String> {
    let client = crate::http_client::create_client()?;
    let base_url = if tenant_url.ends_with('/') {
        tenant_url.to_string()
    } else {
        format!("{}/", tenant_url)
    };

    let api_url = format!("{}get-models", base_url);

    let response = client
        .post(&api_url)
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {}", token))
        .json(&serde_json::json!({}))
        .send()
        .await
        .map_err(|e| format!("HTTP 请求失败: {}", e))?;

    if !response.status().is_success() {
        let error_body = response.text().await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("API 请求失败: {}", error_body));
    }

    let models_info: ModelsResponse = response.json().await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    Ok(models_info)
}

/// 获取积分余额
pub async fn get_credit_info(token: &str, tenant_url: &str) -> Result<CreditInfoResponse, String> {
    let client = crate::http_client::create_client()?;
    let base_url = if tenant_url.ends_with('/') {
        tenant_url.to_string()
    } else {
        format!("{}/", tenant_url)
    };

    let api_url = format!("{}get-credit-info", base_url);

    let response = client
        .post(&api_url)
        .header("Content-Type", "application/json")
        .header("Authorization", &format!("Bearer {}", token))
        .json(&serde_json::json!({}))
        .send()
        .await
        .map_err(|e| format!("HTTP 请求失败: {}", e))?;

    if !response.status().is_success() {
        let error_body = response.text().await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("API 请求失败: {}", error_body));
    }

    let credit_info: CreditInfoResponse = response.json().await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    Ok(credit_info)
}

// 辅助函数
fn generate_random_string(length: usize) -> String {
    use rand::RngCore;
    let mut rng = rand::thread_rng();
    let mut random_bytes = vec![0u8; length];
    rng.fill_bytes(&mut random_bytes);
    base64_url_encode(&random_bytes)
}

fn base64_url_encode(data: &[u8]) -> String {
    general_purpose::URL_SAFE_NO_PAD.encode(data)
}

fn sha256_hash(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

