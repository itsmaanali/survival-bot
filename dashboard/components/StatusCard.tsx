'use client';

import { StatusResponse } from '@/lib/api';

interface Props {
    status: StatusResponse | null;
    loading: boolean;
}

export default function StatusCard({ status, loading }: Props) {
    if (loading) {
        return (
            <div className="glass-card p-6 animate-pulse">
                <div className="h-4 bg-bg-hover rounded w-1/3 mb-4"></div>
                <div className="h-8 bg-bg-hover rounded w-1/2 mb-2"></div>
                <div className="h-4 bg-bg-hover rounded w-2/3"></div>
            </div>
        );
    }

    if (!status) return null;

    const isAlive = status.is_alive;
    const pnlColor = status.total_pnl >= 0 ? 'text-accent-green' : 'text-accent-red';
    const pnlSign = status.total_pnl >= 0 ? '+' : '';

    return (
        <div className="glass-card gradient-border p-6 space-y-5">
            {/* Header */}
            <div className="flex items-center justify-between">
                <h2 className="text-lg font-semibold text-text-primary">Bot Status</h2>
                <div className="flex items-center gap-2">
                    <div
                        className={`w-3 h-3 rounded-full ${isAlive
                                ? 'bg-accent-green status-alive'
                                : 'bg-accent-red'
                            }`}
                    />
                    <span
                        className={`text-sm font-medium ${isAlive ? 'text-accent-green' : 'text-accent-red'
                            }`}
                    >
                        {isAlive ? 'ALIVE' : 'DEAD'}
                    </span>
                </div>
            </div>

            {/* Balance */}
            <div>
                <p className="text-text-secondary text-sm mb-1">Balance (USDC)</p>
                <p className="text-3xl font-bold neon-text font-mono">
                    ${status.balance_usdc.toFixed(2)}
                </p>
            </div>

            {/* Stats grid */}
            <div className="grid grid-cols-2 gap-4">
                <div>
                    <p className="text-text-muted text-xs uppercase tracking-wider mb-1">Total P&L</p>
                    <p className={`text-lg font-semibold font-mono ${pnlColor}`}>
                        {pnlSign}${status.total_pnl.toFixed(4)}
                    </p>
                </div>
                <div>
                    <p className="text-text-muted text-xs uppercase tracking-wider mb-1">Win Rate</p>
                    <p className="text-lg font-semibold font-mono text-text-primary">
                        {status.win_rate.toFixed(1)}%
                    </p>
                </div>
                <div>
                    <p className="text-text-muted text-xs uppercase tracking-wider mb-1">Positions</p>
                    <p className="text-lg font-semibold font-mono text-text-primary">
                        {status.open_positions}/2
                    </p>
                </div>
                <div>
                    <p className="text-text-muted text-xs uppercase tracking-wider mb-1">Total Trades</p>
                    <p className="text-lg font-semibold font-mono text-text-primary">
                        {status.total_trades}
                    </p>
                </div>
                <div>
                    <p className="text-text-muted text-xs uppercase tracking-wider mb-1">Cycles</p>
                    <p className="text-lg font-semibold font-mono text-text-primary">
                        {status.total_cycles}
                    </p>
                </div>
                <div>
                    <p className="text-text-muted text-xs uppercase tracking-wider mb-1">Uptime</p>
                    <p className="text-lg font-semibold font-mono text-text-primary">
                        {status.uptime_hours.toFixed(1)}h
                    </p>
                </div>
            </div>

            {/* Last cycle */}
            {status.last_cycle_at && (
                <p className="text-text-muted text-xs">
                    Last cycle: {new Date(status.last_cycle_at).toLocaleString()}
                </p>
            )}
        </div>
    );
}
