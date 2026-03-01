import type { MetadataRoute } from "next";

const BASE = "https://qryvanta.org";

export default function sitemap(): MetadataRoute.Sitemap {
  return [
    {
      url: BASE,
      lastModified: new Date(),
      changeFrequency: "weekly",
      priority: 1,
    },
    { url: `${BASE}/impressum`, changeFrequency: "yearly", priority: 0.1 },
    { url: `${BASE}/datenschutz`, changeFrequency: "yearly", priority: 0.1 },
    { url: `${BASE}/legal-notice`, changeFrequency: "yearly", priority: 0.1 },
    { url: `${BASE}/privacy`, changeFrequency: "yearly", priority: 0.1 },
  ];
}
