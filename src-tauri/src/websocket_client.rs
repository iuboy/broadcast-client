//! # WebSocket 客户端模块
//!
//! 在 Rust 后端处理 WebSocket 连接，绕过 Tauri WebView 的限制。
//!
//! ## 功能
//! - 建立与服务端的 WebSocket 连接
//! - 发送音频数据（PCM/Opus 编码）
//! - 处理连接状态变化
//! - 错误恢复
//!
//! ## 消息协议
//! - `ready` - 服务端就绪
//! - `start_broadcast` - 请求开始广播
//! - `broadcasting` - 广播已开始
//! - `stop_broadcast` - 请求停止广播
//! - `idle` - 广播已结束
//! - `error:<msg>` - 错误消息
//!
//! ## 心跳机制
//! 每 5 秒检查一次消息，超时时间 10 秒

use futures_util::{SinkExt, StreamExt};
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;
use tauri::Emitter;
use tokio::sync::mpsc;
use tungstenite::Message;

/// 日志宏
macro_rules! log_info {
    ($($arg:tt)*) => {
        println!("[WS Client] {}", format_args!($($arg)*));
    };
}

macro_rules! log_error {
    ($($arg:tt)*) => {
        eprintln!("[WS Client] {}", format_args!($($arg)*));
    };
}

/// 广播状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BroadcastState {
    Disconnected,
    Connecting,
    #[allow(dead_code)]
    Connected,
    Broadcasting,
}

