import type { ErrorResponse } from "@/lib/api";

export const STEP_UP_REQUIRED_ERROR_CODE = "forbidden.step_up_required";

type ApiErrorPayload = Partial<ErrorResponse> & {
  code?: string;
  message?: string;
};

export async function readApiError(
  response: Response,
): Promise<ApiErrorPayload | null> {
  try {
    const contentType = response.headers.get("content-type") ?? "";
    if (!contentType.includes("application/json")) {
      return null;
    }

    return (await response.json()) as ApiErrorPayload;
  } catch {
    return null;
  }
}

export function apiErrorMessage(
  payload: ApiErrorPayload | null,
  fallback: string,
): string {
  return payload?.message?.trim() || fallback;
}

export function isStepUpRequiredError(
  payload: ApiErrorPayload | null,
): boolean {
  return payload?.code === STEP_UP_REQUIRED_ERROR_CODE;
}
