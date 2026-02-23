"use client";

import Link from "next/link";
import { Button, Notice, StatusBadge } from "@qryvanta/ui";

import {
  AppCatalogSection,
  NavigationBindingSection,
  RolePermissionsSection,
} from "@/components/apps/app-studio/sections";
import { useAppStudioPanel } from "@/components/apps/app-studio/use-app-studio-panel";
import type {
  AppResponse,
  EntityResponse,
  RoleResponse,
} from "@/lib/api";
import { cn } from "@/lib/utils";

type AppStudioPanelProps = {
  apps: AppResponse[];
  entities: EntityResponse[];
  roles: RoleResponse[];
};

export function AppStudioPanel({ apps, entities, roles }: AppStudioPanelProps) {
  const panel = useAppStudioPanel({ apps, entities, roles });

  return (
    <div className="space-y-4">
      {!panel.hasStudioData ? (
        <p className="rounded-md border border-amber-200 bg-amber-50 px-3 py-2 text-sm text-amber-800">
          Create at least one app, one entity, and one role before configuring model-driven workspace access.
        </p>
      ) : null}

      <div className="flex flex-wrap items-center gap-2 rounded-md border border-zinc-200 bg-zinc-50 px-3 py-2">
        <StatusBadge tone="neutral">Apps {apps.length}</StatusBadge>
        <StatusBadge tone="neutral">Entities {entities.length}</StatusBadge>
        <StatusBadge tone="neutral">Roles {roles.length}</StatusBadge>
        <StatusBadge tone="success">Active {panel.selectedAppDisplayName}</StatusBadge>
      </div>

      <div className="grid gap-4 xl:grid-cols-[260px_1fr]">
        <aside className="rounded-lg border border-zinc-200 bg-zinc-50 p-3">
          <p className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500">
            Solution Explorer
          </p>
          <p className="mt-1 text-xs text-zinc-600">
            Move through app shell, sitemap navigation, and role matrix.
          </p>

          <div className="mt-3 space-y-2">
            <Button
              type="button"
              className="w-full justify-start"
              variant={panel.activeSection === "apps" ? "default" : "outline"}
              onClick={() => panel.setActiveSection("apps")}
            >
              App Catalog
            </Button>
            <Button
              type="button"
              className="w-full justify-start"
              variant={panel.activeSection === "navigation" ? "default" : "outline"}
              onClick={() => panel.setActiveSection("navigation")}
              disabled={apps.length === 0 || entities.length === 0}
            >
              Sitemap Navigation
            </Button>
            <Button
              type="button"
              className="w-full justify-start"
              variant={panel.activeSection === "permissions" ? "default" : "outline"}
              onClick={() => panel.setActiveSection("permissions")}
              disabled={apps.length === 0 || entities.length === 0 || roles.length === 0}
            >
              Role Matrix
            </Button>
          </div>

          <div className="mt-4 space-y-2 border-t border-zinc-200 pt-3 text-xs text-zinc-600">
            <p>1. Create app shell</p>
            <p>2. Design sitemap tree</p>
            <p>3. Configure role permissions</p>
            <p>4. Validate in Worker Apps</p>
          </div>

          {panel.selectedApp ? (
            <Link
              href={`/maker/apps/${encodeURIComponent(panel.selectedApp)}/sitemap`}
              className={cn(
                "mt-3 inline-flex w-full items-center justify-center rounded-md border border-zinc-200 px-3 py-2 text-xs font-medium text-zinc-700 transition hover:bg-zinc-100",
              )}
            >
              Open Full Sitemap Editor
            </Link>
          ) : null}
        </aside>

        <section className="space-y-4 rounded-lg border border-zinc-200 bg-white p-4">
          {panel.activeSection === "apps" ? (
            <AppCatalogSection
              apps={apps}
              isCreatingApp={panel.pendingState.isCreatingApp}
              newAppDraft={panel.newAppDraft}
              onCreateApp={panel.handleCreateApp}
              onUpdateDraft={panel.setNewAppDraft}
            />
          ) : null}

          {panel.activeSection === "navigation" ? (
            <NavigationBindingSection
              apps={apps}
              bindings={panel.bindings}
              entities={entities}
              selectedEntityFields={panel.selectedEntityFields}
              isLoadingEntityFields={panel.isLoadingEntityFields}
              isBindingEntity={panel.pendingState.isBindingEntity}
              isLoadingAppData={panel.pendingState.isLoadingAppData}
              onBindEntity={panel.handleBindEntity}
              onChangeSelectedApp={panel.setSelectedApp}
              onUpdateBindingDraft={panel.setBindingDraft}
              selectedApp={panel.selectedApp}
              selectedAppDisplayName={panel.selectedAppDisplayName}
              bindingDraft={panel.bindingDraft}
            />
          ) : null}

          {panel.activeSection === "permissions" ? (
            <RolePermissionsSection
              apps={apps}
              entities={entities}
              isLoadingAppData={panel.pendingState.isLoadingAppData}
              isSavingPermission={panel.pendingState.isSavingPermission}
              onChangeSelectedApp={panel.setSelectedApp}
              onSavePermission={panel.handleSavePermission}
              onUpdatePermissionDraft={panel.setPermissionDraft}
              permissions={panel.permissions}
              roles={roles}
              selectedApp={panel.selectedApp}
              permissionDraft={panel.permissionDraft}
            />
          ) : null}
        </section>
      </div>

      {panel.messages.errorMessage ? (
        <Notice tone="error">{panel.messages.errorMessage}</Notice>
      ) : null}
      {panel.messages.statusMessage ? (
        <Notice tone="success">{panel.messages.statusMessage}</Notice>
      ) : null}
    </div>
  );
}
