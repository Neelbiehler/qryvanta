import Link from "next/link";

import { Button } from "@qryvanta/ui";

import type { AppStudioPanelController } from "@/components/apps/app-studio/use-app-studio-panel";
import type { AppResponse, EntityResponse, RoleResponse } from "@/lib/api";
import { cn } from "@/lib/utils";

type SolutionExplorerSidebarProps = {
  apps: AppResponse[];
  entities: EntityResponse[];
  roles: RoleResponse[];
  panel: AppStudioPanelController;
  workspaceSelectionKey: string;
  onToggleShortcuts: () => void;
};

export function SolutionExplorerSidebar({
  apps,
  entities,
  roles,
  panel,
  workspaceSelectionKey,
  onToggleShortcuts,
}: SolutionExplorerSidebarProps) {
  return (
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

      <Button
        type="button"
        variant="outline"
        className="mt-2 w-full"
        onClick={() => void panel.handleRunWorkspaceChecks()}
        disabled={panel.isRunningWorkspaceChecks}
      >
        {panel.isRunningWorkspaceChecks
          ? "Checking workspace..."
          : "Run Full Workspace Checks"}
      </Button>

      <details className="mt-2 rounded-md border border-zinc-200 bg-white p-2">
        <summary className="cursor-pointer text-xs font-semibold uppercase tracking-wide text-zinc-600">
          Selective Publish
        </summary>
        <div className="mt-2 space-y-2">
          <p className="text-[11px] text-zinc-500">
            Choose entities and apps to publish in this run.
          </p>
          <p className="text-[11px] text-zinc-500">
            Selected {panel.workspacePublishDraft.entityLogicalNames.length} entities /{" "}
            {panel.workspacePublishDraft.appLogicalNames.length} apps
          </p>
          <div className="space-y-1">
            <p className="text-[11px] font-semibold uppercase tracking-wide text-zinc-600">
              Entities
            </p>
            {entities.map((entity) => {
              const checked = panel.workspacePublishDraft.entityLogicalNames.includes(
                entity.logical_name,
              );
              return (
                <label
                  key={`publish-entity-${entity.logical_name}`}
                  className="flex items-center gap-2 text-xs text-zinc-700"
                >
                  <input
                    type="checkbox"
                    checked={checked}
                    onChange={(event) => {
                      const next = event.target.checked
                        ? [
                            ...panel.workspacePublishDraft.entityLogicalNames,
                            entity.logical_name,
                          ]
                        : panel.workspacePublishDraft.entityLogicalNames.filter(
                            (value) => value !== entity.logical_name,
                          );
                      panel.setWorkspacePublishDraft({
                        ...panel.workspacePublishDraft,
                        entityLogicalNames: next,
                      });
                    }}
                  />
                  <span>
                    {entity.display_name} ({entity.logical_name})
                  </span>
                </label>
              );
            })}
          </div>
          <div className="space-y-1">
            <p className="text-[11px] font-semibold uppercase tracking-wide text-zinc-600">
              Apps
            </p>
            {apps.map((app) => {
              const checked = panel.workspacePublishDraft.appLogicalNames.includes(
                app.logical_name,
              );
              return (
                <label
                  key={`publish-app-${app.logical_name}`}
                  className="flex items-center gap-2 text-xs text-zinc-700"
                >
                  <input
                    type="checkbox"
                    checked={checked}
                    onChange={(event) => {
                      const next = event.target.checked
                        ? [
                            ...panel.workspacePublishDraft.appLogicalNames,
                            app.logical_name,
                          ]
                        : panel.workspacePublishDraft.appLogicalNames.filter(
                            (value) => value !== app.logical_name,
                          );
                      panel.setWorkspacePublishDraft({
                        ...panel.workspacePublishDraft,
                        appLogicalNames: next,
                      });
                    }}
                  />
                  <span>
                    {app.display_name} ({app.logical_name})
                  </span>
                </label>
              );
            })}
          </div>
          <div className="grid gap-2">
            <Button
              type="button"
              variant="outline"
              className="w-full"
              onClick={() => void panel.handleRunSelectionChecks()}
              disabled={
                panel.isRunningSelectionChecks || panel.isRunningSelectivePublish
              }
            >
              {panel.isRunningSelectionChecks
                ? "Validating selection..."
                : "Validate Selection"}
            </Button>
            <Button
              type="button"
              className="w-full"
              onClick={() => void panel.handleRunSelectivePublish()}
              disabled={
                panel.isRunningSelectivePublish ||
                panel.isRunningSelectionChecks ||
                panel.selectionValidation.selectionKey !== workspaceSelectionKey ||
                !panel.selectionValidation.isPublishable
              }
            >
              {panel.isRunningSelectivePublish ? "Publishing..." : "Publish Selection"}
            </Button>
          </div>
          {panel.selectionValidation.selectionKey === workspaceSelectionKey ? (
            <p className="text-[11px] text-zinc-500">
              {panel.selectionValidation.isPublishable
                ? "Selection validated and ready to publish."
                : "Selection has unresolved publish issues."}
            </p>
          ) : (
            <p className="text-[11px] text-zinc-500">
              Run validation before publishing the current selection.
            </p>
          )}
        </div>
      </details>

      <Button
        type="button"
        variant="outline"
        className="mt-2 w-full"
        onClick={() => void panel.handleRunPublishChecks()}
        disabled={!panel.selectedApp || panel.isRunningPublishChecks}
      >
        {panel.isRunningPublishChecks ? "Checking..." : "Run App Publish Checks"}
      </Button>

      <Button
        type="button"
        variant="outline"
        className="mt-2 w-full"
        onClick={onToggleShortcuts}
        title="Toggle shortcuts (?)"
      >
        Shortcuts
      </Button>
    </aside>
  );
}
