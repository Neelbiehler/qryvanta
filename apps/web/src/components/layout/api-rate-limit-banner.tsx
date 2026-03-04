"use client";

import { useEffect, useMemo, useState } from "react";
import { Notice } from "@qryvanta/ui";

import {
  API_RATE_LIMIT_EVENT,
  type ApiRateLimitEventDetail,
} from "@/lib/api-rate-limit";

type RateLimitNotice = {
  message: string;
  expiresAtMs: number;
};

const DEFAULT_RETRY_AFTER_SECONDS = 60;
const MAX_DISPLAY_SECONDS = 90;

export function ApiRateLimitBanner() {
  const [notice, setNotice] = useState<RateLimitNotice | null>(null);

  useEffect(() => {
    const onRateLimited = (event: Event) => {
      const customEvent = event as CustomEvent<ApiRateLimitEventDetail>;
      const retryAfterSeconds =
        customEvent.detail.retryAfterSeconds ?? DEFAULT_RETRY_AFTER_SECONDS;
      const boundedRetryAfter = Math.min(retryAfterSeconds, MAX_DISPLAY_SECONDS);
      const path = customEvent.detail.path;
      const suffix = path ? ` (${path})` : "";

      setNotice({
        message: `System is under load. Please retry in about ${String(boundedRetryAfter)}s${suffix}.`,
        expiresAtMs: Date.now() + boundedRetryAfter * 1000,
      });
    };

    window.addEventListener(API_RATE_LIMIT_EVENT, onRateLimited as EventListener);
    return () => {
      window.removeEventListener(
        API_RATE_LIMIT_EVENT,
        onRateLimited as EventListener,
      );
    };
  }, []);

  useEffect(() => {
    if (!notice) {
      return;
    }

    const remainingMs = Math.max(0, notice.expiresAtMs - Date.now());
    const timeout = window.setTimeout(() => {
      setNotice((current) =>
        current && current.expiresAtMs <= Date.now() ? null : current,
      );
    }, remainingMs);

    return () => {
      window.clearTimeout(timeout);
    };
  }, [notice]);

  const renderedMessage = useMemo(() => notice?.message ?? null, [notice]);

  if (!renderedMessage) {
    return null;
  }

  return (
    <div className="px-4 pt-4 md:px-8">
      <Notice tone="warning">{renderedMessage}</Notice>
    </div>
  );
}
