export type CodecType = 'pcm' | 'opus';

export interface WebSocketStatus {
  readonly CONNECTING: 'connecting';
  readonly OPEN: 'open';
  readonly CLOSING: 'closing';
  readonly CLOSED: 'closed';
  readonly RECONNECTING: 'reconnecting';
}

export type WebSocketState = 'connecting' | 'open' | 'reconnecting' | 'closed';

export interface UseWebSocketOptions {
  onOpen?: (event: WebSocketEventMap['open']) => void;
  onMessage?: (data: string | ArrayBuffer) => void;
  onClose?: (event: WebSocketEventMap['close']) => void;
  onError?: (event: WebSocketEventMap['error']) => void;
  // 重连配置
  autoReconnect?: boolean;
  maxRetries?: number;
  fallbackDelay?: number; // 指数退避基数 ms
}
