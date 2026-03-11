//! # 广播客户端 - Tauri 应用库
//!
//! 本模块提供 Tauri 应用的核心功能，包括：
//!
//! ## 主要模块
//! - `websocket_client` - WebSocket 客户端，负责与服务端通信
//!
//! ## Tauri 命令
//! - `read_config` - 读取配置文件
//! - `write_config` - 写入配置文件
//! - `get_config_path_str` - 获取配置文件路径
//! - `get_update_info` - 获取更新信息
//! - `install_update` - 安装更新
//! - `log_to_file` - 记录日志到文件
//! - `ws_start_broadcast` - 开始广播
//! - `ws_stop_broadcast` - 停止广播
//! - `ws_send_audio` - 发送音频数据
//! - `ws_get_state` - 获取连接状态

mod websocket_client;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri_plugin_updater::UpdaterExt;
use tracing_subscriber::prelude::*;

// 导出 WebSocket 客户端命令
use websocket_client::{ws_get_state, ws_send_audio, ws_start_broadcast, ws_stop_broadcast};

/// 配置文件结构
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    /// WebSocket 服务器地址
    pub server_url: String,
    /// 更新服务器基础地址（自动拼接 /{target}/{current_version}）
    #[serde(alias = "updateServerUrl")] // 向后兼容旧的字段名
    pub update_server_base_url: String,
    /// 默认编码格式 (pcm/opus)
    pub default_codec: String,
    /// 默认音量 (0.0 - 1.5)
    pub default_volume: f64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server_url: "ws://localhost:8081/ws".to_string(),
            update_server_base_url: String::new(),
            default_codec: "pcm".to_string(),
            default_volume: 1.0,
        }
    }
}

impl AppConfig {
    /// 验证配置是否有效
    pub fn validate(&self) -> Result<(), String> {
        // 验证服务器地址
        if self.server_url.is_empty() {
            return Err("服务器地址不能为空".to_string());
        }

        // 更新服务器地址可以为空，空时使用 Tauri 内置 updater

        // 验证编码格式
        if !["pcm", "opus"].contains(&self.default_codec.as_str()) {
            return Err("编码格式必须是 'pcm' 或 'opus'".to_string());
        }

        // 验证音量范围 (0.0 - 1.5)
        if !(0.0..=1.5).contains(&self.default_volume) {
            return Err("音量必须在 0.0 到 1.5 之间".to_string());
        }

        Ok(())
    }
}

/// 更新信息
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub available: bool,
    pub version: Option<String>,
    pub body: Option<String>,
    pub date: Option<String>,
}

/// 获取配置文件路径（统一使用 ~/.broadcast-service/ 目录）
fn get_config_path(_app: &tauri::AppHandle) -> Result<PathBuf, String> {
    // 使用统一的配置目录 ~/.broadcast-service/
    let config_dir = dirs::home_dir()
        .map(|home| home.join(".broadcast-service"))
        .unwrap_or_else(|| std::path::PathBuf::from(".broadcast-service"));

    // 确保目录存在（限制权限为仅所有者可访问）
    std::fs::create_dir_all(&config_dir).map_err(|e| format!("无法创建配置目录: {}", e))?;

    // 设置目录权限为 0700（仅所有者可访问）
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(mut perms) = std::fs::metadata(&config_dir).map(|m| m.permissions()) {
            perms.set_mode(0o700);
            let _ = std::fs::set_permissions(&config_dir, perms);
        }
    }

    // 返回配置文件完整路径
    Ok(config_dir.join("broadcast-client-config.json"))
}

/// 读取配置文件
#[tauri::command]
fn read_config(app: tauri::AppHandle) -> Result<AppConfig, String> {
    let config_path = get_config_path(&app)?;

    // 如果配置文件不存在，返回默认配置
    if !config_path.exists() {
        return Ok(AppConfig::default());
    }

    // 读取配置文件
    let content =
        fs::read_to_string(&config_path).map_err(|e| format!("无法读取配置文件: {}", e))?;

    // 解析 JSON，并处理旧配置的字段兼容性
    let mut config: AppConfig =
        serde_json::from_str(&content).map_err(|e| format!("配置文件格式错误: {}", e))?;

    // 向后兼容：如果存在旧的 updateServerUrl 字段，迁移到新字段
    if config.update_server_base_url.is_empty() {
        // 尝试从原始 JSON 中读取旧字段
        if let Ok(raw) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(old_url) = raw.get("updateServerUrl").and_then(|v| v.as_str()) {
                config.update_server_base_url = old_url.to_string();
                // 自动保存迁移后的配置
                let _ = write_config_internal(&config_path, &config);
            }
        }
    }

    // 验证配置
    config.validate()?;

    Ok(config)
}

