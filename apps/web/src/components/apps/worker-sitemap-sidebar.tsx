"use client";

import Link from "next/link";
import { type DragEvent, type KeyboardEvent, useEffect, useMemo, useState } from "react";
import { ChevronDown, ChevronRight, Database, GripVertical, LayoutDashboard, Search, Star } from "lucide-react";

import { StatusBadge, buttonVariants } from "@qryvanta/ui";

import type { AppSitemapResponse } from "@/lib/api";
import { cn } from "@/lib/utils";

type WorkerSitemapSidebarProps = {
  appLogicalName: string;
  sitemap: AppSitemapResponse;
  activeEntityLogicalName?: string;
  activeDashboardLogicalName?: string;
};

type SidebarMenuItem = {
  key: string;
  href: string;
  label: string;
  meta: string;
  type: "entity" | "dashboard" | "custom";
  active: boolean;
};

export function WorkerSitemapSidebar({
  appLogicalName,
  sitemap,
  activeEntityLogicalName,
  activeDashboardLogicalName,
}: WorkerSitemapSidebarProps) {
  const areas = sitemap.areas.toSorted((left, right) => left.position - right.position);
  const collapseStorageKey = `worker_sitemap_groups_${appLogicalName}`;
  const favoritesStorageKey = `worker_sitemap_favorites_${appLogicalName}`;

  const [collapsedGroups, setCollapsedGroups] = useState<Record<string, boolean>>({});

  const [favoriteItemKeys, setFavoriteItemKeys] = useState<string[]>([]);
  const [draggedFavoriteKey, setDraggedFavoriteKey] = useState<string | null>(null);
  const [quickFind, setQuickFind] = useState("");

  useEffect(() => {
    try {
      const raw = localStorage.getItem(collapseStorageKey);
      if (!raw) return;
      const parsed = JSON.parse(raw) as Record<string, boolean>;
      queueMicrotask(() => {
        setCollapsedGroups(parsed);
      });
    } catch {
      // ignore storage errors
    }
  }, [collapseStorageKey]);

  useEffect(() => {
    try {
      const raw = localStorage.getItem(favoritesStorageKey);
      if (!raw) return;
      const parsed = JSON.parse(raw) as string[];
      if (!Array.isArray(parsed)) return;
      queueMicrotask(() => {
        setFavoriteItemKeys(parsed);
      });
    } catch {
      // ignore storage errors
    }
  }, [favoritesStorageKey]);

  useEffect(() => {
    try {
      localStorage.setItem(collapseStorageKey, JSON.stringify(collapsedGroups));
    } catch {
      // ignore storage errors
    }
  }, [collapsedGroups, collapseStorageKey]);

  useEffect(() => {
    try {
      localStorage.setItem(favoritesStorageKey, JSON.stringify(favoriteItemKeys));
    } catch {
      // ignore storage errors
    }
  }, [favoriteItemKeys, favoritesStorageKey]);

  const quickFindValue = quickFind.trim().toLowerCase();

  const allMenuItems = useMemo<SidebarMenuItem[]>(() => {
    const items: SidebarMenuItem[] = [];

    for (const area of areas) {
      const groups = area.groups.toSorted((left, right) => left.position - right.position);
      for (const group of groups) {
        const subAreas = group.sub_areas.toSorted((left, right) => left.position - right.position);
        for (const subArea of subAreas) {
          if (subArea.target.type === "entity") {
            const logicalName = subArea.target.entity_logical_name;
            items.push({
              key: `entity:${logicalName}`,
              href: `/worker/apps/${encodeURIComponent(appLogicalName)}/${encodeURIComponent(logicalName)}`,
              label: subArea.display_name,
              meta: logicalName,
              type: "entity",
              active: logicalName === activeEntityLogicalName,
            });
            continue;
          }

          if (subArea.target.type === "dashboard") {
            const logicalName = subArea.target.dashboard_logical_name;
            items.push({
              key: `dashboard:${logicalName}`,
              href: `/worker/apps/${encodeURIComponent(appLogicalName)}/dashboards/${encodeURIComponent(logicalName)}`,
              label: subArea.display_name,
              meta: logicalName,
              type: "dashboard",
              active: logicalName === activeDashboardLogicalName,
            });
            continue;
          }

          items.push({
            key: `custom:${subArea.logical_name}`,
            href: "#",
            label: subArea.display_name,
            meta: subArea.logical_name,
            type: "custom",
            active: false,
          });
        }
      }
    }

    return items;
  }, [activeDashboardLogicalName, activeEntityLogicalName, appLogicalName, areas]);

  const favoriteMenuItems = useMemo(
    () =>
      favoriteItemKeys
        .map((key) => allMenuItems.find((item) => item.key === key) ?? null)
        .filter((item): item is SidebarMenuItem => item !== null)
        .filter((item) =>
          quickFindValue.length === 0
            ? true
            : `${item.label} ${item.meta}`.toLowerCase().includes(quickFindValue),
        ),
    [allMenuItems, favoriteItemKeys, quickFindValue],
  );

  const navigableItems = useMemo(() => {
    const unique = new Map<string, SidebarMenuItem>();
    for (const item of [...favoriteMenuItems, ...allMenuItems]) {
      if (item.type === "custom") continue;
      if (
        quickFindValue.length > 0 &&
        !`${item.label} ${item.meta}`.toLowerCase().includes(quickFindValue)
      ) {
        continue;
      }
      unique.set(item.key, item);
    }
    return [...unique.values()];
  }, [allMenuItems, favoriteMenuItems, quickFindValue]);

  function toggleFavorite(itemKey: string) {
    setFavoriteItemKeys((current) =>
      current.includes(itemKey)
        ? current.filter((candidate) => candidate !== itemKey)
        : [...current, itemKey],
    );
  }

  function reorderFavoriteKeys(sourceKey: string, targetKey: string) {
    if (sourceKey === targetKey) return;
    setFavoriteItemKeys((current) => {
      const sourceIndex = current.indexOf(sourceKey);
      const targetIndex = current.indexOf(targetKey);
      if (sourceIndex === -1 || targetIndex === -1) return current;

      const next = [...current];
      next.splice(sourceIndex, 1);
      next.splice(targetIndex, 0, sourceKey);
      return next;
    });
  }

  function handleFavoriteDragStart(event: DragEvent<HTMLElement>, favoriteKey: string) {
    event.dataTransfer.effectAllowed = "move";
    setDraggedFavoriteKey(favoriteKey);
  }

  function handleFavoriteDrop(targetKey: string) {
    if (!draggedFavoriteKey) return;
    reorderFavoriteKeys(draggedFavoriteKey, targetKey);
    setDraggedFavoriteKey(null);
  }

  function handleKeyboardNavigation(event: KeyboardEvent<HTMLElement>) {
    if (!["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) return;

    const focusable = navigableItems
      .map((item) => document.getElementById(itemDomId(item.key)))
      .filter((element): element is HTMLElement => element instanceof HTMLElement);
    if (focusable.length === 0) return;

    event.preventDefault();

    const currentIndex = focusable.findIndex((element) => element === document.activeElement);
    if (event.key === "Home") {
      focusable[0]?.focus();
      return;
    }
    if (event.key === "End") {
      focusable[focusable.length - 1]?.focus();
      return;
    }

    const nextIndex =
      event.key === "ArrowDown"
        ? (currentIndex + 1 + focusable.length) % focusable.length
        : (currentIndex - 1 + focusable.length) % focusable.length;
    focusable[nextIndex]?.focus();
  }

  return (
    <aside
      className="h-full overflow-y-auto border-r border-emerald-100 bg-white"
      onKeyDown={handleKeyboardNavigation}
    >
      {/* Sidebar Header */}
      <div className="border-b border-emerald-100 bg-emerald-50 px-3 py-3">
        <p className="text-[10px] font-semibold uppercase tracking-[0.14em] text-emerald-700">
          App Navigation
        </p>
        <p className="mt-0.5 truncate text-sm font-semibold text-zinc-900">{appLogicalName}</p>
        <label className="mt-2 flex items-center gap-2 rounded-md border border-emerald-200 bg-white px-2 py-1.5 text-xs text-zinc-500 focus-within:border-emerald-400 focus-within:ring-1 focus-within:ring-emerald-200">
          <Search aria-hidden="true" className="h-3.5 w-3.5 shrink-0 text-zinc-400" />
          <input
            value={quickFind}
            onChange={(event) => setQuickFind(event.target.value)}
            placeholder="Quick find…"
            autoComplete="off"
            spellCheck={false}
            className="w-full border-none bg-transparent p-0 text-xs text-zinc-700 outline-none placeholder:text-zinc-400"
            aria-label="Quick find in sitemap"
          />
        </label>
      </div>

      <nav className="space-y-1 p-2" aria-label="App sitemap">
        {/* Pinned / Favorites */}
        {favoriteMenuItems.length > 0 ? (
          <div className="mb-2">
            <p className="px-2 py-1.5 text-[10px] font-semibold uppercase tracking-[0.14em] text-emerald-600">
              Pinned
            </p>
            <div className="space-y-0.5">
              {favoriteMenuItems.map((item) => (
                <MenuItemLink
                  key={`favorite-${item.key}`}
                  item={item}
                  isFavorite
                  onToggleFavorite={() => toggleFavorite(item.key)}
                  draggable
                  showDragHandle
                  isDragged={draggedFavoriteKey === item.key}
                  onDragStart={(event) => handleFavoriteDragStart(event, item.key)}
                  onDragOver={(event) => event.preventDefault()}
                  onDrop={() => handleFavoriteDrop(item.key)}
                  onDragEnd={() => setDraggedFavoriteKey(null)}
                />
              ))}
            </div>
            <div className="mx-2 my-2 h-px bg-emerald-100" />
          </div>
        ) : null}

        {/* Areas & Groups */}
        {areas.map((area) => (
          <div key={area.logical_name} className="mb-1">
            <p className="px-2 py-1.5 text-[10px] font-semibold uppercase tracking-[0.14em] text-emerald-600">
              {area.display_name}
            </p>

            {area.groups
              .toSorted((left, right) => left.position - right.position)
              .map((group) => {
                const visibleSubAreas = group.sub_areas
                  .toSorted((left, right) => left.position - right.position)
                  .filter((subArea) => {
                    if (quickFindValue.length === 0) return true;
                    if (subArea.target.type === "entity") {
                      return `${subArea.display_name} ${subArea.target.entity_logical_name}`
                        .toLowerCase()
                        .includes(quickFindValue);
                    }
                    if (subArea.target.type === "dashboard") {
                      return `${subArea.display_name} ${subArea.target.dashboard_logical_name}`
                        .toLowerCase()
                        .includes(quickFindValue);
                    }
                    return `${subArea.display_name} ${subArea.logical_name}`
                      .toLowerCase()
                      .includes(quickFindValue);
                  });

                if (visibleSubAreas.length === 0) {
                  return null;
                }

                return (
                  <div key={group.logical_name} className="mb-1">
                    <button
                      type="button"
                      aria-expanded={!collapsedGroups[group.logical_name]}
                      className="flex w-full items-center justify-between rounded-md px-2 py-1.5 text-left text-[11px] font-semibold text-zinc-600 hover:bg-emerald-50 hover:text-zinc-900"
                      onClick={() =>
                        setCollapsedGroups((current) => ({
                          ...current,
                          [group.logical_name]: !current[group.logical_name],
                        }))
                      }
                    >
                      <span>{group.display_name}</span>
                      {collapsedGroups[group.logical_name] ? (
                        <ChevronRight aria-hidden="true" className="h-3.5 w-3.5 text-zinc-400" />
                      ) : (
                        <ChevronDown aria-hidden="true" className="h-3.5 w-3.5 text-zinc-400" />
                      )}
                    </button>

                    {collapsedGroups[group.logical_name]
                      ? null
                      : (
                        <div className="space-y-0.5">
                          {visibleSubAreas.map((subArea) => {
                            if (subArea.target.type === "entity") {
                              const item: SidebarMenuItem = {
                                key: `entity:${subArea.target.entity_logical_name}`,
                                href: `/worker/apps/${encodeURIComponent(appLogicalName)}/${encodeURIComponent(subArea.target.entity_logical_name)}`,
                                label: subArea.display_name,
                                meta: subArea.target.entity_logical_name,
                                type: "entity",
                                active: subArea.target.entity_logical_name === activeEntityLogicalName,
                              };
                              return (
                                <MenuItemLink
                                  key={item.key}
                                  item={item}
                                  isFavorite={favoriteItemKeys.includes(item.key)}
                                  onToggleFavorite={() => toggleFavorite(item.key)}
                                />
                              );
                            }

                            if (subArea.target.type === "dashboard") {
                              const item: SidebarMenuItem = {
                                key: `dashboard:${subArea.target.dashboard_logical_name}`,
                                href: `/worker/apps/${encodeURIComponent(appLogicalName)}/dashboards/${encodeURIComponent(subArea.target.dashboard_logical_name)}`,
                                label: subArea.display_name,
                                meta: subArea.target.dashboard_logical_name,
                                type: "dashboard",
                                active:
                                  subArea.target.dashboard_logical_name === activeDashboardLogicalName,
                              };
                              return (
                                <MenuItemLink
                                  key={item.key}
                                  item={item}
                                  isFavorite={favoriteItemKeys.includes(item.key)}
                                  onToggleFavorite={() => toggleFavorite(item.key)}
                                />
                              );
                            }

                            const item: SidebarMenuItem = {
                              key: `custom:${subArea.logical_name}`,
                              href: "#",
                              label: subArea.display_name,
                              meta: subArea.logical_name,
                              type: "custom",
                              active: false,
                            };
                            return (
                              <MenuItemLink
                                key={item.key}
                                item={item}
                                isFavorite={favoriteItemKeys.includes(item.key)}
                                onToggleFavorite={() => toggleFavorite(item.key)}
                              />
                            );
                          })}
                        </div>
                      )}
                  </div>
                );
              })}
          </div>
        ))}

        {/* Footer Nav */}
        <div className="mx-1 mt-3 border-t border-emerald-100 pt-2">
          <div className="grid grid-cols-2 gap-1.5">
            <Link
              href={`/worker/apps/${encodeURIComponent(appLogicalName)}`}
              className={cn(
                buttonVariants({ variant: "outline", size: "sm" }),
                "w-full border-emerald-200 text-emerald-700 hover:bg-emerald-50",
              )}
            >
              App Home
            </Link>
            <Link
              href="/worker/apps"
              className={cn(
                buttonVariants({ variant: "outline", size: "sm" }),
                "w-full border-emerald-200 text-emerald-700 hover:bg-emerald-50",
              )}
            >
              My Apps
            </Link>
          </div>
        </div>
      </nav>
    </aside>
  );
}

type MenuItemLinkProps = {
  item: SidebarMenuItem;
  isFavorite: boolean;
  onToggleFavorite: () => void;
  draggable?: boolean;
  showDragHandle?: boolean;
  isDragged?: boolean;
  onDragStart?: (event: DragEvent<HTMLElement>) => void;
  onDragOver?: (event: DragEvent<HTMLElement>) => void;
  onDrop?: (event: DragEvent<HTMLElement>) => void;
  onDragEnd?: (event: DragEvent<HTMLElement>) => void;
};

function MenuItemLink({
  item,
  isFavorite,
  onToggleFavorite,
  draggable,
  showDragHandle,
  isDragged,
  onDragStart,
  onDragOver,
  onDrop,
  onDragEnd,
}: MenuItemLinkProps) {
  const id = itemDomId(item.key);
  const icon =
    item.type === "entity" ? (
      <Database aria-hidden="true" className="h-3.5 w-3.5" />
    ) : (
      <LayoutDashboard aria-hidden="true" className="h-3.5 w-3.5" />
    );

  const content = (
    <>
      <div className="flex min-w-0 items-center gap-2">
        <span
          className={cn(
            "inline-flex h-5 w-5 shrink-0 items-center justify-center rounded",
            item.active
              ? "bg-emerald-100 text-emerald-700"
              : "bg-zinc-100 text-zinc-500",
          )}
        >
          {icon}
        </span>
        <div className="min-w-0">
          <p
            className={cn(
              "truncate text-xs font-medium",
              item.active ? "text-emerald-900" : "text-zinc-700",
            )}
          >
            {item.label}
          </p>
          <p className="truncate font-mono text-[10px] text-zinc-400">{item.meta}</p>
        </div>
      </div>

      <div className="ml-2 flex shrink-0 items-center gap-1">
        {showDragHandle ? (
          <GripVertical aria-hidden="true" className="h-3.5 w-3.5 text-zinc-300" />
        ) : null}
        {item.active ? (
          <StatusBadge tone={item.type === "dashboard" ? "info" : "success"}>
            {item.type === "dashboard" ? "Dash" : "Active"}
          </StatusBadge>
        ) : null}
        <button
          type="button"
          className={cn(
            "rounded p-1 transition-colors",
            isFavorite
              ? "text-amber-500"
              : "text-zinc-300 hover:text-zinc-500",
          )}
          onClick={(event) => {
            event.preventDefault();
            event.stopPropagation();
            onToggleFavorite();
          }}
          aria-label={isFavorite ? "Remove favorite" : "Add favorite"}
        >
          <Star aria-hidden="true" className={cn("h-3.5 w-3.5", isFavorite ? "fill-current" : "")} />
        </button>
      </div>
    </>
  );

  const baseClass = cn(
    "flex items-center justify-between rounded-md px-2 py-1.5 text-xs transition-colors",
    item.active
      ? "border-l-2 border-emerald-600 bg-emerald-50 pl-[6px]"
      : "border-l-2 border-transparent pl-[6px] hover:bg-zinc-50",
    isDragged ? "opacity-60" : "",
  );

  if (item.type === "custom") {
    return (
      <div
        id={id}
        className={cn(baseClass, "cursor-default border-dashed")}
        tabIndex={0}
        draggable={draggable}
        onDragStart={onDragStart}
        onDragOver={onDragOver}
        onDrop={onDrop}
        onDragEnd={onDragEnd}
      >
        {content}
      </div>
    );
  }

  return (
    <Link
      id={id}
      href={item.href}
      className={baseClass}
      draggable={draggable}
      onDragStart={onDragStart}
      onDragOver={onDragOver}
      onDrop={onDrop}
      onDragEnd={onDragEnd}
    >
      {content}
    </Link>
  );
}

function itemDomId(itemKey: string): string {
  return `worker_menu_${itemKey.replace(/[^a-zA-Z0-9_-]/g, "_")}`;
}
