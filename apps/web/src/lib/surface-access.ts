import { cookies } from "next/headers";
import { redirect } from "next/navigation";

import { apiServerFetch, type UserIdentityResponse } from "@/lib/api";
import {
  type SurfaceId,
  SURFACES,
  readAccessibleSurfaces,
  resolveDefaultSurface,
} from "@/lib/surfaces";

/**
 * Loads the current user and enforces access for a specific product surface.
 */
export async function requireSurfaceUser(
  surfaceId: SurfaceId,
): Promise<UserIdentityResponse> {
  const cookieHeader = (await cookies()).toString();
  const meResponse = await apiServerFetch("/auth/me", cookieHeader);

  if (meResponse.status === 401) {
    redirect("/login");
  }

  if (!meResponse.ok) {
    throw new Error("Failed to load user identity");
  }

  const user = (await meResponse.json()) as UserIdentityResponse;
  const accessibleSurfaces = readAccessibleSurfaces(user);

  if (accessibleSurfaces.includes(surfaceId)) {
    return user;
  }

  const defaultSurface = resolveDefaultSurface(accessibleSurfaces);
  if (defaultSurface) {
    redirect(SURFACES[defaultSurface].basePath);
  }

  redirect("/");
}