/// WebSocket 客户端管理器
pub struct WebSocketClientManager {
    state: Arc<Mutex<BroadcastState>>,
    url: Arc<Mutex<String>>,
    codec: Arc<Mutex<String>>,
    sample_rate: Arc<Mutex<u32>>,
    channels: Arc<Mutex<u16>>,
    audio_tx: Arc<Mutex<Option<mpsc::UnboundedSender<Vec<u8>>>>>,
    shutdown_tx: Arc<Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
    task_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl WebSocketClientManager {
    /// 创建新的客户端管理器
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(BroadcastState::Disconnected)),
            url: Arc::new(Mutex::new("ws://localhost:8081/ws".to_string())),
            codec: Arc::new(Mutex::new("pcm".to_string())),
            sample_rate: Arc::new(Mutex::new(48000)),
            channels: Arc::new(Mutex::new(1)),
            audio_tx: Arc::new(Mutex::new(None)),
            shutdown_tx: Arc::new(Mutex::new(None)),
            task_handle: Arc::new(Mutex::new(None)),
        }
    }

    /// 获取当前状态
    pub fn get_state(&self) -> BroadcastState {
        *self.state.lock()
    }

    /// 设置连接参数
    pub fn set_config(&self, url: String, codec: String, sample_rate: u32, channels: u16) {
        *self.url.lock() = url;
        *self.codec.lock() = codec;
        *self.sample_rate.lock() = sample_rate;
        *self.channels.lock() = channels;
    }

    /// 发送音频数据
    pub fn send_audio(&self, data: Vec<u8>) -> Result<(), String> {
        let tx = self.audio_tx.lock();
        if let Some(tx) = tx.as_ref() {
            tx.send(data)
                .map_err(|e| format!("发送音频数据失败: {}", e))?;
            Ok(())
        } else {
            Err("未连接到服务器".to_string())
        }
    }

    /// 停止广播
    pub async fn stop_broadcast(&self) -> Result<(), String> {
        // 发送停止信号
        if let Some(tx) = self.shutdown_tx.lock().take() {
            let _ = tx.send(());
        }

        // 等待任务结束（最多等待 2 秒）
        // 注意：必须在 await 前提取出 handle，因为 MutexGuard 不是 Send
        let handle_opt = self.task_handle.lock().take();
        if let Some(handle) = handle_opt {
            match tokio::time::timeout(Duration::from_secs(2), handle).await {
                Ok(Ok(_)) => {
                    log_info!("后台任务已正常结束");
                }
                Ok(Err(e)) => {
                    log_error!("后台任务结束时报错: {:?}", e);
                }
                Err(_) => {
                    log_error!("等待后台任务结束超时");
                    // 超时后强制放弃等待，继续清理
                }
            }
        }

        // 清理音频发送器
        *self.audio_tx.lock() = None;
        *self.state.lock() = BroadcastState::Disconnected;

        Ok(())
    }

    /// 启动广播连接
    pub async fn start_broadcast(&self, app_handle: tauri::AppHandle) -> Result<(), String> {
        // 如果已经有连接，先停止并等待任务结束
        if self.get_state() != BroadcastState::Disconnected {
            log_info!("检测到旧连接，先停止...");
            self.stop_broadcast().await?;
            // 等待更长时间确保服务端资源释放和 TCP 连接关闭
            log_info!("等待服务端资源释放...");
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        *self.state.lock() = BroadcastState::Connecting;

        let url = self.url.lock().clone();
        let codec = self.codec.lock().clone();
        let sample_rate = *self.sample_rate.lock();
        let channels = *self.channels.lock();

        // 创建音频数据通道
        let (audio_tx, mut audio_rx) = mpsc::unbounded_channel::<Vec<u8>>();
        *self.audio_tx.lock() = Some(audio_tx);

        // 创建关闭信号
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();
        *self.shutdown_tx.lock() = Some(shutdown_tx);

        let state_clone = self.state.clone();
        let app_clone = app_handle.clone();
        let task_handle_clone = self.task_handle.clone();

        // 启动后台任务并保存句柄
        let handle = tokio::spawn(async move {
            if let Err(e) = Self::broadcast_task(
                &app_clone,
                &url,
                &codec,
                sample_rate,
                channels,
                &mut audio_rx,
                &mut shutdown_rx,
                state_clone,
            )
            .await
            {
                log_error!("广播任务出错: {}", e);
                let _ = app_clone.emit("broadcast-error", e.to_string());
            }
            // 任务结束时清理句柄
            *task_handle_clone.lock() = None;
        });

        *self.task_handle.lock() = Some(handle);

        Ok(())
    }

    /// 广播任务（在后台运行）
    #[allow(clippy::too_many_arguments)]
    async fn broadcast_task(
        app_handle: &tauri::AppHandle,
        url: &str,
        codec: &str,
        sample_rate: u32,
        channels: u16,
        audio_rx: &mut mpsc::UnboundedReceiver<Vec<u8>>,
        shutdown_rx: &mut tokio::sync::oneshot::Receiver<()>,
        state: Arc<Mutex<BroadcastState>>,
    ) -> Result<(), String> {
        // 构建 WebSocket URL - 确保包含 /ws 路径
        let base_url = if url.contains("/ws") {
            url.to_string()
        } else {
            format!("{}/ws", url.trim_end_matches('/'))
        };

        let url_str = format!(
            "{}?codec={}&sample_rate={}&channels={}",
            base_url, codec, sample_rate, channels
        );

        log_info!("连接到服务器: {}", url_str);

        // 连接到服务器（带超时）
        let mut socket = tokio::time::timeout(
            Duration::from_secs(5),
            tokio_tungstenite::connect_async(&url_str),
        )
        .await
        .map_err(|_| format!("连接超时（5秒）: {}", url_str))?
        .map_err(|e| format!("连接失败: {}", e))?
        .0;

        log_info!("连接成功");

        // 发送前端事件
        let _ = app_handle.emit("ws-connected", ());

        // 等待 ready 消息（带超时）
        log_info!("等待 ready 消息...");
        match tokio::time::timeout(Duration::from_secs(5), socket.next()).await {
            Ok(Some(Ok(Message::Text(text)))) => {
                if text != "ready" {
                    return Err(format!("意外的消息: {}", text));
                }
                log_info!("收到 ready 消息");
            }
            Ok(Some(Ok(msg))) => return Err(format!("意外的消息类型: {:?}", msg)),
            Ok(Some(Err(e))) => return Err(format!("读取消息失败: {}", e)),
            Ok(None) => return Err("连接关闭".to_string()),
            Err(_) => return Err("等待 ready 消息超时（5秒）".to_string()),
        }

        // 发送 start_broadcast
        log_info!("发送 start_broadcast");
        socket
            .send(Message::Text("start_broadcast".to_string()))
            .await
            .map_err(|e| format!("发送 start_broadcast 失败: {}", e))?;

        // 等待 broadcasting 响应（带超时）
        log_info!("等待 broadcasting 响应...");
        match tokio::time::timeout(Duration::from_secs(5), socket.next()).await {
            Ok(Some(Ok(Message::Text(text)))) => {
                if text != "broadcasting" {
                    if text.starts_with("error:") {
                        return Err(format!("服务器错误: {}", text));
                    }
                    return Err(format!("意外的响应: {}", text));
                }
                log_info!("收到 broadcasting 响应");
            }
            Ok(Some(Ok(msg))) => return Err(format!("意外的响应类型: {:?}", msg)),
            Ok(Some(Err(e))) => return Err(format!("读取响应失败: {}", e)),
            Ok(None) => return Err("连接关闭".to_string()),
            Err(_) => return Err("等待 broadcasting 响应超时（5秒）".to_string()),
        }

        log_info!("广播已启动");

        // 更新状态并发送事件给前端
        *state.lock() = BroadcastState::Broadcasting;
        let _ = app_handle.emit("ws-broadcasting", ());
        log_info!("已发送 ws-broadcasting 事件到前端");

        // 广播循环：接收音频数据并发送
        loop {
            tokio::select! {
                // 接收音频数据
                audio_result = audio_rx.recv() => {
                    match audio_result {
                        Some(data) => {
                            // 发送音频数据
                            socket.send(Message::Binary(data))
                                .await
                                .map_err(|e| format!("发送音频数据失败: {}", e))?;
                        }
                        None => {
                            log_info!("音频通道关闭");
                            break;
                        }
                    }
                }

                // 接收关闭信号
                _ = &mut *shutdown_rx => {
                    log_info!("收到关闭信号");
                    break;
                }

                // 接收服务端消息（超时）
                result = tokio::time::timeout(Duration::from_millis(100), socket.next()) => {
                    match result {
                        Ok(Some(Ok(msg))) => {
                            match msg {
                                Message::Text(text) => {
                                    log_info!("收到服务端消息: {}", text);
                                    if text == "idle" {
                                        let _ = app_handle.emit("ws-idle", ());
                                        break;
                                    } else if text.starts_with("error:") {
                                        let _ = app_handle.emit("ws-error", text);
                                        break;
                                    }
                                }
                                Message::Close(_) => {
                                    log_info!("服务端关闭连接");
                                    break;
                                }
                                _ => {}
                            }
                        }
                        Ok(Some(Err(e))) => {
                            log_error!("WebSocket 错误: {}", e);
                            break;
                        }
                        Ok(None) => {
                            log_info!("连接已关闭");
                            break;
                        }
                        Err(_) => {
                            // 超时，继续循环
                        }
                    }
                }
            }
        }

        // 发送 stop_broadcast
        log_info!("发送 stop_broadcast");
        if let Err(e) = socket
            .send(Message::Text("stop_broadcast".to_string()))
            .await
        {
            log_error!("发送 stop_broadcast 失败: {}", e);
        }

        // 等待服务端的 idle 响应（最多 1 秒）
        log_info!("等待服务端 idle 响应...");
        match tokio::time::timeout(Duration::from_secs(1), socket.next()).await {
            Ok(Some(Ok(Message::Text(text)))) => {
                if text == "idle" {
                    log_info!("收到 idle 响应");
                } else {
                    log_info!("收到非预期响应: {}", text);
                }
            }
            Ok(Some(Ok(msg))) => {
                log_info!("收到非预期消息类型: {:?}", msg);
            }
            Ok(Some(Err(e))) => {
                log_error!("读取响应失败: {}", e);
            }
            Ok(None) => {
                log_info!("服务端已关闭连接");
            }
            Err(_) => {
                log_info!("等待 idle 响应超时（1秒），继续关闭连接");
            }
        }

        // 关闭连接
        log_info!("关闭 WebSocket 连接");
        let _ = socket.close(None).await;

        // 更新状态
        *state.lock() = BroadcastState::Disconnected;
        let _ = app_handle.emit("ws-disconnected", ());

        log_info!("广播连接已关闭");
        Ok(())
    }
}

