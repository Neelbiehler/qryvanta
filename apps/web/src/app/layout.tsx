import type { Metadata } from "next";

import "./globals.css";

export const metadata: Metadata = {
  title: "Qryvanta",
  description: "Metadata-driven business platform",
};

export default function RootLayout({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en">
      <body className="min-h-screen bg-app text-zinc-900 antialiased">{children}</body>
    </html>
  );
}
