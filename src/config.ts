/**
 * 应用配置
 *
 * 这是配置的唯一来源，其他模块应从此导入配置
 */

/** 音频编码类型 */
export type CodecType = 'pcm' | 'opus';

/** WebSocket 状态类型 */
export type WebSocketState = 'idle' | 'connecting' | 'open' | 'reconnecting' | 'closed';

/**
 * 应用配置常量
 */
export const APP_CONFIG: {
  WS_URL: string;
  DEFAULT_CODEC: CodecType;
  DEFAULT_VOLUME: number;
  MAX_VOLUME: number;
  WARNING_VOLUME: number;
  MAX_BROADCAST_TIME: number;
  AUTO_RECONNECT: boolean;
  MAX_RETRIES: number;
  FALLBACK_DELAY: number;
} = {
  // WebSocket服务器地址
  // 可以通过环境变量 VITE_WS_URL 覆盖
  WS_URL: (import.meta.env.VITE_WS_URL as string) || 'ws://localhost:8081/ws',

  // 默认音频配置
  DEFAULT_CODEC: 'pcm',
  DEFAULT_VOLUME: 1.0,
  MAX_VOLUME: 1.5, // 最大音量 150%（超过会有失真风险）
  WARNING_VOLUME: 1.0, // 警告阈值 100%
  MAX_BROADCAST_TIME: 5 * 60, // 5分钟

  // 重连配置
  AUTO_RECONNECT: true,
  MAX_RETRIES: 10,
  FALLBACK_DELAY: 1000,
};

/** 本地存储键名 */
export const STORAGE_KEYS = {
  CODEC: 'broadcast-codec',
  VOLUME: 'broadcast-volume',
  DEVICE: 'broadcast-device',
  SERVER_URL: 'broadcast-server-url',
} as const;