/// 内部函数：写入配置文件
fn write_config_internal(
    config_path: &std::path::PathBuf,
    config: &AppConfig,
) -> Result<(), String> {
    // 序列化为 JSON
    let content =
        serde_json::to_string_pretty(config).map_err(|e| format!("序列化配置失败: {}", e))?;

    // 使用原子性写入：先写临时文件，然后重命名
    let temp_path = config_path.with_extension("tmp");

    // 写入临时文件
    fs::write(&temp_path, content).map_err(|e| format!("写入临时文件失败: {}", e))?;

    // 设置文件权限为 0600（仅所有者可读写）
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(mut perms) = std::fs::metadata(&temp_path).map(|m| m.permissions()) {
            perms.set_mode(0o600);
            let _ = std::fs::set_permissions(&temp_path, perms);
        }
    }

    // 原子性重命名
    fs::rename(&temp_path, config_path).map_err(|e| format!("重命名配置文件失败: {}", e))?;

    Ok(())
}

/// 写入配置文件（原子性写入）
#[tauri::command]
fn write_config(app: tauri::AppHandle, config: AppConfig) -> Result<(), String> {
    let config_path = get_config_path(&app)?;

    // 验证配置
    config.validate()?;

    write_config_internal(&config_path, &config)
}

/// 获取配置文件路径（用于调试）
#[tauri::command]
fn get_config_path_str(app: tauri::AppHandle) -> Result<String, String> {
    get_config_path(&app).map(|p| p.to_string_lossy().to_string())
}

/// 获取更新信息
#[tauri::command]
async fn get_update_info(app: tauri::AppHandle) -> Result<UpdateInfo, String> {
    // 从配置读取更新服务器基础地址
    let config_path = get_config_path(&app)?;
    let update_server_base_url = if config_path.exists() {
        let content =
            fs::read_to_string(&config_path).map_err(|e| format!("无法读取配置文件: {}", e))?;
        let config: AppConfig =
            serde_json::from_str(&content).map_err(|e| format!("配置文件格式错误: {}", e))?;
        config.update_server_base_url
    } else {
        String::new()
    };

    // 检查是否使用默认（未配置）的更新服务器
    if update_server_base_url.is_empty() {
        // 使用 Tauri 内置的 updater 插件
        let updater = app.updater().map_err(|e| e.to_string())?;
        if let Some(res) = updater.check().await.map_err(|e| e.to_string())? {
            let date = res.date.map(|d| d.to_string());
            return Ok(UpdateInfo {
                available: true,
                version: Some(res.version.clone()),
                body: res.body.clone(),
                date,
            });
        }
        return Ok(UpdateInfo {
            available: false,
            version: None,
            body: None,
            date: None,
        });
    }

    // 使用自定义更新服务器
    // 获取当前平台和版本
    let target = if cfg!(target_os = "windows") {
        "windows-x86_64"
    } else if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "darwin-aarch64"
        } else {
            "darwin-x86_64"
        }
    } else if cfg!(target_os = "linux") {
        "linux-x86_64"
    } else {
        return Err("不支持的平台".to_string());
    };

    let current_version = env!("CARGO_PKG_VERSION");

    // 自动拼接 URL：基础地址 + "/" + target + "/" + current_version
    let url = if update_server_base_url.ends_with('/') {
        format!("{}{}/{}", update_server_base_url, target, current_version)
    } else {
        format!("{}/{}/{}", update_server_base_url, target, current_version)
    };

    tracing::info!("使用自定义更新服务器: {}", url);

    // 发起 HTTP 请求（连接超时 10 秒，总超时 30 秒）
    let client = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("请求更新服务器失败（连接超时或网络错误）: {}", e))?;

    if response.status().is_success() {
        // 假设返回的是 JSON 格式的更新信息
        #[derive(serde::Deserialize)]
        struct UpdateResponse {
            version: String,
            body: Option<String>,
            date: Option<String>,
            #[allow(dead_code)]
            url: Option<String>,
        }

        let update_info: UpdateResponse = response
            .json()
            .await
            .map_err(|e| format!("解析更新信息失败: {}", e))?;

        // 比较版本号
        let available = update_info.version != current_version;

        Ok(UpdateInfo {
            available,
            version: Some(update_info.version),
            body: update_info.body,
            date: update_info.date,
        })
    } else {
        tracing::warn!("更新服务器返回错误: {}", response.status());
        Ok(UpdateInfo {
            available: false,
            version: None,
            body: None,
            date: None,
        })
    }
}

