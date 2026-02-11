'use client';

import {
    AreaChart,
    Area,
    XAxis,
    YAxis,
    CartesianGrid,
    Tooltip,
    ResponsiveContainer,
} from 'recharts';
import { BalanceSnapshot } from '@/lib/api';

interface Props {
    data: BalanceSnapshot[];
    loading: boolean;
}

export default function BalanceChart({ data, loading }: Props) {
    if (loading) {
        return (
            <div className="glass-card p-6 animate-pulse">
                <div className="h-4 bg-bg-hover rounded w-1/4 mb-4"></div>
                <div className="h-64 bg-bg-hover rounded"></div>
            </div>
        );
    }

    // Reverse so oldest is first (left side of chart)
    const chartData = [...data].reverse().map((d) => ({
        time: new Date(d.recorded_at).toLocaleTimeString([], {
            hour: '2-digit',
            minute: '2-digit',
        }),
        balance: d.balance_usdc,
        pnl: d.total_pnl,
    }));

    const minBalance = Math.min(...chartData.map((d) => d.balance));
    const maxBalance = Math.max(...chartData.map((d) => d.balance));
    const yMin = Math.max(0, minBalance - (maxBalance - minBalance) * 0.1);
    const yMax = maxBalance + (maxBalance - minBalance) * 0.1 || 100;

    return (
        <div className="glass-card p-6">
            <h2 className="text-lg font-semibold text-text-primary mb-4">
                Balance History
            </h2>

            {chartData.length === 0 ? (
                <div className="h-64 flex items-center justify-center text-text-muted">
                    No data yet — waiting for first cycle
                </div>
            ) : (
                <ResponsiveContainer width="100%" height={280}>
                    <AreaChart data={chartData}>
                        <defs>
                            <linearGradient id="balanceGradient" x1="0" y1="0" x2="0" y2="1">
                                <stop offset="5%" stopColor="#00d4ff" stopOpacity={0.3} />
                                <stop offset="95%" stopColor="#00d4ff" stopOpacity={0} />
                            </linearGradient>
                        </defs>
                        <CartesianGrid
                            strokeDasharray="3 3"
                            stroke="rgba(255,255,255,0.05)"
                        />
                        <XAxis
                            dataKey="time"
                            stroke="#5a6178"
                            tick={{ fill: '#9aa0b2', fontSize: 11 }}
                            axisLine={{ stroke: 'rgba(255,255,255,0.1)' }}
                        />
                        <YAxis
                            domain={[yMin, yMax]}
                            stroke="#5a6178"
                            tick={{ fill: '#9aa0b2', fontSize: 11 }}
                            axisLine={{ stroke: 'rgba(255,255,255,0.1)' }}
                            tickFormatter={(v: number) => `$${v.toFixed(0)}`}
                        />
                        <Tooltip
                            contentStyle={{
                                background: '#181924',
                                border: '1px solid rgba(0, 212, 255, 0.2)',
                                borderRadius: '12px',
                                color: '#e8eaed',
                                fontSize: '13px',
                            }}
                            formatter={(value: number) => [`$${value.toFixed(2)}`, 'Balance']}
                        />
                        <Area
                            type="monotone"
                            dataKey="balance"
                            stroke="#00d4ff"
                            strokeWidth={2}
                            fill="url(#balanceGradient)"
                            dot={false}
                            activeDot={{ r: 4, fill: '#00d4ff', stroke: '#0a0b0f', strokeWidth: 2 }}
                        />
                    </AreaChart>
                </ResponsiveContainer>
            )}
        </div>
    );
}
