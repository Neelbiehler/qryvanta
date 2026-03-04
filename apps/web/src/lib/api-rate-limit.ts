export const API_RATE_LIMIT_EVENT = "qryvanta:api-rate-limited";

export type ApiRateLimitEventDetail = {
  path: string;
  retryAfterSeconds: number | null;
};

function parseRetryAfterSeconds(response: Response): number | null {
  const raw = response.headers.get("retry-after");
  if (!raw) {
    return null;
  }

  const parsed = Number.parseInt(raw, 10);
  if (!Number.isFinite(parsed) || parsed <= 0) {
    return null;
  }

  return parsed;
}

export function emitRateLimitEvent(path: string, response: Response): void {
  if (typeof window === "undefined") {
    return;
  }

  window.dispatchEvent(
    new CustomEvent<ApiRateLimitEventDetail>(API_RATE_LIMIT_EVENT, {
      detail: {
        path,
        retryAfterSeconds: parseRetryAfterSeconds(response),
      },
    }),
  );
}
