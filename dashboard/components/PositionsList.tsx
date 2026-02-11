'use client';

import { Position } from '@/lib/api';

interface Props {
    positions: Position[];
    loading: boolean;
}

export default function PositionsList({ positions, loading }: Props) {
    if (loading) {
        return (
            <div className="glass-card p-6 animate-pulse">
                <div className="h-4 bg-bg-hover rounded w-1/4 mb-4"></div>
                <div className="h-20 bg-bg-hover rounded"></div>
            </div>
        );
    }

    return (
        <div className="glass-card p-6">
            <h2 className="text-lg font-semibold text-text-primary mb-4">
                Open Positions
            </h2>

            {positions.length === 0 ? (
                <div className="py-6 text-center text-text-muted text-sm">
                    No open positions
                </div>
            ) : (
                <div className="space-y-3">
                    {positions.map((pos) => {
                        const current = pos.current_price || pos.entry_price;
                        const pnlPct =
                            ((current - pos.entry_price) / pos.entry_price) * 100;
                        const pnlColor = pnlPct >= 0 ? 'text-accent-green' : 'text-accent-red';

                        return (
                            <div
                                key={pos.id}
                                className="p-4 rounded-xl bg-bg-secondary/50 border border-white/[0.03]"
                            >
                                <div className="flex items-center justify-between mb-2">
                                    <span className="font-bold text-text-primary">
                                        {pos.symbol.replace('USDC', '')}
                                        <span className="text-text-muted font-normal">/USDC</span>
                                    </span>
                                    <span className={`font-mono font-bold ${pnlColor}`}>
                                        {pnlPct >= 0 ? '+' : ''}
                                        {pnlPct.toFixed(2)}%
                                    </span>
                                </div>

                                <div className="grid grid-cols-3 gap-2 text-xs">
                                    <div>
                                        <span className="text-text-muted">Entry</span>
                                        <p className="font-mono text-text-primary">
                                            ${pos.entry_price.toFixed(pos.entry_price > 1 ? 2 : 6)}
                                        </p>
                                    </div>
                                    <div>
                                        <span className="text-text-muted">Current</span>
                                        <p className={`font-mono ${pnlColor}`}>
                                            ${current.toFixed(current > 1 ? 2 : 6)}
                                        </p>
                                    </div>
                                    <div>
                                        <span className="text-text-muted">Qty</span>
                                        <p className="font-mono text-text-primary">
                                            {pos.quantity.toFixed(6)}
                                        </p>
                                    </div>
                                </div>

                                <div className="flex gap-4 mt-2 text-[10px] text-text-muted">
                                    {pos.stop_loss && (
                                        <span>
                                            SL: ${pos.stop_loss.toFixed(pos.stop_loss > 1 ? 2 : 6)}
                                        </span>
                                    )}
                                    {pos.take_profit && (
                                        <span>
                                            TP: ${pos.take_profit.toFixed(
                                                pos.take_profit > 1 ? 2 : 6
                                            )}
                                        </span>
                                    )}
                                </div>
                            </div>
                        );
                    })}
                </div>
            )}
        </div>
    );
}
