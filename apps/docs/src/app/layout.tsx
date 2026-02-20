import type { Metadata } from 'next';
import { RootProvider } from 'fumadocs-ui/provider/next';
import './global.css';

export const metadata: Metadata = {
  title: 'Qryvanta Docs',
  description: 'Documentation for Qryvanta, an open-source self-hostable business platform.',
  metadataBase: new URL(process.env.NEXT_PUBLIC_DOCS_URL ?? 'http://localhost:3000'),
};

export default function Layout({ children }: LayoutProps<'/'>) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body className="flex flex-col min-h-screen">
        <RootProvider>{children}</RootProvider>
      </body>
    </html>
  );
}
