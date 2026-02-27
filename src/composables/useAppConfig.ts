/**
 * 应用配置管理
 *
 * 用于读写本地配置文件（平台特定的应用数据目录）
 *
 * 配置文件路径示例：
 * - Windows: C:\Users\<用户名>\AppData\Roaming\app.iamm.broadcast-service\broadcast-client-config.json
 * - macOS: ~/Library/Application Support/app.iamm.broadcast-service/broadcast-client-config.json
 * - Linux: ~/.config/app.iamm.broadcast-service/broadcast-client-config.json
 */

import { invoke } from '@tauri-apps/api/core';
import { ElNotification } from 'element-plus';
import { APP_CONFIG } from '@/config';

/** 配置接口 */
export interface AppConfig {
  /** WebSocket 服务器地址 */
  serverUrl: string;
  /** 默认编码格式 (pcm/opus) */
  defaultCodec: string;
  /** 默认音量 (0.0 - 1.5) */
  defaultVolume: number;
}

/** 默认配置 */
// 从 config.ts 导入，保持配置一致性
const DEFAULT_CONFIG: AppConfig = {
  serverUrl: APP_CONFIG.WS_URL,
  defaultCodec: APP_CONFIG.DEFAULT_CODEC,
  defaultVolume: APP_CONFIG.DEFAULT_VOLUME,
};

/**
 * 读取配置文件
 */
export async function readConfig(): Promise<AppConfig> {
  try {
    const config = await invoke<AppConfig>('read_config');
    return config;
  } catch (error) {
    console.error('读取配置失败:', error);
    // 向用户显示通知，而不是静默返回默认值
    ElNotification.warning({
      title: '配置加载失败',
      message: '无法加载配置文件，将使用默认配置。请检查文件权限。',
      duration: 5000,
      position: 'top-right',
    });
    return { ...DEFAULT_CONFIG };
  }
}

/**
 * 写入配置文件
 */
export async function writeConfig(config: AppConfig): Promise<void> {
  try {
    await invoke('write_config', { config });
  } catch (error) {
    console.error('写入配置失败:', error);
    throw error;
  }
}

/**
 * 获取配置文件路径（用于调试）
 */
export async function getConfigPath(): Promise<string> {
  try {
    return await invoke<string>('get_config_path_str');
  } catch (error) {
    console.error('获取配置路径失败:', error);
    return '配置路径获取失败，请检查应用权限';
  }
}
