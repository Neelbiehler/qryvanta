"use client";

import { type DragEvent, type KeyboardEvent, useEffect, useMemo, useState } from "react";
import { useRouter } from "next/navigation";

import {
  Button,
  Card,
  CardDescription,
  CardHeader,
  CardTitle,
  Notice,
  StatusBadge,
} from "@qryvanta/ui";

import {
  apiFetch,
  type AppSitemapResponse,
  type EntityResponse,
  type FormResponse,
  type SaveAppSitemapRequest,
  type ViewResponse,
} from "@/lib/api";
import {
  createCrmTemplateSitemap,
  createDefaultArea,
  createDefaultGroup,
  createDefaultSubArea,
  moveGroup,
  moveSubArea,
  normalizeSitemap,
  parseDragPayload,
  reorderList,
} from "@/components/apps/sitemap-editor/model";
import { SitemapPreviewCard } from "@/components/apps/sitemap-editor/sitemap-preview-card";
import { SitemapPropertiesCard } from "@/components/apps/sitemap-editor/sitemap-properties-card";
import { SitemapTreeCard } from "@/components/apps/sitemap-editor/sitemap-tree-card";
import type { DragPayload, SelectionState } from "@/components/apps/sitemap-editor/types";
import { isEditableTarget } from "@/components/apps/sitemap-editor/utils";

type SitemapEditorPanelProps = {
  appLogicalName: string;
  initialSitemap: AppSitemapResponse;
  entities: EntityResponse[];
};

export function SitemapEditorPanel(props: SitemapEditorPanelProps) {
  return useSitemapEditorPanelContent(props);
}

