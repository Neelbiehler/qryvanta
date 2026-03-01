import type { Metadata } from "next";
import { Fraunces, Space_Grotesk } from "next/font/google";

import "./globals.css";

const headingFont = Fraunces({
  subsets: ["latin"],
  variable: "--font-landing-serif",
  weight: ["500", "600", "700"],
});

const bodyFont = Space_Grotesk({
  subsets: ["latin"],
  variable: "--font-landing-sans",
  weight: ["400", "500", "600", "700"],
});

const siteUrl = "https://qryvanta.org";

export const metadata: Metadata = {
  metadataBase: new URL(siteUrl),

  title: {
    default: "Qryvanta.org — Open-Source Business Platform",
    template: "%s | Qryvanta.org",
  },

  description:
    "Qryvanta is an open-source, self-hostable platform for metadata-driven business operations. Run it on your own infrastructure, or contribute to the architecture on GitHub.",

  keywords: [
    "open source",
    "business platform",
    "metadata-driven",
    "self-hostable",
    "workflow automation",
    "enterprise software",
    "Rust",
    "developer tools",
    "Qryvanta",
  ],

  authors: [{ name: "Qryvanta", url: siteUrl }],
  creator: "Qryvanta",
  publisher: "Qryvanta",

  robots: {
    index: true,
    follow: true,
    googleBot: {
      index: true,
      follow: true,
      "max-video-preview": -1,
      "max-image-preview": "large",
      "max-snippet": -1,
    },
  },

  openGraph: {
    type: "website",
    locale: "en_US",
    url: siteUrl,
    siteName: "Qryvanta.org",
    title: "Qryvanta.org — Open-Source Business Platform",
    description:
      "Open-source, self-hostable business platform for metadata-driven operations. Build, deploy, and contribute to Qryvanta.",
    images: [
      {
        url: "/opengraph-image",
        width: 1200,
        height: 630,
        alt: "Qryvanta.org — Open-Source Business Platform",
        type: "image/png",
      },
    ],
  },

  twitter: {
    card: "summary_large_image",
    title: "Qryvanta.org — Open-Source Business Platform",
    description:
      "Open-source, self-hostable business platform for metadata-driven operations.",
    images: ["/opengraph-image"],
  },

  alternates: {
    canonical: siteUrl,
  },
};

const jsonLd = {
  "@context": "https://schema.org",
  "@graph": [
    {
      "@type": "WebSite",
      "@id": `${siteUrl}/#website`,
      name: "Qryvanta.org",
      url: siteUrl,
      description:
        "Open-source project hub for the Qryvanta platform. Read architecture notes, follow delivery progress, and find open issues to work on.",
      inLanguage: "en-US",
    },
    {
      "@type": "SoftwareSourceCode",
      "@id": `${siteUrl}/#project`,
      name: "Qryvanta",
      url: siteUrl,
      codeRepository: "https://github.com/neelbiehler/qryvanta",
      programmingLanguage: ["TypeScript", "Rust"],
      description:
        "Open-source, self-hostable platform for metadata-driven business operations. Define entities once — published metadata becomes runtime APIs and operational surfaces.",
      applicationCategory: "BusinessApplication",
      operatingSystem: "Any",
      isAccessibleForFree: true,
      creator: {
        "@type": "Organization",
        name: "Qryvanta",
        url: siteUrl,
      },
    },
  ],
};

export default function RootLayout({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en">
      <head>
        <script
          type="application/ld+json"
          // biome-ignore lint/security/noDangerouslySetInnerHtml: controlled static JSON-LD, no user input
          dangerouslySetInnerHTML={{ __html: JSON.stringify(jsonLd) }}
        />
      </head>
      <body
        className={`${headingFont.variable} ${bodyFont.variable} min-h-screen bg-landing text-slate-950 antialiased`}
      >
        {children}
      </body>
    </html>
  );
}
