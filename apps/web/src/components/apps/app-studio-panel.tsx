"use client";

import { useEffect, useMemo, useState } from "react";
import { Notice, StatusBadge } from "@qryvanta/ui";

import { AppStudioMainContent } from "@/components/apps/app-studio/app-studio-main-content";
import { SolutionExplorerSidebar } from "@/components/apps/app-studio/solution-explorer-sidebar";
import { useAppStudioPanel } from "@/components/apps/app-studio/use-app-studio-panel";
import type { AppResponse, EntityResponse, RoleResponse } from "@/lib/api";

type AppStudioPanelProps = {
  apps: AppResponse[];
  entities: EntityResponse[];
  roles: RoleResponse[];
};

export function AppStudioPanel({ apps, entities, roles }: AppStudioPanelProps) {
  const panel = useAppStudioPanel({ apps, entities, roles });
  const [isShortcutHelpOpen, setIsShortcutHelpOpen] = useState(false);

  const workspaceSelectionKey = useMemo(
    () => buildWorkspaceSelectionKey(panel.workspacePublishDraft),
    [panel.workspacePublishDraft],
  );

  useEffect(() => {
    function onKeyDown(event: globalThis.KeyboardEvent) {
      if (!event.metaKey && !event.ctrlKey && !event.altKey && event.key === "?") {
        if (isEditableTarget(event.target)) {
          return;
        }
        event.preventDefault();
        setIsShortcutHelpOpen((current) => !current);
        return;
      }

      if (event.key === "Escape") {
        setIsShortcutHelpOpen(false);
      }
    }

    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
    };
  }, []);

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
        {panel.workspaceCheckSummary ? (
          <StatusBadge tone="neutral">
            Checked {panel.workspaceCheckSummary.checkedEntities} entities / {panel.workspaceCheckSummary.checkedApps} apps
          </StatusBadge>
        ) : null}
      </div>

      <div className="grid gap-4 xl:grid-cols-[260px_1fr]">
        <SolutionExplorerSidebar
          apps={apps}
          entities={entities}
          roles={roles}
          panel={panel}
          workspaceSelectionKey={workspaceSelectionKey}
          onToggleShortcuts={() => setIsShortcutHelpOpen((current) => !current)}
        />

        <AppStudioMainContent
          apps={apps}
          entities={entities}
          roles={roles}
          panel={panel}
          isShortcutHelpOpen={isShortcutHelpOpen}
        />
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

function buildWorkspaceSelectionKey(selection: {
  entityLogicalNames: string[];
  appLogicalNames: string[];
}): string {
  return JSON.stringify({
    entityLogicalNames: [...selection.entityLogicalNames].sort(),
    appLogicalNames: [...selection.appLogicalNames].sort(),
  });
}

function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) {
    return false;
  }

  const tagName = target.tagName;
  return (
    tagName === "INPUT" ||
    tagName === "TEXTAREA" ||
    tagName === "SELECT" ||
    target.isContentEditable
  );
}
