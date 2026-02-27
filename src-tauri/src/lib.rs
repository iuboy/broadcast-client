//! 广播客户端 - Tauri 应用入口
//!
//! 用于连接到 broadcast-manager 并推送音频数据

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;
use tauri_plugin_updater::UpdaterExt;

/// 配置文件结构
#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    /// WebSocket 服务器地址
    pub server_url: String,
    /// 默认编码格式 (pcm/opus)
    pub default_codec: String,
    /// 默认音量 (0.0 - 1.5)
    pub default_volume: f64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server_url: "ws://localhost:8080/broadcast".to_string(),
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

/// 获取配置文件路径（平台特定的应用数据目录）
fn get_config_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    // 获取平台特定的应用数据目录
    let config_dir = app.path().app_data_dir()
        .map_err(|e| format!("无法获取配置目录: {}", e))?;

    // 确保目录存在
    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("无法创建配置目录: {}", e))?;

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
    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("无法读取配置文件: {}", e))?;

    // 解析 JSON
    let config: AppConfig = serde_json::from_str(&content)
        .map_err(|e| format!("配置文件格式错误: {}", e))?;

    // 验证配置
    config.validate()?;

    Ok(config)
}

/// 写入配置文件（原子性写入）
#[tauri::command]
fn write_config(app: tauri::AppHandle, config: AppConfig) -> Result<(), String> {
    let config_path = get_config_path(&app)?;

    // 验证配置
    config.validate()?;

    // 序列化为 JSON
    let content = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;

    // 使用原子性写入：先写临时文件，然后重命名
    let temp_path = config_path.with_extension("tmp");

    // 写入临时文件
    fs::write(&temp_path, content)
        .map_err(|e| format!("写入临时文件失败: {}", e))?;

    // 原子性重命名
    fs::rename(&temp_path, &config_path)
        .map_err(|e| format!("重命名配置文件失败: {}", e))?;

    Ok(())
}

/// 获取配置文件路径（用于调试）
#[tauri::command]
fn get_config_path_str(app: tauri::AppHandle) -> Result<String, String> {
    get_config_path(&app)
        .map(|p| p.to_string_lossy().to_string())
}

/// 获取更新信息
#[tauri::command]
async fn get_update_info(app: tauri::AppHandle) -> Result<UpdateInfo, String> {
    let updater = app.updater().map_err(|e| e.to_string())?;
    if let Some(res) = updater.check().await.map_err(|e| e.to_string())? {
        let date = res.date.map(|d| d.to_string());
        Ok(UpdateInfo {
            available: true,
            version: Some(res.version.clone()),
            body: res.body.clone(),
            date,
        })
    } else {
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
    let updater = app.updater().map_err(|e| e.to_string())?;
    if let Some(res) = updater.check().await.map_err(|e| e.to_string())? {
        res.download_and_install(
            |_chunk_length, _content_length| {},
            || {},
        )
        .await
        .map(|_| "Update downloaded successfully. Restart to apply.".to_string())
        .map_err(|e| format!("Failed to install update: {}", e))
    } else {
        Err("No update available".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            eprintln!("Tauri 应用启动失败: {}", e);
            eprintln!("请检查配置文件和系统资源");
            std::process::exit(1);
        });
}
