"use client";

import { useEffect, useState } from "react";

import { Notice } from "@qryvanta/ui";

import { SitemapEditorPanel } from "@/components/apps/sitemap-editor-panel";
import { apiFetch, type AppSitemapResponse, type EntityResponse } from "@/lib/api";

type StudioSitemapCanvasProps = {
  appLogicalName: string;
  entities: EntityResponse[];
};

export function StudioSitemapCanvas({
  appLogicalName,
  entities,
}: StudioSitemapCanvasProps) {
  const [sitemap, setSitemap] = useState<AppSitemapResponse | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  useEffect(() => {
    let isMounted = true;

    async function loadSitemap(): Promise<void> {
      setIsLoading(true);
      setErrorMessage(null);

      try {
        const response = await apiFetch(
          `/api/apps/${encodeURIComponent(appLogicalName)}/sitemap`,
        );
        if (!isMounted) return;

        if (!response.ok) {
          const payload = (await response.json()) as { message?: string };
          setErrorMessage(payload.message ?? "Unable to load sitemap.");
          setSitemap(null);
          return;
        }

        setSitemap((await response.json()) as AppSitemapResponse);
      } catch {
        if (!isMounted) return;
        setErrorMessage("Unable to load sitemap.");
        setSitemap(null);
      } finally {
        if (isMounted) {
          setIsLoading(false);
        }
      }
    }

    void loadSitemap();

    return () => {
      isMounted = false;
    };
  }, [appLogicalName]);

  if (isLoading) {
    return (
      <div className="rounded-xl border border-zinc-200 bg-white p-4">
        <p className="text-sm text-zinc-500">Loading sitemap editor...</p>
      </div>
    );
  }

  if (!sitemap) {
    return (
      <div className="space-y-2 rounded-xl border border-zinc-200 bg-white p-4">
        <Notice tone="error">{errorMessage ?? "Unable to load sitemap."}</Notice>
      </div>
    );
  }

  return (
    <SitemapEditorPanel
      key={appLogicalName}
      appLogicalName={appLogicalName}
      initialSitemap={sitemap}
      entities={entities}
    />
  );
}
