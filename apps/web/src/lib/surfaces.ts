/**
 * UX surface definitions for Qryvanta.
 *
 * Each surface represents a distinct product area targeting a specific persona:
 * - Admin Center: tenant administrators (roles, audit, security)
 * - Maker Center: low-code builders (entities, fields, app studio)
 * - Worker Apps: end-user operations (published apps, runtime records)
 *
 * Access is resolved from the `accessible_surfaces` field returned by
 * `/auth/me` which the API resolves from RBAC permissions.
 */

import type { UserIdentityResponse } from "@/lib/api";

export type SurfaceId = "admin" | "maker" | "worker";

export type NavigationItem = {
  label: string;
  href: string;
};

export type SurfaceDefinition = {
  id: SurfaceId;
  label: string;
  description: string;
  basePath: string;
  navigationItems: NavigationItem[];
};

export const SURFACES: Record<SurfaceId, SurfaceDefinition> = {
  admin: {
    id: "admin",
    label: "Admin Center",
    description: "Tenant administration, security, and compliance",
    basePath: "/admin",
    navigationItems: [
      { label: "Overview", href: "/admin" },
      { label: "Roles", href: "/admin/roles" },
      { label: "Audit Log", href: "/admin/audit" },
      { label: "Security Settings", href: "/admin/account" },
    ],
  },
  maker: {
    id: "maker",
    label: "Maker Center",
    description: "Entity modeling, field configuration, and app studio",
    basePath: "/maker",
    navigationItems: [
      { label: "Overview", href: "/maker" },
      { label: "Entities", href: "/maker/entities" },
      { label: "App Studio", href: "/maker/apps" },
      { label: "Automation", href: "/maker/automation" },
    ],
  },
  worker: {
    id: "worker",
    label: "Worker Apps",
    description: "Operational apps and runtime record management",
    basePath: "/worker",
    navigationItems: [
      { label: "Overview", href: "/worker" },
      { label: "My Apps", href: "/worker/apps" },
    ],
  },
};

/** Ordered list of surfaces for display purposes. */
export const SURFACE_ORDER: SurfaceId[] = ["worker", "maker", "admin"];

/**
 * Returns the first accessible surface for navigation after login,
 * preferring worker > maker > admin.
 */
export function resolveDefaultSurface(
  accessibleSurfaces: string[],
): SurfaceId | null {
  for (const id of SURFACE_ORDER) {
    if (accessibleSurfaces.includes(id)) {
      return id;
    }
  }
  return null;
}

/**
 * Checks whether a specific surface is in the accessible list.
 */
export function hasSurfaceAccess(
  accessibleSurfaces: string[],
  surface: SurfaceId,
): boolean {
  return accessibleSurfaces.includes(surface);
}

/**
 * Reads accessible surfaces from `/auth/me` payload safely.
 *
 * The field is delivered by the API as `accessible_surfaces`. During
 * contract rollout windows this helper tolerates older generated TS types
 * that may not yet include the field.
 */
export function readAccessibleSurfaces(user: UserIdentityResponse): string[] {
  const candidate = (
    user as UserIdentityResponse & { accessible_surfaces?: unknown }
  ).accessible_surfaces;

  if (!Array.isArray(candidate)) {
    return [];
  }

  return candidate.filter(
    (value): value is string => typeof value === "string",
  );
}
