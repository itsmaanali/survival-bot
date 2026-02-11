'use client';

import { useEffect, useState, useCallback } from 'react';
import { api, StatusResponse, Trade, BalanceSnapshot, CycleLog, Position } from '@/lib/api';
import { wsManager } from '@/lib/websocket';
import StatusCard from '@/components/StatusCard';
import BalanceChart from '@/components/BalanceChart';
import TradeHistory from '@/components/TradeHistory';
import CycleHistory from '@/components/CycleHistory';
import Controls from '@/components/Controls';
import PositionsList from '@/components/PositionsList';

export default function Dashboard() {
    const [status, setStatus] = useState<StatusResponse | null>(null);
    const [trades, setTrades] = useState<Trade[]>([]);
    const [balance, setBalance] = useState<BalanceSnapshot[]>([]);
    const [cycles, setCycles] = useState<CycleLog[]>([]);
    const [positions, setPositions] = useState<Position[]>([]);
    const [loading, setLoading] = useState(true);
    const [connected, setConnected] = useState(false);
    const [lastUpdate, setLastUpdate] = useState<Date | null>(null);

    const fetchAll = useCallback(async () => {
        try {
            const [s, t, b, c, p] = await Promise.all([
                api.getStatus(),
                api.getTrades(),
                api.getBalance(),
                api.getCycles(),
                api.getPositions(),
            ]);
            setStatus(s);
            setTrades(t);
            setBalance(b);
            setCycles(c);
            setPositions(p);
            setConnected(true);
            setLastUpdate(new Date());
        } catch (e) {
            console.error('Failed to fetch data:', e);
            setConnected(false);
        } finally {
            setLoading(false);
        }
    }, []);

    useEffect(() => {
        fetchAll();

        // Auto-refresh every 30 seconds
        const interval = setInterval(fetchAll, 30000);

        // WebSocket for real-time updates
        wsManager.connect();
        const unsub = wsManager.subscribe(() => {
            // Refetch all data on cycle update
            fetchAll();
        });

        return () => {
            clearInterval(interval);
            unsub();
            wsManager.disconnect();
        };
    }, [fetchAll]);

    return (
        <div className="min-h-screen bg-bg-primary">
            {/* Header */}
            <header className="border-b border-white/[0.06] bg-bg-secondary/50 backdrop-blur-lg sticky top-0 z-50">
                <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4">
                    <div className="flex items-center justify-between">
                        <div className="flex items-center gap-3">
                            <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-accent-cyan to-accent-purple flex items-center justify-center text-lg font-bold">
                                S
                            </div>
                            <div>
                                <h1 className="text-xl font-bold text-text-primary tracking-tight">
                                    Survival Bot
                                </h1>
                                <p className="text-xs text-text-muted">
                                    Autonomous Crypto Trading
                                </p>
                            </div>
                        </div>
                        <div className="flex items-center gap-4">
                            {lastUpdate && (
                                <span className="text-xs text-text-muted hidden sm:block">
                                    Updated {lastUpdate.toLocaleTimeString()}
                                </span>
                            )}
                            <div className="flex items-center gap-2">
                                <div
                                    className={`w-2 h-2 rounded-full ${connected ? 'bg-accent-green' : 'bg-accent-red'
                                        }`}
                                />
                                <span className="text-xs text-text-secondary">
                                    {connected ? 'Connected' : 'Disconnected'}
                                </span>
                            </div>
                        </div>
                    </div>
                </div>
            </header>

            {/* Main Content */}
            <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
                <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
                    {/* Left Column: Status + Positions + Controls */}
                    <div className="space-y-6">
                        <StatusCard status={status} loading={loading} />
                        <PositionsList positions={positions} loading={loading} />
                        <Controls />
                    </div>

                    {/* Center + Right: Charts, Trades, Cycles */}
                    <div className="lg:col-span-2 space-y-6">
                        <BalanceChart data={balance} loading={loading} />

                        <div className="grid grid-cols-1 xl:grid-cols-2 gap-6">
                            <TradeHistory trades={trades} loading={loading} />
                            <CycleHistory cycles={cycles} loading={loading} />
                        </div>
                    </div>
                </div>
            </main>

            {/* Footer */}
            <footer className="border-t border-white/[0.04] mt-12">
                <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4">
                    <p className="text-center text-xs text-text-muted">
                        Survival Trading Bot v1.0 — Built with Rust + Next.js — Not financial advice
                    </p>
                </div>
            </footer>
        </div>
    );
}