/// 安装更新
#[tauri::command]
async fn install_update(app: tauri::AppHandle) -> Result<String, String> {
    // 检查是否使用自定义更新服务器
    let config_path = get_config_path(&app)?;
    let update_server_base_url = if config_path.exists() {
        let content =
            fs::read_to_string(&config_path).map_err(|e| format!("无法读取配置文件: {}", e))?;
        let config: AppConfig =
            serde_json::from_str(&content).map_err(|e| format!("配置文件格式错误: {}", e))?;
        config.update_server_base_url
    } else {
        String::new()
    };

    // 如果配置了自定义更新服务器，提示用户手动下载
    if !update_server_base_url.is_empty() {
        return Err(
            "使用自定义更新服务器时，请手动下载更新包。\n\n请联系管理员获取最新版本。".to_string(),
        );
    }

    // 使用 Tauri 内置的 updater 插件
    let updater = app.updater().map_err(|e| e.to_string())?;
    if let Some(res) = updater.check().await.map_err(|e| e.to_string())? {
        res.download_and_install(|_chunk_length, _content_length| {}, || {})
            .await
            .map(|_| "Update downloaded successfully. Restart to apply.".to_string())
            .map_err(|e| format!("Failed to install update: {}", e))
    } else {
        Err("No update available".to_string())
    }
}

/// 从前端写入日志到文件
#[tauri::command]
fn log_to_file(level: String, tag: String, message: String) {
    match level.as_str() {
        "error" => tracing::error!("[{}] {}", tag, message),
        "warn" => tracing::warn!("[{}] {}", tag, message),
        _ => tracing::info!("[{}] {}", tag, message),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志系统 - 同时输出到控制台和文件
    let log_dir = dirs::home_dir()
        .map(|home| home.join(".broadcast-service").join("logs"))
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    // 确保日志目录存在
    std::fs::create_dir_all(&log_dir).unwrap_or_else(|e| {
        eprintln!("无法创建日志目录 {:?}: {}", log_dir, e);
    });

    let log_path = log_dir.join("broadcast-client.log");
    eprintln!("日志目录: {:?}", log_dir);

    // 配置文件日志记录器（按天轮转）
    let file_appender = tracing_appender::rolling::daily(log_dir.clone(), "broadcast-client");

    // 创建非阻塞写入器
    let (non_blocking_appender, _guard) = tracing_appender::non_blocking(file_appender);

    // 配置日志订阅器 - 同时输出到控制台和文件
    let env_filter = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive(tracing::Level::INFO.into());

    // 控制台层
    let console_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true);

    // 文件层
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking_appender)
        .with_ansi(false)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .init();

    tracing::info!("广播客户端启动中...");
    tracing::info!("日志文件: {:?}", log_path);

    // 启动日志清理任务（保留最近 7 天的日志）
    let log_dir_clone = log_dir.clone();
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600)); // 每小时检查一次
        loop {
            interval.tick().await;
            if let Ok(entries) = std::fs::read_dir(&log_dir_clone) {
                let now = std::time::SystemTime::now();
                let max_age = std::time::Duration::from_secs(7 * 24 * 3600); // 7 天

                for entry in entries.filter_map(Result::ok) {
                    if let Ok(metadata) = entry.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            if let Ok(age) = now.duration_since(modified) {
                                if age > max_age {
                                    let path = entry.path();
                                    let path_str = path.to_string_lossy();
                                    // 只删除 .log 或 .log.* 文件
                                    if path
                                        .extension()
                                        .map(|s| s.to_string_lossy())
                                        .unwrap_or_default()
                                        == "log"
                                        || path_str.ends_with(".log")
                                    {
                                        let _ = std::fs::remove_file(&path);
                                        tracing::info!("清理过期日志文件: {:?}", path);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    // 保留 _guard 以防止文件日志过早关闭
    std::mem::forget(_guard);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_update_info,
            install_update,
            read_config,
            write_config,
            get_config_path_str,
            log_to_file,
            ws_start_broadcast,
            ws_stop_broadcast,
            ws_send_audio,
            ws_get_state,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            eprintln!("Tauri 应用启动失败: {}", e);
            eprintln!("请检查配置文件和系统资源");
            std::process::exit(1);
        });
}
