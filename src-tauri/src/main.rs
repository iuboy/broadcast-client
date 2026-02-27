// 防止在 Windows 发布版本中出现额外的控制台窗口，请勿移除!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    broadcast_client_lib::run()
}
