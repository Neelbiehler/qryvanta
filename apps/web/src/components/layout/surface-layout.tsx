"use client";

import { useState, useEffect } from "react";

import { Header } from "@/components/layout/header";
import { SurfaceSidebar } from "@/components/layout/surface-sidebar";
import { AccessDeniedCard } from "@/components/shared/access-denied-card";
import { apiFetch, type UserIdentityResponse } from "@/lib/api";
import {
  type SurfaceId,
  SURFACES,
  hasSurfaceAccess,
  readAccessibleSurfaces,
} from "@/lib/surfaces";
import { cn } from "@/lib/utils";

type SurfaceLayoutProps = {
  children: React.ReactNode;
  surfaceId: SurfaceId;
  user: UserIdentityResponse;
};

/**
 * Shared layout for surface-scoped route groups.
 *
 * Renders the surface-specific sidebar, header, and content area.
 * Note: Auth check should be done at the page level before rendering this layout.
 */
export function SurfaceLayout({
  children,
  surfaceId,
  user,
}: SurfaceLayoutProps) {
  const accessibleSurfaces = readAccessibleSurfaces(user);
  const definition = SURFACES[surfaceId];

  // Always start expanded for SSR consistency, then sync with localStorage
  const [collapsed, setCollapsed] = useState(false);
  const [mounted, setMounted] = useState(false);

  // Sync with localStorage after mount (client-side only)
  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- Required for hydration handling
    setMounted(true);
    const saved = localStorage.getItem("sidebar-collapsed");
    if (saved === "true") {
      setCollapsed(true);
    }
  }, []);

  // Save collapsed state to localStorage when it changes
  useEffect(() => {
    if (mounted) {
      localStorage.setItem("sidebar-collapsed", String(collapsed));
    }
  }, [collapsed, mounted]);

  if (!hasSurfaceAccess(accessibleSurfaces, surfaceId)) {
    return (
      <div className="mx-auto flex min-h-screen w-full max-w-3xl items-center p-6">
        <AccessDeniedCard
          section={definition.label}
          title="Surface Access Denied"
          message={`Your account does not have the required permissions to access the ${definition.label}. Contact your tenant administrator to request access.`}
        />
      </div>
    );
  }

  return (
    <div 
      className={cn(
        "grid min-h-screen grid-cols-1 bg-app transition-all duration-300 ease-in-out",
        collapsed ? "lg:grid-cols-[64px_1fr]" : "lg:grid-cols-[260px_1fr]"
      )}
      suppressHydrationWarning
    >
      <SurfaceSidebar
        surface={surfaceId}
        accessibleSurfaces={accessibleSurfaces}
        collapsed={collapsed}
        onToggle={() => setCollapsed(!collapsed)}
      />
      <div className="flex min-h-screen min-w-0 flex-col">
        <Header user={user} surfaceId={surfaceId} />
        <main className="flex-1 px-4 py-5 md:px-8 md:py-8">{children}</main>
      </div>
    </div>
  );
}
