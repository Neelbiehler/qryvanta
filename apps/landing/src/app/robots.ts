import type { MetadataRoute } from "next";

export default function robots(): MetadataRoute.Robots {
  return {
    rules: [
      {
        userAgent: "*",
        allow: "/",
      },
    ],
    sitemap: "https://qryvanta.org/sitemap.xml",
    host: "https://qryvanta.org",
  };
}
