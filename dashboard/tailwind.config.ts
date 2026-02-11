/** @type {import('tailwindcss').Config} */
module.exports = {
    content: [
        './app/**/*.{js,ts,jsx,tsx,mdx}',
        './components/**/*.{js,ts,jsx,tsx,mdx}',
    ],
    darkMode: 'class',
    theme: {
        extend: {
            colors: {
                bg: {
                    primary: '#0a0b0f',
                    secondary: '#12131a',
                    card: '#181924',
                    hover: '#1e2030',
                },
                accent: {
                    cyan: '#00d4ff',
                    green: '#00e676',
                    red: '#ff1744',
                    amber: '#ffab00',
                    purple: '#b388ff',
                },
                text: {
                    primary: '#e8eaed',
                    secondary: '#9aa0b2',
                    muted: '#5a6178',
                },
            },
            fontFamily: {
                sans: ['Inter', 'system-ui', 'sans-serif'],
                mono: ['JetBrains Mono', 'Fira Code', 'monospace'],
            },
            animation: {
                'pulse-slow': 'pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite',
                'glow': 'glow 2s ease-in-out infinite alternate',
            },
            keyframes: {
                glow: {
                    '0%': { boxShadow: '0 0 5px rgba(0, 212, 255, 0.2)' },
                    '100%': { boxShadow: '0 0 20px rgba(0, 212, 255, 0.4)' },
                },
            },
        },
    },
    plugins: [],
};
