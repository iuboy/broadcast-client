import type { CodecType, WebSocketState } from '@/config';

export type { CodecType, WebSocketState };

export interface UseWebSocketOptions {
  onOpen?: (event: WebSocketEventMap['open']) => void;
  onMessage?: (data: string | ArrayBuffer) => void;
  onClose?: (event: WebSocketEventMap['close']) => void;
  onError?: (event: WebSocketEventMap['error']) => void;
  autoReconnect?: boolean;
  maxRetries?: number;
  fallbackDelay?: number;
}
