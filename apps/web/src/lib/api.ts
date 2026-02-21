import type {
  AcceptInviteRequest,
  AppEntityBindingResponse,
  AppEntityCapabilitiesResponse,
  AppResponse,
  AppRoleEntityPermissionResponse,
  AuthLoginRequest,
  AuthLoginResponse,
  AuthMfaVerifyRequest,
  AuthRegisterRequest,
  AuditPurgeResultResponse,
  AuditRetentionPolicyResponse,
  AuditLogEntryResponse,
  BindAppEntityRequest,
  CreateAppRequest,
  CreateFieldRequest,
  CreateRuntimeRecordRequest,
  CreateTemporaryAccessGrantRequest,
  EntityResponse,
  FieldResponse,
  GenericMessageResponse,
  InviteRequest,
  PublishedSchemaResponse,
  QueryRuntimeRecordsRequest,
  RevokeTemporaryAccessGrantRequest,
  RoleAssignmentResponse,
  RoleResponse,
  RuntimeFieldPermissionResponse,
  RuntimeRecordResponse,
  SaveAppRoleEntityPermissionRequest,
  SaveRuntimeFieldPermissionsRequest,
  TemporaryAccessGrantResponse,
  TenantRegistrationModeResponse,
  UpdateAuditRetentionPolicyRequest,
  UpdateTenantRegistrationModeRequest,
  UserIdentityResponse,
} from "@qryvanta/api-types";

export const API_BASE_URL =
  process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://localhost:3001";

function resolveApiUrl(path: string): string {
  if (
    process.env.NODE_ENV === "production" &&
    API_BASE_URL.startsWith("http://")
  ) {
    throw new Error("NEXT_PUBLIC_API_BASE_URL must use HTTPS in production");
  }

  return `${API_BASE_URL}${path}`;
}

export type {
  AcceptInviteRequest,
  AppEntityBindingResponse,
  AppEntityCapabilitiesResponse,
  AppResponse,
  AppRoleEntityPermissionResponse,
  AuthLoginRequest,
  AuthLoginResponse,
  AuthMfaVerifyRequest,
  AuthRegisterRequest,
  AuditPurgeResultResponse,
  AuditRetentionPolicyResponse,
  AuditLogEntryResponse,
  BindAppEntityRequest,
  CreateAppRequest,
  CreateFieldRequest,
  CreateRuntimeRecordRequest,
  CreateTemporaryAccessGrantRequest,
  EntityResponse,
  FieldResponse,
  GenericMessageResponse,
  InviteRequest,
  PublishedSchemaResponse,
  QueryRuntimeRecordsRequest,
  RevokeTemporaryAccessGrantRequest,
  RoleAssignmentResponse,
  RoleResponse,
  RuntimeFieldPermissionResponse,
  RuntimeRecordResponse,
  SaveAppRoleEntityPermissionRequest,
  SaveRuntimeFieldPermissionsRequest,
  TemporaryAccessGrantResponse,
  TenantRegistrationModeResponse,
  UpdateAuditRetentionPolicyRequest,
  UpdateTenantRegistrationModeRequest,
  UserIdentityResponse,
};

function shouldSetJsonContentType(body: BodyInit | null | undefined): boolean {
  if (!body) {
    return false;
  }

  return typeof body === "string";
}

function withDefaultHeaders(
  headers?: HeadersInit,
  body?: BodyInit | null,
): Headers {
  const requestHeaders = new Headers(headers);
  if (!requestHeaders.has("Content-Type") && shouldSetJsonContentType(body)) {
    requestHeaders.set("Content-Type", "application/json");
  }
  return requestHeaders;
}

export async function apiFetch(
  path: string,
  init: RequestInit = {},
): Promise<Response> {
  const response = await fetch(resolveApiUrl(path), {
    ...init,
    cache: "no-store",
    credentials: "include",
    headers: withDefaultHeaders(init.headers, init.body),
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
  const headers = withDefaultHeaders(init.headers, init.body);
  if (cookieHeader) {
    headers.set("cookie", cookieHeader);
  }

  return fetch(resolveApiUrl(path), {
    ...init,
    cache: "no-store",
    headers,
  });
}
