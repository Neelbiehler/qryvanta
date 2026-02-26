import { Notice } from "@qryvanta/ui";

import {
  AppCatalogSection,
  NavigationBindingSection,
  RolePermissionsSection,
} from "@/components/apps/app-studio/sections";
import { PublishDiffPanel } from "@/components/apps/app-studio/publish-diff-panel";
import { PublishHistoryPanel } from "@/components/apps/app-studio/publish-history-panel";
import { WorkspaceChecksPanel } from "@/components/apps/app-studio/workspace-checks-panel";
import type { AppStudioPanelController } from "@/components/apps/app-studio/use-app-studio-panel";
import type { AppResponse, EntityResponse, RoleResponse } from "@/lib/api";

type AppStudioMainContentProps = {
  apps: AppResponse[];
  entities: EntityResponse[];
  roles: RoleResponse[];
  panel: AppStudioPanelController;
  isShortcutHelpOpen: boolean;
};

export function AppStudioMainContent({
  apps,
  entities,
  roles,
  panel,
  isShortcutHelpOpen,
}: AppStudioMainContentProps) {
  return (
    <section className="space-y-4 rounded-lg border border-zinc-200 bg-white p-4">
      <WorkspaceChecksPanel
        publishCheckErrors={panel.publishCheckErrors}
        workspaceIssues={panel.workspaceIssues}
      />

      <PublishHistoryPanel
        publishHistory={panel.publishHistory}
        onLoadSelection={panel.applyWorkspacePublishSelection}
      />

      <PublishDiffPanel publishDiff={panel.publishDiff} />

      {isShortcutHelpOpen ? (
        <Notice tone="neutral">
          <p className="font-semibold">App Studio Shortcuts</p>
          <ul className="mt-1 list-disc pl-5 text-sm">
            <li>`?` toggle this help</li>
            <li>`Alt + Arrow` reorder focused form/view field chips</li>
            <li>`Escape` close this help</li>
            <li>For sitemap canvas shortcuts, use the dedicated Sitemap Editor</li>
          </ul>
        </Notice>
      ) : null}

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
          isReorderingBinding={panel.pendingState.isReorderingBinding}
          isLoadingAppData={panel.pendingState.isLoadingAppData}
          onBindEntity={panel.handleBindEntity}
          onReorderBinding={panel.handleReorderBinding}
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
  );
}