// 全局客户端管理器
lazy_static::lazy_static! {
    pub static ref WS_CLIENT: WebSocketClientManager = WebSocketClientManager::new();
}

/// 连接并开始广播
#[tauri::command]
pub async fn ws_start_broadcast(
    app_handle: tauri::AppHandle,
    server_url: String,
    codec: String,
    sample_rate: u32,
    channels: u16,
) -> Result<String, String> {
    log_info!(
        "ws_start_broadcast 被调用: url={}, codec={}, sampleRate={}, channels={}",
        server_url,
        codec,
        sample_rate,
        channels
    );

    WS_CLIENT.set_config(server_url, codec, sample_rate, channels);
    WS_CLIENT.start_broadcast(app_handle).await?;
    Ok("广播已启动".to_string())
}

/// 停止广播
#[tauri::command]
pub async fn ws_stop_broadcast() -> Result<String, String> {
    log_info!("ws_stop_broadcast 被调用");
    WS_CLIENT.stop_broadcast().await?;
    Ok("广播已停止".to_string())
}

/// 发送音频数据
#[tauri::command]
pub fn ws_send_audio(data: Vec<u8>) -> Result<(), String> {
    WS_CLIENT.send_audio(data)
}

/// 获取连接状态
#[tauri::command]
pub fn ws_get_state() -> String {
    match WS_CLIENT.get_state() {
        BroadcastState::Disconnected => "disconnected".to_string(),
        BroadcastState::Connecting => "connecting".to_string(),
        BroadcastState::Connected => "connected".to_string(),
        BroadcastState::Broadcasting => "broadcasting".to_string(),
    }
}
