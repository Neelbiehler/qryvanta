"use client";

import { Notice } from "@qryvanta/ui";

import {
  AppCatalogSection,
  AppStudioOverview,
  NavigationBindingSection,
  RolePermissionsSection,
} from "@/components/apps/app-studio/sections";
import { useAppStudioPanel } from "@/components/apps/app-studio/use-app-studio-panel";
import type {
  AppResponse,
  EntityResponse,
  RoleResponse,
} from "@/lib/api";

type AppStudioPanelProps = {
  apps: AppResponse[];
  entities: EntityResponse[];
  roles: RoleResponse[];
};

export function AppStudioPanel({ apps, entities, roles }: AppStudioPanelProps) {
  const panel = useAppStudioPanel({ apps, entities, roles });

  return (
    <div className="space-y-6">
      <AppStudioOverview
        activeSection={panel.activeSection}
        appsCount={apps.length}
        canOpenNavigation={apps.length > 0 && entities.length > 0}
        canOpenPermissions={
          apps.length > 0 && entities.length > 0 && roles.length > 0
        }
        entitiesCount={entities.length}
        hasStudioData={panel.hasStudioData}
        onSectionChange={panel.setActiveSection}
        rolesCount={roles.length}
        selectedAppDisplayName={panel.selectedAppDisplayName}
      />

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

      {panel.messages.errorMessage ? (
        <Notice tone="error">{panel.messages.errorMessage}</Notice>
      ) : null}
      {panel.messages.statusMessage ? (
        <Notice tone="success">{panel.messages.statusMessage}</Notice>
      ) : null}
    </div>
  );
}
