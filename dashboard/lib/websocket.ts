type MessageHandler = (data: any) => void;

const WS_BASE = process.env.NEXT_PUBLIC_WS_URL ||
    (typeof window !== 'undefined'
        ? `ws://${window.location.hostname}:3001/ws`
        : 'ws://localhost:3001/ws');

export class WebSocketManager {
    private ws: WebSocket | null = null;
    private handlers: Set<MessageHandler> = new Set();
    private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
    private reconnectDelay = 3000;

    connect() {
        if (typeof window === 'undefined') return;

        try {
            this.ws = new WebSocket(WS_BASE);

            this.ws.onopen = () => {
                console.log('🔌 WebSocket connected');
                this.reconnectDelay = 3000; // Reset on success
            };

            this.ws.onmessage = (event) => {
                try {
                    const data = JSON.parse(event.data);
                    this.handlers.forEach((handler) => handler(data));
                } catch (e) {
                    console.error('WS parse error:', e);
                }
            };

            this.ws.onclose = () => {
                console.log('🔌 WebSocket disconnected, reconnecting...');
                this.scheduleReconnect();
            };

            this.ws.onerror = (error) => {
                console.error('WS error:', error);
                this.ws?.close();
            };
        } catch (e) {
            console.error('WS connection error:', e);
            this.scheduleReconnect();
        }
    }

    private scheduleReconnect() {
        if (this.reconnectTimer) return;
        this.reconnectTimer = setTimeout(() => {
            this.reconnectTimer = null;
            this.connect();
            // Exponential backoff up to 30s
            this.reconnectDelay = Math.min(this.reconnectDelay * 1.5, 30000);
        }, this.reconnectDelay);
    }

    subscribe(handler: MessageHandler): () => void {
        this.handlers.add(handler);
        return () => this.handlers.delete(handler);
    }

    disconnect() {
        if (this.reconnectTimer) {
            clearTimeout(this.reconnectTimer);
            this.reconnectTimer = null;
        }
        this.ws?.close();
        this.ws = null;
    }
}

// Singleton instance
export const wsManager = new WebSocketManager();
