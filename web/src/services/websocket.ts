import type { LiveDataPoint, WsMessage } from '../types';

export type LiveDataCallback = (data: LiveDataPoint) => void;
export type CanFrameCallback = (id: number, data: number[], timestamp: number) => void;

export class CanaryWebSocket {
  private ws: WebSocket | null = null;
  private url: string;
  private reconnectInterval: number;
  private onLiveData: LiveDataCallback | null = null;
  private onCanFrame: CanFrameCallback | null = null;
  private onConnectionChange: ((connected: boolean) => void) | null = null;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;

  constructor(url: string = `ws://${window.location.host}/api/v1/stream/live`) {
    this.url = url;
    this.reconnectInterval = 3000;
  }

  connect(): void {
    if (this.ws?.readyState === WebSocket.OPEN) return;

    this.ws = new WebSocket(this.url);

    this.ws.onopen = () => {
      console.log('[WS] Connected');
      this.onConnectionChange?.(true);
    };

    this.ws.onmessage = (event) => {
      try {
        const msg: WsMessage = JSON.parse(event.data);
        this.handleMessage(msg);
      } catch (e) {
        console.warn('[WS] Failed to parse message:', e);
      }
    };

    this.ws.onclose = () => {
      console.log('[WS] Disconnected, reconnecting...');
      this.onConnectionChange?.(false);
      this.scheduleReconnect();
    };

    this.ws.onerror = (error) => {
      console.error('[WS] Error:', error);
    };
  }

  disconnect(): void {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    this.ws?.close();
    this.ws = null;
  }

  subscribe(ecuId: number, pids: number[]): void {
    this.send({
      type: 'Subscribe',
      data: { ecu_id: ecuId, pids },
    });
  }

  unsubscribe(ecuId: number): void {
    this.send({
      type: 'Unsubscribe',
      data: { ecu_id: ecuId },
    });
  }

  setOnLiveData(callback: LiveDataCallback): void {
    this.onLiveData = callback;
  }

  setOnCanFrame(callback: CanFrameCallback): void {
    this.onCanFrame = callback;
  }

  setOnConnectionChange(callback: (connected: boolean) => void): void {
    this.onConnectionChange = callback;
  }

  get isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }

  private handleMessage(msg: WsMessage): void {
    switch (msg.type) {
      case 'LiveData':
        this.onLiveData?.(msg.data);
        break;
      case 'CanFrame':
        this.onCanFrame?.(msg.data.id, msg.data.data, msg.data.timestamp_us);
        break;
      case 'Ping':
        this.send({ type: 'Pong' });
        break;
      case 'Error':
        console.error('[WS] Server error:', msg.data.message);
        break;
    }
  }

  private send(msg: WsMessage): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(msg));
    }
  }

  private scheduleReconnect(): void {
    if (this.reconnectTimer) return;
    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = null;
      this.connect();
    }, this.reconnectInterval);
  }
}

export const wsClient = new CanaryWebSocket();
