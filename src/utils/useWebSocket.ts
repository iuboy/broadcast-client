import type { UseWebSocketOptions, WebSocketState } from '@/types/index';

export const useWebSocket = (url: string, options: UseWebSocketOptions = {}) => {
  const {
    onOpen,
    onMessage,
    onClose,
    onError,
    autoReconnect = true,
    maxRetries = 10,
    fallbackDelay = 1000,
  } = options;

  let ws: WebSocket | null = null;
  let status: WebSocketState = 'idle';
  let reconnectAttempt = 0;
  let shouldReconnect = false;
  let forcedClose = false;

  const connect = () => {
    if (ws) {
      ws.removeEventListener('open', handleOpen);
      ws.removeEventListener('message', handleMessage);
      ws.removeEventListener('close', handleClose);
      ws.removeEventListener('error', handleError);
    }

    console.log(`[WebSocket] 正在连接: ${url}`);
    setStatus('connecting');
    forcedClose = false;

    try {
      ws = new WebSocket(url);
      ws.addEventListener('open', handleOpen);
      ws.addEventListener('message', handleMessage);
      ws.addEventListener('close', handleClose);
      ws.addEventListener('error', handleError);
    } catch (err) {
      console.error('[WebSocket] 连接失败', err);
      setStatus('idle');
    }
  };

  const handleOpen = (event: WebSocketEventMap['open']) => {
    console.log('[WebSocket] 已连接');
    setStatus('open');
    reconnectAttempt = 0;
    onOpen?.(event);
  };

  const handleMessage = (event: WebSocketEventMap['message']) => {
    onMessage?.(event.data);
  };

  const handleError = (event: WebSocketEventMap['error']) => {
    console.error('[WebSocket] 发生错误', event);
    onError?.(event);
  };

  const handleClose = (event: WebSocketEventMap['close']) => {
    console.log('[WebSocket] 已关闭', event.code, event.reason);
    setStatus('idle');
    onClose?.(event);

    if (!forcedClose && shouldReconnect && autoReconnect && reconnectAttempt < maxRetries) {
      const delay = Math.min(fallbackDelay * 2 ** reconnectAttempt, 10000); // 最大 10s
      reconnectAttempt++;
      console.log(`[WebSocket] 将在 ${delay}ms 后尝试重连... (${reconnectAttempt}/${maxRetries})`);
      setStatus('reconnecting');
      setTimeout(connect, delay);
    }
  };

  const send = (data: string | ArrayBuffer): boolean => {
    // 首先检查 ws 实例是否存在
    if (!ws) {
      console.warn('[WebSocket] WebSocket 实例不存在，无法发送:', data);
      return false;
    }

    // 然后检查连接状态
    if (ws.readyState === WebSocket.OPEN) {
      ws.send(data);
      return true;
    }

    console.warn('[WebSocket] 连接未就绪，无法发送. 状态:', ws.readyState);
    return false;
  };

  const close = () => {
    forcedClose = true;
    shouldReconnect = false;
    ws?.close();
  };

  const reconnect = () => {
    shouldReconnect = true;
    close();
    setTimeout(connect, 1000);
  };

  const setStatus = (newStatus: WebSocketState) => {
    status = newStatus;
  };

  // 初始化连接
  shouldReconnect = true;
  connect();

  return {
    // 状态
    status: () => status as WebSocketState,
    // 方法
    send,
    close,
    reconnect,
    // 原生访问（谨慎使用）
    getWebSocket: () => ws,
  };
};
