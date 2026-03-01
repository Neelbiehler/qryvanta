import type { Metadata } from 'next';
import { RootProvider } from 'fumadocs-ui/provider/next';
import './global.css';

export const metadata: Metadata = {
  title: 'Qryvanta Docs',
  description: 'End-user and self-hosting documentation for the Qryvanta open-source platform.',
  metadataBase: new URL(process.env.NEXT_PUBLIC_DOCS_URL ?? 'http://127.0.0.1:3002'),
};

export default function Layout({ children }: LayoutProps<'/'>) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body className="flex min-h-screen flex-col antialiased">
        <RootProvider
          theme={{
            enabled: true,
            forcedTheme: 'light',
            defaultTheme: 'light',
            enableSystem: false,
            disableTransitionOnChange: true,
          }}
        >
          {children}
        </RootProvider>
      </body>
    </html>
  );
}
