'use client';

import { CycleLog } from '@/lib/api';

interface Props {
    cycles: CycleLog[];
    loading: boolean;
}

const actionStyles: Record<string, string> = {
    BUY: 'bg-accent-green/10 text-accent-green',
    SELL: 'bg-accent-red/10 text-accent-red',
    HOLD: 'bg-accent-amber/10 text-accent-amber',
    ERROR: 'bg-accent-red/20 text-accent-red',
};

export default function CycleHistory({ cycles, loading }: Props) {
    if (loading) {
        return (
            <div className="glass-card p-6 animate-pulse">
                <div className="h-4 bg-bg-hover rounded w-1/4 mb-4"></div>
                {[...Array(5)].map((_, i) => (
                    <div key={i} className="h-16 bg-bg-hover rounded mb-2"></div>
                ))}
            </div>
        );
    }

    return (
        <div className="glass-card p-6">
            <h2 className="text-lg font-semibold text-text-primary mb-4">
                Decision Log
            </h2>

            {cycles.length === 0 ? (
                <div className="py-8 text-center text-text-muted">
                    No cycles yet
                </div>
            ) : (
                <div className="space-y-3 max-h-[500px] overflow-y-auto pr-2">
                    {cycles.map((cycle) => (
                        <div
                            key={cycle.id}
                            className="p-4 rounded-xl bg-bg-secondary/50 border border-white/[0.03] hover:border-white/[0.08] transition-colors"
                        >
                            <div className="flex items-center justify-between mb-2">
                                <div className="flex items-center gap-3">
                                    <span
                                        className={`px-2.5 py-0.5 rounded-full text-xs font-bold ${actionStyles[cycle.action] || actionStyles.HOLD
                                            }`}
                                    >
                                        {cycle.action}
                                    </span>
                                    {cycle.symbol && (
                                        <span className="text-sm font-semibold text-text-primary">
                                            {cycle.symbol}
                                        </span>
                                    )}
                                    {cycle.confidence !== null && (
                                        <span className="text-xs text-text-muted">
                                            {cycle.confidence}% confidence
                                        </span>
                                    )}
                                </div>
                                <div className="flex items-center gap-3 text-xs text-text-muted">
                                    {cycle.fear_greed !== null && (
                                        <span>F&G: {cycle.fear_greed}</span>
                                    )}
                                    <span className="font-mono">
                                        ${cycle.balance_usdc.toFixed(2)}
                                    </span>
                                </div>
                            </div>

                            {cycle.reasoning && (
                                <p className="text-xs text-text-secondary line-clamp-2">
                                    {cycle.reasoning}
                                </p>
                            )}

                            {cycle.error && (
                                <p className="text-xs text-accent-red mt-1">
                                    ⚠️ {cycle.error}
                                </p>
                            )}

                            <div className="flex items-center justify-between mt-2">
                                <span className="text-[10px] text-text-muted font-mono">
                                    {new Date(cycle.created_at).toLocaleString()}
                                </span>
                                {cycle.execution_ms !== null && (
                                    <span className="text-[10px] text-text-muted">
                                        {cycle.execution_ms}ms
                                    </span>
                                )}
                            </div>
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
}
