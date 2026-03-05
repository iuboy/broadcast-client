// 防止在 Windows 发布版本中出现额外的控制台窗口，请勿移除!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! # 广播客户端 - Tauri 应用入口
//!
//! 这是一个语音广播客户端应用，用于连接到 broadcast-manager 服务端并推送音频数据。
//!
//! ## 功能
//! - WebSocket 连接管理
//! - 音频采集和编码（PCM/Opus）
//! - 配置文件管理
//! - 自动更新检测
//! - 日志记录
//!
//! ## 技术栈
//! - Tauri 2.x - 跨平台桌面应用框架
//! - Vue 3 + Element Plus - 前端界面
//! - tokio-tungstenite - WebSocket 客户端
//! - tracing - 日志框架

fn main() {
    broadcast_client_lib::run()
}
