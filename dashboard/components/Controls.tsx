'use client';

import { useState } from 'react';
import { api } from '@/lib/api';

export default function Controls() {
    const [triggerLoading, setTriggerLoading] = useState(false);
    const [killConfirm, setKillConfirm] = useState(false);
    const [killSecret, setKillSecret] = useState('');
    const [message, setMessage] = useState('');

    const handleTrigger = async () => {
        setTriggerLoading(true);
        setMessage('');
        try {
            const result = await api.trigger();
            setMessage(`✅ ${result}`);
        } catch (e) {
            setMessage('❌ Failed to trigger cycle');
        } finally {
            setTriggerLoading(false);
        }
    };

    const handleKill = async () => {
        if (!killSecret) {
            setMessage('⚠️ Enter the kill secret');
            return;
        }
        try {
            const result = await api.kill(killSecret);
            setMessage(`🛑 ${result}`);
            setKillConfirm(false);
            setKillSecret('');
        } catch (e) {
            setMessage('❌ Kill failed — check secret');
        }
    };

    return (
        <div className="glass-card p-6">
            <h2 className="text-lg font-semibold text-text-primary mb-4">
                Controls
            </h2>

            <div className="space-y-4">
                {/* Manual Trigger */}
                <button
                    onClick={handleTrigger}
                    disabled={triggerLoading}
                    className="w-full py-3 px-4 rounded-xl font-semibold text-sm
            bg-gradient-to-r from-accent-cyan/20 to-accent-purple/20
            border border-accent-cyan/30 text-accent-cyan
            hover:from-accent-cyan/30 hover:to-accent-purple/30
            hover:border-accent-cyan/50 hover:shadow-lg hover:shadow-accent-cyan/10
            disabled:opacity-50 disabled:cursor-not-allowed
            transition-all duration-300"
                >
                    {triggerLoading ? (
                        <span className="flex items-center justify-center gap-2">
                            <svg className="animate-spin h-4 w-4" viewBox="0 0 24 24">
                                <circle
                                    className="opacity-25"
                                    cx="12"
                                    cy="12"
                                    r="10"
                                    stroke="currentColor"
                                    strokeWidth="4"
                                    fill="none"
                                />
                                <path
                                    className="opacity-75"
                                    fill="currentColor"
                                    d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
                                />
                            </svg>
                            Running Cycle...
                        </span>
                    ) : (
                        '⚡ Trigger Manual Cycle'
                    )}
                </button>

                {/* Kill Switch */}
                {!killConfirm ? (
                    <button
                        onClick={() => setKillConfirm(true)}
                        className="w-full py-3 px-4 rounded-xl font-semibold text-sm
              bg-accent-red/10 border border-accent-red/20 text-accent-red/80
              hover:bg-accent-red/20 hover:border-accent-red/40 hover:text-accent-red
              transition-all duration-300"
                    >
                        ☠️ Kill Switch
                    </button>
                ) : (
                    <div className="space-y-2 p-4 rounded-xl bg-accent-red/5 border border-accent-red/20">
                        <p className="text-xs text-accent-red font-semibold">
                            ⚠️ This will permanently stop the bot!
                        </p>
                        <input
                            type="password"
                            placeholder="Enter kill secret..."
                            value={killSecret}
                            onChange={(e) => setKillSecret(e.target.value)}
                            className="w-full py-2 px-3 rounded-lg bg-bg-primary border border-white/10
                text-text-primary text-sm placeholder:text-text-muted
                focus:outline-none focus:border-accent-red/50"
                        />
                        <div className="flex gap-2">
                            <button
                                onClick={handleKill}
                                className="flex-1 py-2 rounded-lg bg-accent-red text-white text-sm font-bold
                  hover:bg-accent-red/80 transition-colors"
                            >
                                Confirm Kill
                            </button>
                            <button
                                onClick={() => {
                                    setKillConfirm(false);
                                    setKillSecret('');
                                }}
                                className="flex-1 py-2 rounded-lg bg-bg-hover text-text-secondary text-sm
                  hover:bg-bg-hover/80 transition-colors"
                            >
                                Cancel
                            </button>
                        </div>
                    </div>
                )}

                {/* Status message */}
                {message && (
                    <p className="text-center text-sm text-text-secondary">{message}</p>
                )}
            </div>
        </div>
    );
}
