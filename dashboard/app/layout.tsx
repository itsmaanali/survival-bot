import type { Metadata } from 'next';
import './globals.css';

export const metadata: Metadata = {
    title: 'Survival Bot — Trading Dashboard',
    description: 'Real-time monitoring dashboard for the Survival Trading Bot',
};

export default function RootLayout({
    children,
}: {
    children: React.ReactNode;
}) {
    return (
        <html lang="en" className="dark">
            <body className="min-h-screen bg-bg-primary">
                {children}
            </body>
        </html>
    );
}
