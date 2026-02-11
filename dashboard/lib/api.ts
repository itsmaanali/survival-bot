const API_BASE = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3001';

export interface StatusResponse {
    is_alive: boolean;
    balance_usdc: number;
    total_pnl: number;
    open_positions: number;
    total_trades: number;
    total_cycles: number;
    win_rate: number;
    uptime_hours: number;
    last_cycle_at: string | null;
}

export interface Trade {
    id: string;
    position_id: string | null;
    symbol: string;
    side: string;
    quantity: number;
    price: number;
    usdc_amount: number;
    commission: number | null;
    executed_at: string;
}

export interface BalanceSnapshot {
    id: number;
    balance_usdc: number;
    open_positions: number;
    total_pnl: number;
    recorded_at: string;
}

export interface CycleLog {
    id: string;
    cycle_number: number;
    balance_usdc: number;
    action: string;
    symbol: string | null;
    confidence: number | null;
    reasoning: string | null;
    fear_greed: number | null;
    execution_ms: number | null;
    result: string | null;
    error: string | null;
    created_at: string;
}

export interface Position {
    id: string;
    symbol: string;
    side: string;
    quantity: number;
    entry_price: number;
    current_price: number | null;
    stop_loss: number | null;
    take_profit: number | null;
    status: string;
    pnl: number | null;
    opened_at: string;
}

async function fetchApi<T>(endpoint: string): Promise<T> {
    const res = await fetch(`${API_BASE}${endpoint}`, {
        cache: 'no-store',
    });
    if (!res.ok) throw new Error(`API error: ${res.status}`);
    return res.json();
}

export const api = {
    getStatus: () => fetchApi<StatusResponse>('/status'),
    getTrades: () => fetchApi<Trade[]>('/trades'),
    getBalance: () => fetchApi<BalanceSnapshot[]>('/balance'),
    getCycles: () => fetchApi<CycleLog[]>('/cycles'),
    getPositions: () => fetchApi<Position[]>('/positions'),

    trigger: async () => {
        const res = await fetch(`${API_BASE}/trigger`, { method: 'POST' });
        return res.text();
    },

    kill: async (secret: string) => {
        const res = await fetch(`${API_BASE}/kill`, {
            method: 'POST',
            headers: { 'X-Kill-Secret': secret },
        });
        return res.text();
    },
};