function useSitemapEditorPanelContent({
  appLogicalName,
  initialSitemap,
  entities,
}: SitemapEditorPanelProps) {
  const router = useRouter();
  const [sitemap, setSitemap] = useState<AppSitemapResponse>(() =>
    normalizeSitemap(initialSitemap),
  );
  const [selection, setSelection] = useState<SelectionState | null>(
    initialSitemap.areas.length > 0 ? { kind: "area", areaIndex: 0 } : null,
  );
  const [history, setHistory] = useState<AppSitemapResponse[]>([]);
  const [future, setFuture] = useState<AppSitemapResponse[]>([]);
  const [activeDropLineId, setActiveDropLineId] = useState<string | null>(null);
  const [dragLabel, setDragLabel] = useState<string | null>(null);
  const [isShortcutHelpOpen, setIsShortcutHelpOpen] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [targetMetadataState, setTargetMetadataState] = useState<{
    entityFormsByLogicalName: Record<string, FormResponse[]>;
    entityViewsByLogicalName: Record<string, ViewResponse[]>;
    isLoadingTargetMetadata: boolean;
  }>({
    entityFormsByLogicalName: {},
    entityViewsByLogicalName: {},
    isLoadingTargetMetadata: false,
  });

  const entityFormsByLogicalName = targetMetadataState.entityFormsByLogicalName;
  const entityViewsByLogicalName = targetMetadataState.entityViewsByLogicalName;
  const isLoadingTargetMetadata = targetMetadataState.isLoadingTargetMetadata;

  const selectedArea =
    selection?.kind === "area" ||
    selection?.kind === "group" ||
    selection?.kind === "sub_area"
      ? (sitemap.areas[selection.areaIndex] ?? null)
      : null;
  const selectedGroup =
    selection?.kind === "group" || selection?.kind === "sub_area"
      ? (selectedArea?.groups[selection.groupIndex] ?? null)
      : null;
  const selectedSubArea =
    selection?.kind === "sub_area"
      ? (selectedGroup?.sub_areas[selection.subAreaIndex] ?? null)
      : null;

  const selectedEntityLogicalName =
    selectedSubArea?.target.type === "entity"
      ? selectedSubArea.target.entity_logical_name
      : null;

  const selectedEntityForms = selectedEntityLogicalName
    ? (entityFormsByLogicalName[selectedEntityLogicalName] ?? [])
    : [];
  const selectedEntityViews = selectedEntityLogicalName
    ? (entityViewsByLogicalName[selectedEntityLogicalName] ?? [])
    : [];

  const nodeCounts = useMemo(() => {
    const groups = sitemap.areas.reduce(
      (count, area) => count + area.groups.length,
      0,
    );
    const subAreas = sitemap.areas.reduce(
      (count, area) =>
        count +
        area.groups.reduce(
          (groupCount, group) => groupCount + group.sub_areas.length,
          0,
        ),
      0,
    );
    return {
      areas: sitemap.areas.length,
      groups,
      subAreas,
    };
  }, [sitemap]);

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

  useEffect(() => {
    async function loadEntityTargetMetadata() {
      if (!selectedEntityLogicalName) {
        return;
      }
      if (
        entityFormsByLogicalName[selectedEntityLogicalName] &&
        entityViewsByLogicalName[selectedEntityLogicalName]
      ) {
        return;
      }

      setTargetMetadataState((current) => ({
        ...current,
        isLoadingTargetMetadata: true,
      }));

      let forms: FormResponse[] | null = null;
      let views: ViewResponse[] | null = null;
      try {
        const [formsResponse, viewsResponse] = await Promise.all([
          apiFetch(`/api/entities/${selectedEntityLogicalName}/forms`),
          apiFetch(`/api/entities/${selectedEntityLogicalName}/views`),
        ]);

        forms = formsResponse.ok ? ((await formsResponse.json()) as FormResponse[]) : null;
        views = viewsResponse.ok ? ((await viewsResponse.json()) as ViewResponse[]) : null;
      } catch {
        forms = null;
        views = null;
      }

      setTargetMetadataState((current) => ({
        entityFormsByLogicalName: forms
          ? {
              ...current.entityFormsByLogicalName,
              [selectedEntityLogicalName]: forms,
            }
          : current.entityFormsByLogicalName,
        entityViewsByLogicalName: views
          ? {
              ...current.entityViewsByLogicalName,
              [selectedEntityLogicalName]: views,
            }
          : current.entityViewsByLogicalName,
        isLoadingTargetMetadata: false,
      }));
    }

    void loadEntityTargetMetadata();
  }, [
    entityFormsByLogicalName,
    entityViewsByLogicalName,
    selectedEntityLogicalName,
  ]);

  function updateSitemap(
    mutator: (current: AppSitemapResponse) => AppSitemapResponse,
    options: { trackHistory?: boolean } = {},
  ): void {
    const trackHistory = options.trackHistory ?? true;
    setSitemap((current) => {
      const next = normalizeSitemap(mutator(current));
      if (trackHistory && JSON.stringify(next) !== JSON.stringify(current)) {
        setHistory((previous) => [...previous.slice(-49), current]);
        setFuture([]);
      }
      return next;
    });
  }

  function undo(): void {
    const previous = history.at(-1);
    if (!previous) {
      return;
    }

    setHistory((current) => current.slice(0, -1));
    setFuture((current) => [sitemap, ...current].slice(0, 50));
    setSitemap(previous);
  }

  function redo(): void {
    const next = future.at(0);
    if (!next) {
      return;
    }

    setFuture((current) => current.slice(1));
    setHistory((current) => [...current, sitemap].slice(-50));
    setSitemap(next);
  }

  function moveSelectionByOffset(offset: number): void {
    if (!selection) {
      return;
    }

    if (selection.kind === "area") {
      const targetIndex = selection.areaIndex + offset;
      if (targetIndex < 0 || targetIndex >= sitemap.areas.length) {
        return;
      }

      updateSitemap((current) => ({
        ...current,
        areas: reorderList(current.areas, selection.areaIndex, targetIndex),
      }));
      setSelection({ kind: "area", areaIndex: targetIndex });
      return;
    }

    if (selection.kind === "group") {
      const groups = sitemap.areas[selection.areaIndex]?.groups ?? [];
      const targetIndex = selection.groupIndex + offset;
      if (targetIndex < 0 || targetIndex >= groups.length) {
        return;
      }

      updateSitemap((current) => ({
        ...current,
        areas: moveGroup(
          current.areas,
          selection.areaIndex,
          selection.groupIndex,
          selection.areaIndex,
          targetIndex,
        ),
      }));
      setSelection({
        kind: "group",
        areaIndex: selection.areaIndex,
        groupIndex: targetIndex,
      });
      return;
    }

    const subAreas =
      sitemap.areas[selection.areaIndex]?.groups[selection.groupIndex]?.sub_areas ?? [];
    const targetIndex = selection.subAreaIndex + offset;
    if (targetIndex < 0 || targetIndex >= subAreas.length) {
      return;
    }

    updateSitemap((current) => ({
      ...current,
      areas: moveSubArea(
        current.areas,
        selection.areaIndex,
        selection.groupIndex,
        selection.subAreaIndex,
        selection.areaIndex,
        selection.groupIndex,
        targetIndex,
      ),
    }));
    setSelection({
      kind: "sub_area",
      areaIndex: selection.areaIndex,
      groupIndex: selection.groupIndex,
      subAreaIndex: targetIndex,
    });
  }

  function handleCanvasKeyDown(event: KeyboardEvent<HTMLDivElement>): void {
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "z") {
      event.preventDefault();
      if (event.shiftKey) {
        redo();
      } else {
        undo();
      }
      return;
    }

    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "y") {
      event.preventDefault();
      redo();
      return;
    }

    if (!event.altKey) {
      return;
    }

    if (event.key === "ArrowUp" || event.key === "ArrowLeft") {
      event.preventDefault();
      moveSelectionByOffset(-1);
      return;
    }

    if (event.key === "ArrowDown" || event.key === "ArrowRight") {
      event.preventDefault();
      moveSelectionByOffset(1);
    }
  }

  function addArea() {
    updateSitemap((current) => ({
      ...current,
      areas: [...current.areas, createDefaultArea(current.areas.length)],
    }));
    setSelection({ kind: "area", areaIndex: sitemap.areas.length });
  }

  function addGroupToSelectedArea() {
    if (!selectedArea || !selection) {
      return;
    }
    const areaIndex = selection.areaIndex;
    updateSitemap((current) => ({
      ...current,
      areas: current.areas.map((area, currentAreaIndex) => {
        if (currentAreaIndex !== areaIndex) {
          return area;
        }
        return {
          ...area,
          groups: [
            ...area.groups,
            createDefaultGroup(areaIndex, area.groups.length),
          ],
        };
      }),
    }));
    setSelection({
      kind: "group",
      areaIndex,
      groupIndex: selectedArea.groups.length,
    });
  }

  function addSubAreaToSelectedGroup() {
    if (!selectedGroup || !selection) {
      return;
    }
    const areaIndex = selection.areaIndex;
    const groupIndex =
      selection.kind === "group" || selection.kind === "sub_area"
        ? selection.groupIndex
        : 0;
    updateSitemap((current) => ({
      ...current,
      areas: current.areas.map((area, currentAreaIndex) => {
        if (currentAreaIndex !== areaIndex) {
          return area;
        }
        return {
          ...area,
          groups: area.groups.map((group, currentGroupIndex) => {
            if (currentGroupIndex !== groupIndex) {
              return group;
            }
            return {
              ...group,
              sub_areas: [
                ...group.sub_areas,
                createDefaultSubArea(
                  areaIndex,
                  groupIndex,
                  group.sub_areas.length,
                  entities[0]?.logical_name ?? null,
                ),
              ],
            };
          }),
        };
      }),
    }));
    setSelection({
      kind: "sub_area",
      areaIndex,
      groupIndex,
      subAreaIndex: selectedGroup.sub_areas.length,
    });
  }

  function addGroupToArea(areaIndex: number) {
    updateSitemap((current) => ({
      ...current,
      areas: current.areas.map((area, currentAreaIndex) => {
        if (currentAreaIndex !== areaIndex) {
          return area;
        }

        return {
          ...area,
          groups: [
            ...area.groups,
            createDefaultGroup(areaIndex, area.groups.length),
          ],
        };
      }),
    }));
    setSelection({
      kind: "group",
      areaIndex,
      groupIndex: (sitemap.areas[areaIndex]?.groups.length ?? 0),
    });
  }

  function addSubAreaToGroup(areaIndex: number, groupIndex: number) {
    updateSitemap((current) => ({
      ...current,
      areas: current.areas.map((area, currentAreaIndex) => {
        if (currentAreaIndex !== areaIndex) {
          return area;
        }

        return {
          ...area,
          groups: area.groups.map((group, currentGroupIndex) => {
            if (currentGroupIndex !== groupIndex) {
              return group;
            }

            return {
              ...group,
              sub_areas: [
                ...group.sub_areas,
                createDefaultSubArea(
                  areaIndex,
                  groupIndex,
                  group.sub_areas.length,
                  entities[0]?.logical_name ?? null,
                ),
              ],
            };
          }),
        };
      }),
    }));
    setSelection({
      kind: "sub_area",
      areaIndex,
      groupIndex,
      subAreaIndex:
        (sitemap.areas[areaIndex]?.groups[groupIndex]?.sub_areas.length ?? 0),
    });
  }

  function deleteSelection() {
    if (!selection) {
      return;
    }

    if (selection.kind === "area") {
      const nextAreaCount = sitemap.areas.length - 1;
      updateSitemap((current) => ({
        ...current,
        areas: current.areas.filter((_, index) => index !== selection.areaIndex),
      }));

      if (nextAreaCount <= 0) {
        setSelection(null);
        return;
      }

      setSelection({
        kind: "area",
        areaIndex: Math.max(0, selection.areaIndex - 1),
      });
      return;
    }

    if (selection.kind === "group") {
      const area = sitemap.areas[selection.areaIndex];
      const nextGroupCount = (area?.groups.length ?? 1) - 1;
      updateSitemap((current) => ({
        ...current,
        areas: current.areas.map((currentArea, areaIndex) => {
          if (areaIndex !== selection.areaIndex) {
            return currentArea;
          }

          return {
            ...currentArea,
            groups: currentArea.groups.filter(
              (_, groupIndex) => groupIndex !== selection.groupIndex,
            ),
          };
        }),
      }));

      if (nextGroupCount <= 0) {
        setSelection({ kind: "area", areaIndex: selection.areaIndex });
        return;
      }

      setSelection({
        kind: "group",
        areaIndex: selection.areaIndex,
        groupIndex: Math.max(0, selection.groupIndex - 1),
      });
      return;
    }

    const group =
      sitemap.areas[selection.areaIndex]?.groups[selection.groupIndex] ?? null;
    const nextSubAreaCount = (group?.sub_areas.length ?? 1) - 1;
    updateSitemap((current) => ({
      ...current,
      areas: current.areas.map((area, areaIndex) => {
        if (areaIndex !== selection.areaIndex) {
          return area;
        }

        return {
          ...area,
          groups: area.groups.map((currentGroup, groupIndex) => {
            if (groupIndex !== selection.groupIndex) {
              return currentGroup;
            }

            return {
              ...currentGroup,
              sub_areas: currentGroup.sub_areas.filter(
                (_, subAreaIndex) => subAreaIndex !== selection.subAreaIndex,
              ),
            };
          }),
        };
      }),
    }));

    if (nextSubAreaCount <= 0) {
      setSelection({
        kind: "group",
        areaIndex: selection.areaIndex,
        groupIndex: selection.groupIndex,
      });
      return;
    }

    setSelection({
      kind: "sub_area",
      areaIndex: selection.areaIndex,
      groupIndex: selection.groupIndex,
      subAreaIndex: Math.max(0, selection.subAreaIndex - 1),
    });
  }

  function onDragStart(
    payload: DragPayload,
    label: string,
    event: DragEvent<HTMLButtonElement>,
  ) {
    setDragLabel(label);
    event.dataTransfer.setData("text/sitemap-node", JSON.stringify(payload));
    event.dataTransfer.effectAllowed = "move";
  }

  function onDropNode(
    targetPayload: DragPayload,
    event: DragEvent<HTMLButtonElement>,
  ) {
    event.preventDefault();
    setDragLabel(null);
    const rawPayload = event.dataTransfer.getData("text/sitemap-node");
    if (!rawPayload) {
      return;
    }
    const sourcePayload = parseDragPayload(rawPayload);
    if (!sourcePayload || sourcePayload.kind !== targetPayload.kind) {
      return;
    }

    if (sourcePayload.kind === "area") {
      if (targetPayload.kind !== "area") {
        return;
      }
      updateSitemap((current) => ({
        ...current,
        areas: reorderList(
          current.areas,
          sourcePayload.areaIndex,
          targetPayload.areaIndex,
        ),
      }));
      return;
    }

    if (sourcePayload.kind === "group") {
      if (targetPayload.kind !== "group" && targetPayload.kind !== "area") {
        return;
      }

      const targetAreaIndex =
        targetPayload.kind === "group"
          ? targetPayload.areaIndex
          : targetPayload.areaIndex;
      const targetGroupIndex =
        targetPayload.kind === "group"
          ? targetPayload.groupIndex
          : (sitemap.areas[targetAreaIndex]?.groups.length ?? 0);

      updateSitemap((current) => ({
        ...current,
        areas: moveGroup(
          current.areas,
          sourcePayload.areaIndex,
          sourcePayload.groupIndex,
          targetAreaIndex,
          targetGroupIndex,
        ),
      }));

      const selectionTargetGroupIndex =
        sourcePayload.areaIndex === targetAreaIndex &&
        sourcePayload.groupIndex < targetGroupIndex
          ? targetGroupIndex - 1
          : targetGroupIndex;
      setSelection({
        kind: "group",
        areaIndex: targetAreaIndex,
        groupIndex: Math.max(0, selectionTargetGroupIndex),
      });
      return;
    }

    if (sourcePayload.kind === "sub_area") {
      if (targetPayload.kind !== "sub_area" && targetPayload.kind !== "group") {
        return;
      }

      const targetAreaIndex =
        targetPayload.kind === "sub_area"
          ? targetPayload.areaIndex
          : targetPayload.areaIndex;
      const targetGroupIndex =
        targetPayload.kind === "sub_area"
          ? targetPayload.groupIndex
          : targetPayload.groupIndex;
      const targetSubAreaIndex =
        targetPayload.kind === "sub_area"
          ? targetPayload.subAreaIndex
          : (sitemap.areas[targetAreaIndex]?.groups[targetGroupIndex]?.sub_areas
              .length ?? 0);

      updateSitemap((current) => ({
        ...current,
        areas: moveSubArea(
          current.areas,
          sourcePayload.areaIndex,
          sourcePayload.groupIndex,
          sourcePayload.subAreaIndex,
          targetAreaIndex,
          targetGroupIndex,
          targetSubAreaIndex,
        ),
      }));

      const selectionTargetSubAreaIndex =
        sourcePayload.areaIndex === targetAreaIndex &&
        sourcePayload.groupIndex === targetGroupIndex &&
        sourcePayload.subAreaIndex < targetSubAreaIndex
          ? targetSubAreaIndex - 1
          : targetSubAreaIndex;
      setSelection({
        kind: "sub_area",
        areaIndex: targetAreaIndex,
        groupIndex: targetGroupIndex,
        subAreaIndex: Math.max(0, selectionTargetSubAreaIndex),
      });
    }
  }

  function onDropLine(
    targetPayload: DragPayload,
    event: DragEvent<HTMLDivElement>,
  ) {
    event.preventDefault();
    setDragLabel(null);
    const rawPayload = event.dataTransfer.getData("text/sitemap-node");
    if (!rawPayload) {
      return;
    }
    const sourcePayload = parseDragPayload(rawPayload);
    if (!sourcePayload) {
      return;
    }

    const fakeTarget = {
      preventDefault: () => undefined,
      dataTransfer: event.dataTransfer,
    } as unknown as DragEvent<HTMLButtonElement>;
    event.dataTransfer.setData("text/sitemap-node", JSON.stringify(sourcePayload));
    onDropNode(targetPayload, fakeTarget);
  }

  async function handleSave() {
    setStatusMessage(null);
    setErrorMessage(null);
    setIsSaving(true);
    try {
      const payload: SaveAppSitemapRequest = {
        areas: sitemap.areas,
      };
      const response = await apiFetch(`/api/apps/${appLogicalName}/sitemap`, {
        method: "PUT",
        body: JSON.stringify(payload),
      });
      if (!response.ok) {
        const payload = (await response.json()) as { message?: string };
        setErrorMessage(payload.message ?? "Unable to save sitemap.");
        return;
      }
      const saved = (await response.json()) as AppSitemapResponse;
      setSitemap(normalizeSitemap(saved));
      setStatusMessage("Sitemap saved.");
      router.refresh();
    } catch {
      setErrorMessage("Unable to save sitemap.");
    } finally {
      setIsSaving(false);
    }
  }

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
          <div className="space-y-2">
            <CardTitle>Sitemap Editor</CardTitle>
            <CardDescription>
              Manage hierarchical navigation areas, groups, and sub areas for
              worker sidebar rendering.
            </CardDescription>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <StatusBadge tone="neutral">Areas {nodeCounts.areas}</StatusBadge>
            <StatusBadge tone="neutral">Groups {nodeCounts.groups}</StatusBadge>
            <StatusBadge tone="neutral">
              Sub Areas {nodeCounts.subAreas}
            </StatusBadge>
            <Button type="button" variant="outline" onClick={addArea}>
              Add Area
            </Button>
            <Button
              type="button"
              variant="outline"
              disabled={!selectedArea}
              onClick={addGroupToSelectedArea}
            >
              Add Group
            </Button>
            <Button
              type="button"
              variant="outline"
              disabled={!selectedGroup}
              onClick={addSubAreaToSelectedGroup}
            >
              Add Sub Area
            </Button>
            <Button
              type="button"
              variant="outline"
              disabled={!selection}
              onClick={deleteSelection}
            >
              Delete Selected
            </Button>
            <Button type="button" variant="outline" disabled={history.length === 0} onClick={undo}>
              Undo
            </Button>
            <Button type="button" variant="outline" disabled={future.length === 0} onClick={redo}>
              Redo
            </Button>
            <Button
              type="button"
              variant="outline"
              onClick={() => setIsShortcutHelpOpen((current) => !current)}
              title="Toggle shortcuts (?)"
            >
              Shortcuts
            </Button>
            <Button
              type="button"
              variant="outline"
              onClick={() => {
                updateSitemap(
                  () => createCrmTemplateSitemap(appLogicalName, entities),
                  { trackHistory: true },
                );
                setSelection({ kind: "area", areaIndex: 0 });
                setStatusMessage("Applied CRM template.");
              }}
              disabled={entities.length === 0}
            >
              Apply CRM Template
            </Button>
            <Button type="button" disabled={isSaving} onClick={handleSave}>
              {isSaving ? "Saving..." : "Save Sitemap"}
            </Button>
          </div>
        </CardHeader>
      </Card>

      <div className="grid gap-4 xl:grid-cols-[320px_1fr_340px]">
        <SitemapTreeCard
          sitemap={sitemap}
          selection={selection}
          activeDropLineId={activeDropLineId}
          onSetActiveDropLineId={setActiveDropLineId}
          onCanvasKeyDown={handleCanvasKeyDown}
          onSelectNode={setSelection}
          onDragStart={onDragStart}
          onDragEnd={() => setDragLabel(null)}
          onDropNode={onDropNode}
          onDropLine={onDropLine}
          onAddGroupToArea={addGroupToArea}
          onAddSubAreaToGroup={addSubAreaToGroup}
        />

        <SitemapPreviewCard sitemap={sitemap} onSelectNode={setSelection} />

        <SitemapPropertiesCard
          selection={selection}
          selectedArea={selectedArea}
          selectedGroup={selectedGroup}
          selectedSubArea={selectedSubArea}
          entities={entities}
          selectedEntityForms={selectedEntityForms}
          selectedEntityViews={selectedEntityViews}
          isLoadingTargetMetadata={isLoadingTargetMetadata}
          onUpdateSitemap={updateSitemap}
        />
      </div>

      {isShortcutHelpOpen ? (
        <Notice tone="neutral">
          <p className="font-semibold">Sitemap Editor Shortcuts</p>
          <ul className="mt-1 list-disc pl-5 text-sm">
            <li>`?` toggle this help</li>
            <li>`Ctrl/Cmd + Z` undo</li>
            <li>`Ctrl/Cmd + Y` redo</li>
            <li>`Ctrl/Cmd + Shift + Z` redo</li>
            <li>`Alt + Arrow` reorder selected area/group/sub area</li>
            <li>`Escape` close this help</li>
          </ul>
        </Notice>
      ) : null}
      {dragLabel ? (
        <Notice tone="neutral">Dragging `{dragLabel}` - drop on highlighted insertion line.</Notice>
      ) : null}
      {errorMessage ? <Notice tone="error">{errorMessage}</Notice> : null}
      {statusMessage ? <Notice tone="success">{statusMessage}</Notice> : null}
    </div>
  );
}
