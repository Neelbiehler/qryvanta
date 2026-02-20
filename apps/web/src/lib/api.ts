import type {
  AcceptInviteRequest,
  AuthLoginRequest,
  AuthLoginResponse,
  AuthMfaVerifyRequest,
  AuthRegisterRequest,
  AuditLogEntryResponse,
  EntityResponse,
  GenericMessageResponse,
  InviteRequest,
  RoleAssignmentResponse,
  RoleResponse,
  TenantRegistrationModeResponse,
  UpdateTenantRegistrationModeRequest,
  UserIdentityResponse,
} from "@qryvanta/api-types";

export const API_BASE_URL =
  process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://localhost:3001";

export type {
  AcceptInviteRequest,
  AuthLoginRequest,
  AuthLoginResponse,
  AuthMfaVerifyRequest,
  AuthRegisterRequest,
  AuditLogEntryResponse,
  EntityResponse,
  GenericMessageResponse,
  InviteRequest,
  RoleAssignmentResponse,
  RoleResponse,
  TenantRegistrationModeResponse,
  UpdateTenantRegistrationModeRequest,
  UserIdentityResponse,
};

function withDefaultHeaders(headers?: HeadersInit): Headers {
  const requestHeaders = new Headers(headers);
  if (!requestHeaders.has("Content-Type")) {
    requestHeaders.set("Content-Type", "application/json");
  }
  return requestHeaders;
}

export async function apiFetch(
  path: string,
  init: RequestInit = {},
): Promise<Response> {
  const response = await fetch(`${API_BASE_URL}${path}`, {
    ...init,
    cache: "no-store",
    credentials: "include",
    headers: withDefaultHeaders(init.headers),
  });

  if (response.status === 401 && typeof window !== "undefined") {
    window.location.href = "/login";
  }

  return response;
}

export async function apiServerFetch(
  path: string,
  cookieHeader: string,
  init: RequestInit = {},
): Promise<Response> {
  const headers = withDefaultHeaders(init.headers);
  if (cookieHeader) {
    headers.set("cookie", cookieHeader);
  }

  return fetch(`${API_BASE_URL}${path}`, {
    ...init,
    cache: "no-store",
    headers,
  });
}
