"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useState, useEffect } from "react";

import {
  ChevronLeft,
  ChevronRight,
  LayoutDashboard,
  Settings,
  Shield,
  Briefcase,
  Home,
  Users,
  FileText,
  Lock,
  Box,
  AppWindow,
  type LucideIcon,
} from "lucide-react";

import { Sidebar as SidebarContainer, StatusBadge } from "@qryvanta/ui";

import {
  type SurfaceDefinition,
  type SurfaceId,
  SURFACES,
  SURFACE_ORDER,
} from "@/lib/surfaces";

import { cn } from "@/lib/utils";

const surfaceIcons: Record<SurfaceId, LucideIcon> = {
  worker: Briefcase,
  maker: Settings,
  admin: Shield,
};

const navigationIcons: Record<string, LucideIcon> = {
  Overview: Home,
  Roles: Users,
  "Audit Log": FileText,
  "Security Settings": Lock,
  Entities: Box,
  "App Studio": AppWindow,
  "My Apps": AppWindow,
};

type SurfaceSidebarProps = {
  surface: SurfaceId;
  accessibleSurfaces: string[];
  collapsed: boolean;
  onToggle: () => void;
};

export function SurfaceSidebar({
  surface,
  accessibleSurfaces,
  collapsed,
  onToggle,
}: SurfaceSidebarProps) {
  const pathname = usePathname();
  const definition: SurfaceDefinition = SURFACES[surface];

  return (
    <SidebarContainer
      collapsed={collapsed}
      className="relative lg:sticky lg:top-0 lg:h-screen"
    >
      {/* Header */}
      <div
        className={cn(
          "flex items-center border-b border-emerald-100/60",
          collapsed ? "justify-center p-4" : "justify-between p-4",
        )}
      >
        {!collapsed && (
          <div className="flex items-center gap-2">
            <div className="flex h-7 w-7 shrink-0 items-center justify-center rounded-md bg-emerald-700">
              <span className="text-xs font-bold text-white">Q</span>
            </div>
            <span className="text-sm font-semibold text-zinc-800">
              Qryvanta
            </span>
          </div>
        )}
        {collapsed && (
          <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-emerald-700">
            <span className="text-sm font-bold text-white">Q</span>
          </div>
        )}
      </div>

      {/* Current Surface Info */}
      <div
        className={cn(
          "border-b border-emerald-100/60",
          collapsed ? "p-3" : "p-4",
        )}
      >
        {!collapsed ? (
          <div className="space-y-2">
            <div className="flex items-center gap-2">
              <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-emerald-100">
                {(() => {
                  const Icon = surfaceIcons[surface];
                  return <Icon className="h-4 w-4 text-emerald-700" />;
                })()}
              </div>
              <div className="flex-1 min-w-0">
                <p className="truncate text-sm font-semibold text-zinc-900">
                  {definition.label}
                </p>
                <p className="truncate text-[11px] text-zinc-500">
                  {definition.description}
                </p>
              </div>
            </div>
            <StatusBadge tone="success" className="text-[10px]">
              Active
            </StatusBadge>
          </div>
        ) : (
          <div className="flex justify-center">
            <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-emerald-100">
              {(() => {
                const Icon = surfaceIcons[surface];
                return <Icon className="h-4 w-4 text-emerald-700" />;
              })()}
            </div>
          </div>
        )}
      </div>

      {/* Main Navigation */}
      <nav
        className="flex-1 overflow-auto py-3"
        aria-label={`${definition.label} navigation`}
      >
        <ul className="space-y-1 px-2">
          {definition.navigationItems.map((item) => {
            const isExactMatch = pathname === item.href;
            // Only allow child matching for non-base-path items
            // (Overview is typically the basePath, so it only matches exact)
            const isChildMatch =
              pathname !== item.href &&
              pathname.startsWith(`${item.href}/`) &&
              item.href !== definition.basePath;
            const isActive = isExactMatch || isChildMatch;
            const ItemIcon = navigationIcons[item.label] || LayoutDashboard;

            return (
              <li key={item.href}>
                <Link
                  href={item.href}
                  className={cn(
                    "flex items-center rounded-md transition-all duration-200",
                    collapsed ? "justify-center px-2 py-2" : "gap-3 px-3 py-2",
                    isActive
                      ? "bg-emerald-100/80 font-medium text-emerald-900"
                      : "text-zinc-600 hover:bg-emerald-50/60 hover:text-zinc-900",
                  )}
                  title={collapsed ? item.label : undefined}
                >
                  {collapsed ? (
                    <ItemIcon
                      className={cn(
                        "h-4 w-4",
                        isActive ? "text-emerald-600" : "text-zinc-400",
                      )}
                    />
                  ) : (
                    <>
                      <span
                        className={cn(
                          "h-1.5 w-1.5 rounded-full",
                          isActive ? "bg-emerald-600" : "bg-zinc-300",
                        )}
                      />
                      <span className="truncate text-sm">{item.label}</span>
                    </>
                  )}
                </Link>
              </li>
            );
          })}
        </ul>
      </nav>

      {/* Surface Switcher */}
      <div
        className={cn(
          "border-t border-emerald-100/60",
          collapsed ? "p-2" : "p-4",
        )}
      >
        {!collapsed && (
          <p className="mb-2 text-[10px] font-semibold uppercase tracking-wider text-zinc-400">
            Switch Surface
          </p>
        )}
        <nav className="space-y-1">
          {SURFACE_ORDER.filter((id) => accessibleSurfaces.includes(id)).map(
            (id) => {
              const target = SURFACES[id];
              const isCurrent = id === surface;
              const Icon = surfaceIcons[id];

              return (
                <Link
                  key={id}
                  href={target.basePath}
                  className={cn(
                    "flex items-center gap-2 rounded-md px-2 py-1.5 text-xs transition-colors",
                    isCurrent
                      ? "bg-zinc-100 font-medium text-zinc-900"
                      : "text-zinc-500 hover:bg-zinc-50 hover:text-zinc-700",
                  )}
                >
                  <Icon
                    className={cn(
                      "h-3.5 w-3.5",
                      isCurrent ? "text-emerald-600" : "text-zinc-400",
                    )}
                  />
                  {!collapsed && <span>{target.label}</span>}
                </Link>
              );
            },
          )}
        </nav>
      </div>

      {/* Toggle Button at Bottom */}
      <div
        className={cn(
          "border-t border-emerald-100/60",
          collapsed ? "p-2" : "p-3",
        )}
      >
        <button
          onClick={onToggle}
          className={cn(
            "flex items-center rounded-md text-zinc-400 transition hover:bg-emerald-50 hover:text-zinc-600",
            collapsed
              ? "w-full justify-center p-2"
              : "w-full justify-center gap-2 p-2",
          )}
          title={collapsed ? "Expand sidebar" : "Collapse sidebar"}
        >
          {collapsed ? (
            <ChevronRight className="h-4 w-4" />
          ) : (
            <>
              <ChevronLeft className="h-4 w-4" />
              <span className="text-xs">Collapse</span>
            </>
          )}
        </button>
      </div>
    </SidebarContainer>
  );
}
