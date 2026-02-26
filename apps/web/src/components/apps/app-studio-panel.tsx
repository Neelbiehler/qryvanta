"use client";

import Link from "next/link";
import { useEffect, useMemo, useState } from "react";
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
  PublishCheckCategoryDto,
  PublishCheckIssueResponse,
  PublishCheckSeverityDto,
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
  const [isShortcutHelpOpen, setIsShortcutHelpOpen] = useState(false);
  const groupedWorkspaceIssues = useMemo(
    () => groupIssuesBySeverityAndCategory(panel.workspaceIssues),
    [panel.workspaceIssues],
  );
  const [selectedHistoryRunId, setSelectedHistoryRunId] = useState<
    string | null
  >(null);
  const selectedHistoryEntry = useMemo(
    () =>
      panel.publishHistory.find(
        (entry) => entry.runId === selectedHistoryRunId,
      ) ??
      panel.publishHistory[0] ??
      null,
    [panel.publishHistory, selectedHistoryRunId],
  );
  const workspaceSelectionKey = useMemo(
    () => buildWorkspaceSelectionKey(panel.workspacePublishDraft),
    [panel.workspacePublishDraft],
  );

  useEffect(() => {
    function onKeyDown(event: globalThis.KeyboardEvent) {
      if (
        !event.metaKey &&
        !event.ctrlKey &&
        !event.altKey &&
        event.key === "?"
      ) {
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

  useEffect(() => {
    if (panel.publishHistory.length === 0) {
      setSelectedHistoryRunId(null);
      return;
    }

    if (
      selectedHistoryRunId === null ||
      !panel.publishHistory.some(
        (entry) => entry.runId === selectedHistoryRunId,
      )
    ) {
      setSelectedHistoryRunId(panel.publishHistory[0]?.runId ?? null);
    }
  }, [panel.publishHistory, selectedHistoryRunId]);

  return (
    <div className="space-y-4">
      {!panel.hasStudioData ? (
        <p className="rounded-md border border-amber-200 bg-amber-50 px-3 py-2 text-sm text-amber-800">
          Create at least one app, one entity, and one role before configuring
          model-driven workspace access.
        </p>
      ) : null}

      <div className="flex flex-wrap items-center gap-2 rounded-md border border-zinc-200 bg-zinc-50 px-3 py-2">
        <StatusBadge tone="neutral">Apps {apps.length}</StatusBadge>
        <StatusBadge tone="neutral">Entities {entities.length}</StatusBadge>
        <StatusBadge tone="neutral">Roles {roles.length}</StatusBadge>
        <StatusBadge tone="success">
          Active {panel.selectedAppDisplayName}
        </StatusBadge>
        {panel.workspaceCheckSummary ? (
          <StatusBadge tone="neutral">
            Checked {panel.workspaceCheckSummary.checkedEntities} entities /{" "}
            {panel.workspaceCheckSummary.checkedApps} apps
          </StatusBadge>
        ) : null}
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
              variant={
                panel.activeSection === "navigation" ? "default" : "outline"
              }
              onClick={() => panel.setActiveSection("navigation")}
              disabled={apps.length === 0 || entities.length === 0}
            >
              Sitemap Navigation
            </Button>
            <Button
              type="button"
              className="w-full justify-start"
              variant={
                panel.activeSection === "permissions" ? "default" : "outline"
              }
              onClick={() => panel.setActiveSection("permissions")}
              disabled={
                apps.length === 0 || entities.length === 0 || roles.length === 0
              }
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
                Selected {panel.workspacePublishDraft.entityLogicalNames.length}{" "}
                entities / {panel.workspacePublishDraft.appLogicalNames.length}{" "}
                apps
              </p>
              <div className="space-y-1">
                <p className="text-[11px] font-semibold uppercase tracking-wide text-zinc-600">
                  Entities
                </p>
                {entities.map((entity) => {
                  const checked =
                    panel.workspacePublishDraft.entityLogicalNames.includes(
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
                                ...panel.workspacePublishDraft
                                  .entityLogicalNames,
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
                  const checked =
                    panel.workspacePublishDraft.appLogicalNames.includes(
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
                    panel.isRunningSelectionChecks ||
                    panel.isRunningSelectivePublish
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
                    panel.selectionValidation.selectionKey !==
                      workspaceSelectionKey ||
                    !panel.selectionValidation.isPublishable
                  }
                >
                  {panel.isRunningSelectivePublish
                    ? "Publishing..."
                    : "Publish Selection"}
                </Button>
              </div>
              {panel.selectionValidation.selectionKey ===
              workspaceSelectionKey ? (
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
            {panel.isRunningPublishChecks
              ? "Checking..."
              : "Run App Publish Checks"}
          </Button>

          <Button
            type="button"
            variant="outline"
            className="mt-2 w-full"
            onClick={() => setIsShortcutHelpOpen((current) => !current)}
            title="Toggle shortcuts (?)"
          >
            Shortcuts
          </Button>
        </aside>

        <section className="space-y-4 rounded-lg border border-zinc-200 bg-white p-4">
          {panel.publishCheckErrors.length > 0 ? (
            <Notice tone="error">
              <p className="font-semibold">App publish blockers</p>
              <ul className="mt-1 list-disc pl-5">
                {panel.publishCheckErrors.map((error) => (
                  <li key={error}>{error}</li>
                ))}
              </ul>
            </Notice>
          ) : null}

          {panel.workspaceIssues.length > 0 ? (
            <Notice tone="error">
              <p className="font-semibold">Workspace publish blockers</p>
              {groupedWorkspaceIssues.map((severityGroup) => (
                <div
                  key={`workspace-issues-severity-${severityGroup.severity}`}
                  className="mt-3 rounded border border-zinc-200 bg-white p-2"
                >
                  <p className="text-xs font-semibold uppercase tracking-wide">
                    {severityGroup.severity} ({severityGroup.total})
                  </p>
                  {severityGroup.categories.map(({ category, items }) => (
                    <div
                      key={`workspace-issues-${severityGroup.severity}-${category}`}
                      className="mt-2"
                    >
                      <p className="text-xs font-semibold uppercase tracking-wide text-zinc-600">
                        {category}
                      </p>
                      <ul className="mt-1 list-disc pl-5">
                        {items.map((issue, index) => (
                          <li
                            key={`${issue.scope}-${issue.scope_logical_name}-${String(index)}`}
                          >
                            <span className="font-medium">{issue.scope}:</span>{" "}
                            {issue.scope_logical_name} - {issue.message}{" "}
                            {issue.dependency_path ? (
                              <span className="text-xs text-zinc-600">
                                [{issue.dependency_path}]{" "}
                              </span>
                            ) : null}
                            {issue.fix_path ? (
                              <Link href={issue.fix_path} className="underline">
                                Open fix
                              </Link>
                            ) : null}
                          </li>
                        ))}
                      </ul>
                    </div>
                  ))}
                </div>
              ))}
            </Notice>
          ) : null}

          {panel.publishHistory.length > 0 ? (
            <div className="rounded-md border border-zinc-200 bg-zinc-50 p-3">
              <p className="text-xs font-semibold uppercase tracking-[0.14em] text-zinc-600">
                Publish History
              </p>
              <div className="mt-2 space-y-2">
                {panel.publishHistory.map((entry) => (
                  <button
                    key={entry.runId}
                    type="button"
                    className={cn(
                      "w-full rounded border px-2 py-2 text-left text-xs",
                      selectedHistoryEntry?.runId === entry.runId
                        ? "border-zinc-800 bg-white"
                        : "border-zinc-200 bg-white/70",
                    )}
                    onClick={() => setSelectedHistoryRunId(entry.runId)}
                  >
                    <div className="flex items-center justify-between gap-2">
                      <span className="font-semibold text-zinc-800">
                        {new Date(entry.runAt).toLocaleString()}
                      </span>
                      <StatusBadge
                        tone={entry.isPublishable ? "success" : "warning"}
                      >
                        {entry.isPublishable ? "publishable" : "blocked"}
                      </StatusBadge>
                    </div>
                    <p className="mt-1 text-zinc-600">
                      {entry.requestedEntities} requested entities /{" "}
                      {entry.requestedApps} requested apps - {entry.issueCount}{" "}
                      issues - by {entry.subject}
                    </p>
                  </button>
                ))}
              </div>

              {selectedHistoryEntry ? (
                <div className="mt-3 rounded-md border border-zinc-200 bg-white p-3 text-xs text-zinc-700">
                  <p className="font-semibold text-zinc-800">
                    Run {selectedHistoryEntry.runId}
                  </p>
                  <p className="mt-1">
                    Requested entities:{" "}
                    {selectedHistoryEntry.requestedEntityLogicalNames.join(
                      ", ",
                    ) || "none"}
                  </p>
                  <p className="mt-1">
                    Requested apps:{" "}
                    {selectedHistoryEntry.requestedAppLogicalNames.join(", ") ||
                      "none"}
                  </p>
                  <p className="mt-1">
                    Published entities:{" "}
                    {selectedHistoryEntry.publishedEntities.join(", ") ||
                      "none"}
                  </p>
                  <p className="mt-1">
                    Validated apps:{" "}
                    {selectedHistoryEntry.validatedApps.join(", ") || "none"}
                  </p>
                  <div className="mt-2 grid gap-2 md:grid-cols-2">
                    <Button
                      type="button"
                      variant="outline"
                      className="w-full"
                      onClick={() => {
                        panel.applyWorkspacePublishSelection(
                          selectedHistoryEntry.requestedEntityLogicalNames,
                          selectedHistoryEntry.requestedAppLogicalNames,
                        );
                      }}
                    >
                      Load Requested Selection
                    </Button>
                    <Button
                      type="button"
                      variant="outline"
                      className="w-full"
                      onClick={() => {
                        panel.applyWorkspacePublishSelection(
                          selectedHistoryEntry.publishedEntities,
                          selectedHistoryEntry.validatedApps,
                        );
                      }}
                      disabled={
                        selectedHistoryEntry.publishedEntities.length === 0 &&
                        selectedHistoryEntry.validatedApps.length === 0
                      }
                    >
                      Load Published Selection
                    </Button>
                  </div>
                </div>
              ) : null}
            </div>
          ) : null}

          <div className="rounded-md border border-zinc-200 bg-zinc-50 p-3">
            <p className="text-xs font-semibold uppercase tracking-[0.14em] text-zinc-600">
              Publish Diff Preview
            </p>
            <p className="mt-1 text-xs text-zinc-500">
              Field/form/view-level draft-to-published preview for selected
              entities and apps.
            </p>
            <div className="mt-3 space-y-3">
              {panel.publishDiff.unknownEntityLogicalNames.length > 0 ||
              panel.publishDiff.unknownAppLogicalNames.length > 0 ? (
                <Notice tone="warning">
                  Unknown selections: entities [
                  {panel.publishDiff.unknownEntityLogicalNames.join(", ") ||
                    "none"}
                  ], apps [
                  {panel.publishDiff.unknownAppLogicalNames.join(", ") ||
                    "none"}
                  ]
                </Notice>
              ) : null}

              <div className="rounded-md border border-zinc-200 bg-white p-2">
                <p className="text-[11px] font-semibold uppercase tracking-wide text-zinc-600">
                  Entity Diffs ({panel.publishDiff.entityDiffs.length})
                </p>
                <div className="mt-2 space-y-2">
                  {panel.publishDiff.entityDiffs.map((entityDiff) => (
                    <details
                      key={`entity-diff-${entityDiff.entity_logical_name}`}
                      className="rounded border border-zinc-200 p-2"
                      open
                    >
                      <summary className="cursor-pointer text-xs font-semibold text-zinc-800">
                        {entityDiff.entity_logical_name} -{" "}
                        {entityDiff.field_diff.length} field changes,{" "}
                        {entityDiff.forms.length} forms,{" "}
                        {entityDiff.views.length} views
                      </summary>
                      <div className="mt-2 grid gap-2 md:grid-cols-3 text-xs text-zinc-700">
                        <div>
                          <p className="font-semibold uppercase tracking-wide text-zinc-500">
                            Field Changes
                          </p>
                          <ul className="mt-1 space-y-1">
                            {entityDiff.field_diff.length > 0 ? (
                              entityDiff.field_diff.map((item) => (
                                <li
                                  key={`field-diff-${entityDiff.entity_logical_name}-${item.field_logical_name}`}
                                >
                                  {item.field_logical_name} [{item.change_type}]{" "}
                                  {item.published_field_type ?? "-"} {"->"}{" "}
                                  {item.draft_field_type ?? "-"}
                                </li>
                              ))
                            ) : (
                              <li>No field deltas</li>
                            )}
                          </ul>
                        </div>
                        <div>
                          <p className="font-semibold uppercase tracking-wide text-zinc-500">
                            Forms
                          </p>
                          <ul className="mt-1 space-y-1">
                            {entityDiff.forms.map((form) => (
                              <li
                                key={`entity-form-diff-${entityDiff.entity_logical_name}-${form.logical_name}`}
                              >
                                {form.logical_name} [{form.change_type}]{" "}
                                {form.published_item_count ?? 0} {"->"}{" "}
                                {form.draft_item_count ?? 0} fields
                                {form.published_is_default ||
                                form.draft_is_default ? (
                                  <span>
                                    {" "}
                                    [default{" "}
                                    {String(
                                      form.published_is_default ?? false,
                                    )}{" "}
                                    {"->"}{" "}
                                    {String(form.draft_is_default ?? false)}]
                                  </span>
                                ) : null}
                              </li>
                            ))}
                          </ul>
                        </div>
                        <div>
                          <p className="font-semibold uppercase tracking-wide text-zinc-500">
                            Views
                          </p>
                          <ul className="mt-1 space-y-1">
                            {entityDiff.views.map((view) => (
                              <li
                                key={`entity-view-diff-${entityDiff.entity_logical_name}-${view.logical_name}`}
                              >
                                {view.logical_name} [{view.change_type}]{" "}
                                {view.published_item_count ?? 0} {"->"}{" "}
                                {view.draft_item_count ?? 0} columns
                                {view.published_is_default ||
                                view.draft_is_default ? (
                                  <span>
                                    {" "}
                                    [default{" "}
                                    {String(
                                      view.published_is_default ?? false,
                                    )}{" "}
                                    {"->"}{" "}
                                    {String(view.draft_is_default ?? false)}]
                                  </span>
                                ) : null}
                              </li>
                            ))}
                          </ul>
                        </div>
                      </div>
                    </details>
                  ))}
                </div>
              </div>

              <div className="rounded-md border border-zinc-200 bg-white p-2">
                <p className="text-[11px] font-semibold uppercase tracking-wide text-zinc-600">
                  App Diffs ({panel.publishDiff.appDiffs.length})
                </p>
                <div className="mt-2 space-y-2">
                  {panel.publishDiff.appDiffs.map((appDiff) => (
                    <details
                      key={`app-diff-${appDiff.app_logical_name}`}
                      className="rounded border border-zinc-200 p-2"
                      open
                    >
                      <summary className="cursor-pointer text-xs font-semibold text-zinc-800">
                        {appDiff.app_logical_name} - {appDiff.bindings.length}{" "}
                        entity bindings
                      </summary>
                      <ul className="mt-2 space-y-1 text-xs text-zinc-700">
                        {appDiff.bindings.map((binding) => (
                          <li
                            key={`app-binding-diff-${appDiff.app_logical_name}-${binding.entity_logical_name}`}
                          >
                            {binding.entity_logical_name} / form:{" "}
                            {binding.default_form_logical_name} / view:{" "}
                            {binding.default_list_view_logical_name} / forms{" "}
                            {binding.forms.length} / views{" "}
                            {binding.views.length}
                          </li>
                        ))}
                      </ul>
                    </details>
                  ))}
                </div>
              </div>
            </div>
          </div>

          {isShortcutHelpOpen ? (
            <Notice tone="neutral">
              <p className="font-semibold">App Studio Shortcuts</p>
              <ul className="mt-1 list-disc pl-5 text-sm">
                <li>`?` toggle this help</li>
                <li>`Alt + Arrow` reorder focused form/view field chips</li>
                <li>`Escape` close this help</li>
                <li>
                  For sitemap canvas shortcuts, use the dedicated Sitemap Editor
                </li>
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

function groupIssuesBySeverityAndCategory(
  issues: PublishCheckIssueResponse[],
): Array<{
  severity: PublishCheckSeverityDto;
  total: number;
  categories: Array<{
    category: PublishCheckCategoryDto;
    items: PublishCheckIssueResponse[];
  }>;
}> {
  const severityGroups = new Map<
    PublishCheckSeverityDto,
    Map<PublishCheckCategoryDto, PublishCheckIssueResponse[]>
  >();

  for (const issue of issues) {
    const categoryMap =
      severityGroups.get(issue.severity) ??
      new Map<PublishCheckCategoryDto, PublishCheckIssueResponse[]>();
    const existing = categoryMap.get(issue.category) ?? [];
    existing.push(issue);
    categoryMap.set(issue.category, existing);
    severityGroups.set(issue.severity, categoryMap);
  }

  return Array.from(severityGroups.entries())
    .sort(
      ([leftSeverity], [rightSeverity]) =>
        severitySortWeight(leftSeverity) - severitySortWeight(rightSeverity),
    )
    .map(([severity, categories]) => ({
      severity,
      total: Array.from(categories.values()).reduce(
        (sum, bucket) => sum + bucket.length,
        0,
      ),
      categories: Array.from(categories.entries()).map(([category, items]) => ({
        category,
        items,
      })),
    }));
}

function severitySortWeight(severity: PublishCheckSeverityDto): number {
  switch (severity) {
    case "error":
      return 0;
    default:
      return 10;
  }
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
