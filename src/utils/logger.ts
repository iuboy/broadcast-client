/**
 * 前端日志工具
 *
 * 同时输出到控制台和文件（通过 Tauri IPC）
 */

import { invoke } from '@tauri-apps/api/core';

type LogLevel = 'info' | 'warn' | 'error';

let logEnabled = true;

/**
 * 设置日志开关
 */
export function setLogEnabled(enabled: boolean) {
  logEnabled = enabled;
}

/**
 * 记录日志到文件
 */
async function logToFile(level: LogLevel, tag: string, message: string) {
  if (!logEnabled) return;
  try {
    await invoke('log_to_file', { level, tag, message });
  } catch {
    // 忽略日志写入失败
  }
}

/**
 * 信息日志
 */
export async function logInfo(tag: string, message: string) {
  console.log(`[${tag}] ${message}`);
  await logToFile('info', tag, message);
}

/**
 * 警告日志
 */
export async function logWarn(tag: string, message: string) {
  console.warn(`[${tag}] ${message}`);
  await logToFile('warn', tag, message);
}

/**
 * 错误日志
 */
export async function logError(tag: string, message: string) {
  console.error(`[${tag}] ${message}`);
  await logToFile('error', tag, message);
}

/**
 * 默认日志器（App 专用）
 */
export const logger = {
  info: (message: string) => logInfo('App', message),
  warn: (message: string) => logWarn('App', message),
  error: (message: string) => logError('App', message),
};

/**
 * 创建带标签的日志器
 */
export function createLogger(tag: string) {
  return {
    info: (message: string) => logInfo(tag, message),
    warn: (message: string) => logWarn(tag, message),
    error: (message: string) => logError(tag, message),
  };
}
