'use client';

import { Trade } from '@/lib/api';

interface Props {
    trades: Trade[];
    loading: boolean;
}

export default function TradeHistory({ trades, loading }: Props) {
    if (loading) {
        return (
            <div className="glass-card p-6 animate-pulse">
                <div className="h-4 bg-bg-hover rounded w-1/4 mb-4"></div>
                {[...Array(5)].map((_, i) => (
                    <div key={i} className="h-10 bg-bg-hover rounded mb-2"></div>
                ))}
            </div>
        );
    }

    return (
        <div className="glass-card p-6">
            <h2 className="text-lg font-semibold text-text-primary mb-4">
                Recent Trades
            </h2>

            {trades.length === 0 ? (
                <div className="py-8 text-center text-text-muted">
                    No trades yet
                </div>
            ) : (
                <div className="overflow-x-auto">
                    <table className="w-full text-sm">
                        <thead>
                            <tr className="text-text-muted text-xs uppercase tracking-wider border-b border-white/5">
                                <th className="text-left py-3 pr-4">Time</th>
                                <th className="text-left py-3 pr-4">Symbol</th>
                                <th className="text-left py-3 pr-4">Side</th>
                                <th className="text-right py-3 pr-4">Price</th>
                                <th className="text-right py-3 pr-4">Qty</th>
                                <th className="text-right py-3">USDC</th>
                            </tr>
                        </thead>
                        <tbody>
                            {trades.map((trade) => (
                                <tr
                                    key={trade.id}
                                    className="border-b border-white/[0.03] hover:bg-bg-hover/50 transition-colors"
                                >
                                    <td className="py-3 pr-4 font-mono text-text-secondary text-xs">
                                        {new Date(trade.executed_at).toLocaleString([], {
                                            month: 'short',
                                            day: 'numeric',
                                            hour: '2-digit',
                                            minute: '2-digit',
                                        })}
                                    </td>
                                    <td className="py-3 pr-4 font-semibold text-text-primary">
                                        {trade.symbol.replace('USDC', '')}
                                        <span className="text-text-muted">/USDC</span>
                                    </td>
                                    <td className="py-3 pr-4">
                                        <span
                                            className={`px-2 py-0.5 rounded-full text-xs font-semibold ${trade.side === 'BUY'
                                                    ? 'bg-accent-green/10 text-accent-green'
                                                    : 'bg-accent-red/10 text-accent-red'
                                                }`}
                                        >
                                            {trade.side}
                                        </span>
                                    </td>
                                    <td className="py-3 pr-4 text-right font-mono text-text-primary">
                                        ${trade.price.toFixed(trade.price > 1 ? 2 : 6)}
                                    </td>
                                    <td className="py-3 pr-4 text-right font-mono text-text-secondary">
                                        {trade.quantity.toFixed(6)}
                                    </td>
                                    <td className="py-3 text-right font-mono text-text-primary">
                                        ${trade.usdc_amount.toFixed(2)}
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>
            )}
        </div>
    );
}
