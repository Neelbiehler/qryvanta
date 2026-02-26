"use client";

import { type DragEvent, type KeyboardEvent, useEffect, useMemo, useState } from "react";
import { useRouter } from "next/navigation";

import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Input,
  Label,
  Notice,
  Select,
  StatusBadge,
} from "@qryvanta/ui";

import {
  apiFetch,
  type AppSitemapAreaDto,
  type AppSitemapResponse,
  type AppSitemapSubAreaDto,
  type AppSitemapTargetDto,
  type EntityResponse,
  type FormResponse,
  type SaveAppSitemapRequest,
  type ViewResponse,
} from "@/lib/api";

type SitemapEditorPanelProps = {
  appLogicalName: string;
  initialSitemap: AppSitemapResponse;
  entities: EntityResponse[];
};

type SelectionState =
  | { kind: "area"; areaIndex: number }
  | { kind: "group"; areaIndex: number; groupIndex: number }
  | {
      kind: "sub_area";
      areaIndex: number;
      groupIndex: number;
      subAreaIndex: number;
    };

type DragPayload =
  | { kind: "area"; areaIndex: number }
  | { kind: "group"; areaIndex: number; groupIndex: number }
  | {
      kind: "sub_area";
      areaIndex: number;
      groupIndex: number;
      subAreaIndex: number;
    };

function parseDragPayload(rawPayload: string): DragPayload | null {
  try {
    return JSON.parse(rawPayload) as DragPayload;
  } catch {
    return null;
  }
}

function reorderList<T>(
  items: T[],
  sourceIndex: number,
  targetIndex: number,
): T[] {
  if (sourceIndex === targetIndex) {
    return items;
  }
  const next = [...items];
  const [entry] = next.splice(sourceIndex, 1);
  next.splice(targetIndex, 0, entry);
  return next;
}

function moveGroup(
  areas: AppSitemapAreaDto[],
  sourceAreaIndex: number,
  sourceGroupIndex: number,
  targetAreaIndex: number,
  targetGroupIndex: number,
): AppSitemapAreaDto[] {
  const sourceArea = areas[sourceAreaIndex];
  if (!sourceArea) {
    return areas;
  }
  const movingGroup = sourceArea.groups[sourceGroupIndex];
  if (!movingGroup) {
    return areas;
  }

  const withoutSource = areas.map((area, areaIndex) => {
    if (areaIndex !== sourceAreaIndex) {
      return area;
    }
    return {
      ...area,
      groups: area.groups.filter((_, groupIndex) => groupIndex !== sourceGroupIndex),
    };
  });

  const normalizedTargetIndex =
    sourceAreaIndex === targetAreaIndex && sourceGroupIndex < targetGroupIndex
      ? targetGroupIndex - 1
      : targetGroupIndex;

  return withoutSource.map((area, areaIndex) => {
    if (areaIndex !== targetAreaIndex) {
      return area;
    }
    const nextGroups = [...area.groups];
    nextGroups.splice(Math.max(0, normalizedTargetIndex), 0, movingGroup);
    return {
      ...area,
      groups: nextGroups,
    };
  });
}

function moveSubArea(
  areas: AppSitemapAreaDto[],
  sourceAreaIndex: number,
  sourceGroupIndex: number,
  sourceSubAreaIndex: number,
  targetAreaIndex: number,
  targetGroupIndex: number,
  targetSubAreaIndex: number,
): AppSitemapAreaDto[] {
  const sourceGroup = areas[sourceAreaIndex]?.groups[sourceGroupIndex];
  if (!sourceGroup) {
    return areas;
  }
  const movingSubArea = sourceGroup.sub_areas[sourceSubAreaIndex];
  if (!movingSubArea) {
    return areas;
  }

  const withoutSource = areas.map((area, areaIndex) => {
    if (areaIndex !== sourceAreaIndex) {
      return area;
    }

    return {
      ...area,
      groups: area.groups.map((group, groupIndex) => {
        if (groupIndex !== sourceGroupIndex) {
          return group;
        }
        return {
          ...group,
          sub_areas: group.sub_areas.filter(
            (_, subAreaIndex) => subAreaIndex !== sourceSubAreaIndex,
          ),
        };
      }),
    };
  });

  const normalizedTargetIndex =
    sourceAreaIndex === targetAreaIndex &&
    sourceGroupIndex === targetGroupIndex &&
    sourceSubAreaIndex < targetSubAreaIndex
      ? targetSubAreaIndex - 1
      : targetSubAreaIndex;

  return withoutSource.map((area, areaIndex) => {
    if (areaIndex !== targetAreaIndex) {
      return area;
    }

    return {
      ...area,
      groups: area.groups.map((group, groupIndex) => {
        if (groupIndex !== targetGroupIndex) {
          return group;
        }
        const nextSubAreas = [...group.sub_areas];
        nextSubAreas.splice(Math.max(0, normalizedTargetIndex), 0, movingSubArea);
        return {
          ...group,
          sub_areas: nextSubAreas,
        };
      }),
    };
  });
}

function normalizeSitemap(sitemap: AppSitemapResponse): AppSitemapResponse {
  return {
    ...sitemap,
    areas: sitemap.areas.map((area, areaIndex) => ({
      ...area,
      position: areaIndex,
      groups: area.groups.map((group, groupIndex) => ({
        ...group,
        position: groupIndex,
        sub_areas: group.sub_areas.map((subArea, subAreaIndex) => ({
          ...subArea,
          position: subAreaIndex,
        })),
      })),
    })),
  };
}

function createDefaultArea(index: number): AppSitemapAreaDto {
  return {
    logical_name: `area_${index + 1}`,
    display_name: `Area ${index + 1}`,
    position: index,
    icon: null,
    groups: [],
  };
}

function createDefaultGroup(areaIndex: number, groupIndex: number) {
  return {
    logical_name: `group_${areaIndex + 1}_${groupIndex + 1}`,
    display_name: `Group ${groupIndex + 1}`,
    position: groupIndex,
    sub_areas: [],
  };
}

function createDefaultSubArea(
  areaIndex: number,
  groupIndex: number,
  subAreaIndex: number,
  entityLogicalName: string | null,
): AppSitemapSubAreaDto {
  const target: AppSitemapTargetDto = entityLogicalName
    ? {
        type: "entity",
        entity_logical_name: entityLogicalName,
        default_form: null,
        default_view: null,
      }
    : {
        type: "custom_page",
        url: "",
      };

  return {
    logical_name: `sub_area_${areaIndex + 1}_${groupIndex + 1}_${subAreaIndex + 1}`,
    display_name: `Sub Area ${subAreaIndex + 1}`,
    position: subAreaIndex,
    icon: null,
    target,
  };
}

function createCrmTemplateSitemap(
  appLogicalName: string,
  entities: EntityResponse[],
): AppSitemapResponse {
  const primaryEntities = entities.slice(0, 6);
  const coreEntities = primaryEntities.slice(0, 3);
  const activityEntities = primaryEntities.slice(3, 6);

  return normalizeSitemap({
    app_logical_name: appLogicalName,
    areas: [
      {
        logical_name: "operations",
        display_name: "Operations",
        position: 0,
        icon: "briefcase",
        groups: [
          {
            logical_name: "core",
            display_name: "Core",
            position: 0,
            sub_areas: coreEntities.map((entity, index) => ({
              logical_name: `${entity.logical_name}_home`,
              display_name: entity.display_name,
              position: index,
              icon: null,
              target: {
                type: "entity",
                entity_logical_name: entity.logical_name,
                default_form: null,
                default_view: null,
              },
            })),
          },
          {
            logical_name: "activity",
            display_name: "Activity",
            position: 1,
            sub_areas: activityEntities.map((entity, index) => ({
              logical_name: `${entity.logical_name}_activity`,
              display_name: entity.display_name,
              position: index,
              icon: null,
              target: {
                type: "entity",
                entity_logical_name: entity.logical_name,
                default_form: null,
                default_view: null,
              },
            })),
          },
        ],
      },
    ],
  });
}

export function SitemapEditorPanel({
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
  const [entityFormsByLogicalName, setEntityFormsByLogicalName] = useState<
    Record<string, FormResponse[]>
  >({});
  const [entityViewsByLogicalName, setEntityViewsByLogicalName] = useState<
    Record<string, ViewResponse[]>
  >({});
  const [isLoadingTargetMetadata, setIsLoadingTargetMetadata] = useState(false);

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

      setIsLoadingTargetMetadata(true);
      try {
        const [formsResponse, viewsResponse] = await Promise.all([
          apiFetch(`/api/entities/${selectedEntityLogicalName}/forms`),
          apiFetch(`/api/entities/${selectedEntityLogicalName}/views`),
        ]);

        if (formsResponse.ok) {
          const forms = (await formsResponse.json()) as FormResponse[];
          setEntityFormsByLogicalName((current) => ({
            ...current,
            [selectedEntityLogicalName]: forms,
          }));
        }
        if (viewsResponse.ok) {
          const views = (await viewsResponse.json()) as ViewResponse[];
          setEntityViewsByLogicalName((current) => ({
            ...current,
            [selectedEntityLogicalName]: views,
          }));
        }
      } finally {
        setIsLoadingTargetMetadata(false);
      }
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
        <Card className="h-fit">
          <CardHeader>
            <CardTitle className="text-base">Tree</CardTitle>
            <CardDescription>
              Drag and drop to reorder nodes within each hierarchy level.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-2" tabIndex={0} onKeyDown={handleCanvasKeyDown}>
            <DropLine
              lineId="area-insert-0"
              activeLineId={activeDropLineId}
              onSetActiveLine={setActiveDropLineId}
              onDrop={(event) => onDropLine({ kind: "area", areaIndex: 0 }, event)}
            />
            {sitemap.areas.map((area, areaIndex) => (
              <div
                key={`area-${area.logical_name}-${areaIndex}`}
                className="rounded-md border border-zinc-200 p-2"
              >
                <button
                  type="button"
                  className={`w-full rounded-md border px-2 py-1 text-left ${selection?.kind === "area" && selection.areaIndex === areaIndex ? "border-emerald-400 bg-emerald-50" : "border-zinc-200 bg-zinc-50"}`}
                  onClick={() => setSelection({ kind: "area", areaIndex })}
                  draggable
                    onDragStart={(event) =>
                      onDragStart({ kind: "area", areaIndex }, area.display_name, event)
                    }
                    onDragEnd={() => setDragLabel(null)}
                  onDragOver={(event) => event.preventDefault()}
                  onDrop={(event) =>
                    onDropNode({ kind: "area", areaIndex }, event)
                  }
                >
                  <p className="text-sm font-semibold">{area.display_name}</p>
                  <p className="font-mono text-xs text-zinc-500">
                    {area.logical_name}
                  </p>
                </button>
                <div className="mt-2">
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    onClick={() => addGroupToArea(areaIndex)}
                  >
                    + Group
                  </Button>
                </div>
                <div className="mt-2 space-y-2 pl-3">
                  <DropLine
                    lineId={`group-insert-${areaIndex}-0`}
                    activeLineId={activeDropLineId}
                    onSetActiveLine={setActiveDropLineId}
                    onDrop={(event) =>
                      onDropLine(
                        {
                          kind: "group",
                          areaIndex,
                          groupIndex: 0,
                        },
                        event,
                      )
                    }
                  />
                  {area.groups.map((group, groupIndex) => (
                    <div
                      key={`group-${group.logical_name}-${groupIndex}`}
                      className="rounded-md border border-zinc-100 p-2"
                    >
                      <button
                        type="button"
                        className={`w-full rounded-md border px-2 py-1 text-left ${selection?.kind === "group" && selection.areaIndex === areaIndex && selection.groupIndex === groupIndex ? "border-emerald-400 bg-emerald-50" : "border-zinc-200 bg-white"}`}
                        onClick={() =>
                          setSelection({ kind: "group", areaIndex, groupIndex })
                        }
                        draggable
                          onDragStart={(event) =>
                            onDragStart(
                              { kind: "group", areaIndex, groupIndex },
                              group.display_name,
                              event,
                            )
                          }
                          onDragEnd={() => setDragLabel(null)}
                        onDragOver={(event) => event.preventDefault()}
                        onDrop={(event) =>
                          onDropNode(
                            { kind: "group", areaIndex, groupIndex },
                            event,
                          )
                        }
                      >
                        <p className="text-sm font-medium">
                          {group.display_name}
                        </p>
                        <p className="font-mono text-xs text-zinc-500">
                          {group.logical_name}
                        </p>
                      </button>
                      <div className="mt-2">
                        <Button
                          type="button"
                          size="sm"
                          variant="outline"
                          onClick={() => addSubAreaToGroup(areaIndex, groupIndex)}
                        >
                          + Sub Area
                        </Button>
                      </div>
                      <div className="mt-2 space-y-2 pl-3">
                        <DropLine
                          lineId={`subarea-insert-${areaIndex}-${groupIndex}-0`}
                          activeLineId={activeDropLineId}
                          onSetActiveLine={setActiveDropLineId}
                          onDrop={(event) =>
                            onDropLine(
                              {
                                kind: "sub_area",
                                areaIndex,
                                groupIndex,
                                subAreaIndex: 0,
                              },
                              event,
                            )
                          }
                        />
                        {group.sub_areas.map((subArea, subAreaIndex) => (
                          <button
                            key={`sub-area-${subArea.logical_name}-${subAreaIndex}`}
                            type="button"
                            className={`w-full rounded-md border px-2 py-1 text-left ${selection?.kind === "sub_area" && selection.areaIndex === areaIndex && selection.groupIndex === groupIndex && selection.subAreaIndex === subAreaIndex ? "border-emerald-400 bg-emerald-50" : "border-zinc-200 bg-white"}`}
                            onClick={() =>
                              setSelection({
                                kind: "sub_area",
                                areaIndex,
                                groupIndex,
                                subAreaIndex,
                              })
                            }
                            draggable
                            onDragStart={(event) =>
                              onDragStart(
                                {
                                  kind: "sub_area",
                                  areaIndex,
                                  groupIndex,
                                  subAreaIndex,
                                },
                                subArea.display_name,
                                event,
                              )
                            }
                            onDragEnd={() => setDragLabel(null)}
                            onDragOver={(event) => event.preventDefault()}
                            onDrop={(event) =>
                              onDropNode(
                                {
                                  kind: "sub_area",
                                  areaIndex,
                                  groupIndex,
                                  subAreaIndex,
                                },
                                event,
                              )
                            }
                          >
                            <p className="text-sm">{subArea.display_name}</p>
                            <p className="font-mono text-xs text-zinc-500">
                              {subArea.logical_name}
                            </p>
                          </button>
                        ))}
                        <DropLine
                          lineId={`subarea-insert-${areaIndex}-${groupIndex}-${group.sub_areas.length}`}
                          activeLineId={activeDropLineId}
                          onSetActiveLine={setActiveDropLineId}
                          onDrop={(event) =>
                            onDropLine(
                              {
                                kind: "sub_area",
                                areaIndex,
                                groupIndex,
                                subAreaIndex: group.sub_areas.length,
                              },
                              event,
                            )
                          }
                        />
                      </div>
                    </div>
                  ))}
                  <DropLine
                    lineId={`group-insert-${areaIndex}-${area.groups.length}`}
                    activeLineId={activeDropLineId}
                    onSetActiveLine={setActiveDropLineId}
                    onDrop={(event) =>
                      onDropLine(
                        {
                          kind: "group",
                          areaIndex,
                          groupIndex: area.groups.length,
                        },
                        event,
                      )
                    }
                  />
                </div>
              </div>
            ))}
            <DropLine
              lineId={`area-insert-${sitemap.areas.length}`}
              activeLineId={activeDropLineId}
              onSetActiveLine={setActiveDropLineId}
              onDrop={(event) =>
                onDropLine({ kind: "area", areaIndex: sitemap.areas.length }, event)
              }
            />
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-base">Preview</CardTitle>
            <CardDescription>
              Worker sidebar preview updates as you modify the sitemap tree.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="space-y-3 rounded-md border border-zinc-200 bg-zinc-50 p-3">
              {sitemap.areas.map((area) => (
                <div
                  key={`preview-area-${area.logical_name}`}
                  className="space-y-2"
                >
                  <button
                    type="button"
                    className="text-left text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 hover:text-zinc-700"
                    onClick={() =>
                      setSelection({
                        kind: "area",
                        areaIndex: area.position,
                      })
                    }
                  >
                    {area.display_name}
                  </button>
                  <div className="space-y-2 pl-2">
                    {area.groups.map((group) => (
                      <details key={`preview-group-${group.logical_name}`} open>
                        <summary
                          className="cursor-pointer text-sm font-medium text-zinc-700"
                          onClick={() =>
                            setSelection({
                              kind: "group",
                              areaIndex: area.position,
                              groupIndex: group.position,
                            })
                          }
                        >
                          {group.display_name}
                        </summary>
                        <div className="mt-1 space-y-1 pl-3">
                          {group.sub_areas.map((subArea) => (
                            <button
                              key={`preview-sub-area-${subArea.logical_name}`}
                              type="button"
                              className="text-left text-sm text-zinc-600 hover:text-zinc-900"
                              onClick={() =>
                                setSelection({
                                  kind: "sub_area",
                                  areaIndex: area.position,
                                  groupIndex: group.position,
                                  subAreaIndex: subArea.position,
                                })
                              }
                            >
                              {subArea.display_name}
                            </button>
                          ))}
                        </div>
                      </details>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>

        <Card className="h-fit">
          <CardHeader>
            <CardTitle className="text-base">Properties</CardTitle>
            <CardDescription>
              Configure selected node metadata and target behavior.
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            {selection?.kind === "area" && selectedArea ? (
              <>
                <div className="space-y-2">
                  <Label htmlFor="area_display_name">Area Display Name</Label>
                  <Input
                    id="area_display_name"
                    value={selectedArea.display_name}
                    onChange={(event) =>
                      updateSitemap((current) => ({
                        ...current,
                        areas: current.areas.map((area, index) =>
                          index === selection.areaIndex
                            ? { ...area, display_name: event.target.value }
                            : area,
                        ),
                      }))
                    }
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="area_logical_name">Area Logical Name</Label>
                  <Input
                    id="area_logical_name"
                    value={selectedArea.logical_name}
                    onChange={(event) =>
                      updateSitemap((current) => ({
                        ...current,
                        areas: current.areas.map((area, index) =>
                          index === selection.areaIndex
                            ? { ...area, logical_name: event.target.value }
                            : area,
                        ),
                      }))
                    }
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="area_icon">Area Icon</Label>
                  <Input
                    id="area_icon"
                    value={selectedArea.icon ?? ""}
                    onChange={(event) =>
                      updateSitemap((current) => ({
                        ...current,
                        areas: current.areas.map((area, index) =>
                          index === selection.areaIndex
                            ? {
                                ...area,
                                icon:
                                  event.target.value.trim().length > 0
                                    ? event.target.value
                                    : null,
                              }
                            : area,
                        ),
                      }))
                    }
                  />
                </div>
              </>
            ) : null}

            {selection?.kind === "group" && selectedGroup ? (
              <>
                <div className="space-y-2">
                  <Label htmlFor="group_display_name">Group Display Name</Label>
                  <Input
                    id="group_display_name"
                    value={selectedGroup.display_name}
                    onChange={(event) =>
                      updateSitemap((current) => ({
                        ...current,
                        areas: current.areas.map((area, areaIndex) => {
                          if (areaIndex !== selection.areaIndex) {
                            return area;
                          }
                          return {
                            ...area,
                            groups: area.groups.map((group, groupIndex) =>
                              groupIndex === selection.groupIndex
                                ? { ...group, display_name: event.target.value }
                                : group,
                            ),
                          };
                        }),
                      }))
                    }
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="group_logical_name">Group Logical Name</Label>
                  <Input
                    id="group_logical_name"
                    value={selectedGroup.logical_name}
                    onChange={(event) =>
                      updateSitemap((current) => ({
                        ...current,
                        areas: current.areas.map((area, areaIndex) => {
                          if (areaIndex !== selection.areaIndex) {
                            return area;
                          }
                          return {
                            ...area,
                            groups: area.groups.map((group, groupIndex) =>
                              groupIndex === selection.groupIndex
                                ? { ...group, logical_name: event.target.value }
                                : group,
                            ),
                          };
                        }),
                      }))
                    }
                  />
                </div>
              </>
            ) : null}

            {selection?.kind === "sub_area" && selectedSubArea ? (
              <>
                <div className="space-y-2">
                  <Label htmlFor="sub_area_display_name">
                    Sub Area Display Name
                  </Label>
                  <Input
                    id="sub_area_display_name"
                    value={selectedSubArea.display_name}
                    onChange={(event) =>
                      updateSitemap((current) => ({
                        ...current,
                        areas: current.areas.map((area, areaIndex) => {
                          if (areaIndex !== selection.areaIndex) {
                            return area;
                          }
                          return {
                            ...area,
                            groups: area.groups.map((group, groupIndex) => {
                              if (groupIndex !== selection.groupIndex) {
                                return group;
                              }
                              return {
                                ...group,
                                sub_areas: group.sub_areas.map(
                                  (subArea, subAreaIndex) =>
                                    subAreaIndex === selection.subAreaIndex
                                      ? {
                                          ...subArea,
                                          display_name: event.target.value,
                                        }
                                      : subArea,
                                ),
                              };
                            }),
                          };
                        }),
                      }))
                    }
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="sub_area_icon">Sub Area Icon</Label>
                  <Input
                    id="sub_area_icon"
                    value={selectedSubArea.icon ?? ""}
                    onChange={(event) =>
                      updateSitemap((current) => ({
                        ...current,
                        areas: current.areas.map((area, areaIndex) => {
                          if (areaIndex !== selection.areaIndex) {
                            return area;
                          }
                          return {
                            ...area,
                            groups: area.groups.map((group, groupIndex) => {
                              if (groupIndex !== selection.groupIndex) {
                                return group;
                              }
                              return {
                                ...group,
                                sub_areas: group.sub_areas.map(
                                  (subArea, subAreaIndex) =>
                                    subAreaIndex === selection.subAreaIndex
                                      ? {
                                          ...subArea,
                                          icon:
                                            event.target.value.trim().length > 0
                                              ? event.target.value
                                              : null,
                                        }
                                      : subArea,
                                ),
                              };
                            }),
                          };
                        }),
                      }))
                    }
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="sub_area_target_type">Target Type</Label>
                  <Select
                    id="sub_area_target_type"
                    value={selectedSubArea.target.type}
                    onChange={(event) =>
                      updateSitemap((current) => ({
                        ...current,
                        areas: current.areas.map((area, areaIndex) => {
                          if (areaIndex !== selection.areaIndex) {
                            return area;
                          }
                          return {
                            ...area,
                            groups: area.groups.map((group, groupIndex) => {
                              if (groupIndex !== selection.groupIndex) {
                                return group;
                              }
                              return {
                                ...group,
                                sub_areas: group.sub_areas.map(
                                  (subArea, subAreaIndex) => {
                                    if (
                                      subAreaIndex !== selection.subAreaIndex
                                    ) {
                                      return subArea;
                                    }
                                    const nextType = event.target
                                      .value as AppSitemapTargetDto["type"];
                                    const nextTarget: AppSitemapTargetDto =
                                      nextType === "entity"
                                        ? {
                                            type: "entity",
                                            entity_logical_name:
                                              entities[0]?.logical_name ?? "",
                                            default_form: null,
                                            default_view: null,
                                          }
                                        : nextType === "dashboard"
                                          ? {
                                              type: "dashboard",
                                              dashboard_logical_name: "",
                                            }
                                          : {
                                              type: "custom_page",
                                              url: "",
                                            };
                                    return {
                                      ...subArea,
                                      target: nextTarget,
                                    };
                                  },
                                ),
                              };
                            }),
                          };
                        }),
                      }))
                    }
                  >
                    <option value="entity">Entity</option>
                    <option value="dashboard">Dashboard</option>
                    <option value="custom_page">Custom Page</option>
                  </Select>
                </div>

                {selectedSubArea.target.type === "entity" ? (
                  <>
                    <div className="space-y-2">
                      <Label htmlFor="sub_area_target_entity">Entity</Label>
                      <Select
                        id="sub_area_target_entity"
                        value={selectedSubArea.target.entity_logical_name}
                        onChange={(event) =>
                          updateSitemap((current) => ({
                            ...current,
                            areas: current.areas.map((area, areaIndex) => {
                              if (areaIndex !== selection.areaIndex) {
                                return area;
                              }
                              return {
                                ...area,
                                groups: area.groups.map((group, groupIndex) => {
                                  if (groupIndex !== selection.groupIndex) {
                                    return group;
                                  }
                                  return {
                                    ...group,
                                    sub_areas: group.sub_areas.map(
                                      (subArea, subAreaIndex) => {
                                        if (
                                          subAreaIndex !==
                                          selection.subAreaIndex
                                        ) {
                                          return subArea;
                                        }
                                        if (subArea.target.type !== "entity") {
                                          return subArea;
                                        }
                                        return {
                                          ...subArea,
                                          target: {
                                            ...subArea.target,
                                            entity_logical_name:
                                              event.target.value,
                                            default_form: null,
                                            default_view: null,
                                          },
                                        };
                                      },
                                    ),
                                  };
                                }),
                              };
                            }),
                          }))
                        }
                      >
                        {entities.map((entity) => (
                          <option
                            key={entity.logical_name}
                            value={entity.logical_name}
                          >
                            {entity.display_name} ({entity.logical_name})
                          </option>
                        ))}
                      </Select>
                    </div>

                    <div className="space-y-2">
                      <Label htmlFor="sub_area_target_form">Default Form</Label>
                      <Select
                        id="sub_area_target_form"
                        value={selectedSubArea.target.default_form ?? ""}
                        disabled={isLoadingTargetMetadata}
                        onChange={(event) =>
                          updateSitemap((current) => ({
                            ...current,
                            areas: current.areas.map((area, areaIndex) => {
                              if (areaIndex !== selection.areaIndex) {
                                return area;
                              }
                              return {
                                ...area,
                                groups: area.groups.map((group, groupIndex) => {
                                  if (groupIndex !== selection.groupIndex) {
                                    return group;
                                  }
                                  return {
                                    ...group,
                                    sub_areas: group.sub_areas.map(
                                      (subArea, subAreaIndex) => {
                                        if (
                                          subAreaIndex !==
                                          selection.subAreaIndex
                                        ) {
                                          return subArea;
                                        }
                                        if (subArea.target.type !== "entity") {
                                          return subArea;
                                        }
                                        return {
                                          ...subArea,
                                          target: {
                                            ...subArea.target,
                                            default_form:
                                              event.target.value.trim().length >
                                              0
                                                ? event.target.value
                                                : null,
                                          },
                                        };
                                      },
                                    ),
                                  };
                                }),
                              };
                            }),
                          }))
                        }
                      >
                        <option value="">None</option>
                        {selectedEntityForms.map((form) => (
                          <option
                            key={form.logical_name}
                            value={form.logical_name}
                          >
                            {form.display_name} ({form.logical_name})
                          </option>
                        ))}
                      </Select>
                    </div>

                    <div className="space-y-2">
                      <Label htmlFor="sub_area_target_view">Default View</Label>
                      <Select
                        id="sub_area_target_view"
                        value={selectedSubArea.target.default_view ?? ""}
                        disabled={isLoadingTargetMetadata}
                        onChange={(event) =>
                          updateSitemap((current) => ({
                            ...current,
                            areas: current.areas.map((area, areaIndex) => {
                              if (areaIndex !== selection.areaIndex) {
                                return area;
                              }
                              return {
                                ...area,
                                groups: area.groups.map((group, groupIndex) => {
                                  if (groupIndex !== selection.groupIndex) {
                                    return group;
                                  }
                                  return {
                                    ...group,
                                    sub_areas: group.sub_areas.map(
                                      (subArea, subAreaIndex) => {
                                        if (
                                          subAreaIndex !==
                                          selection.subAreaIndex
                                        ) {
                                          return subArea;
                                        }
                                        if (subArea.target.type !== "entity") {
                                          return subArea;
                                        }
                                        return {
                                          ...subArea,
                                          target: {
                                            ...subArea.target,
                                            default_view:
                                              event.target.value.trim().length >
                                              0
                                                ? event.target.value
                                                : null,
                                          },
                                        };
                                      },
                                    ),
                                  };
                                }),
                              };
                            }),
                          }))
                        }
                      >
                        <option value="">None</option>
                        {selectedEntityViews.map((view) => (
                          <option
                            key={view.logical_name}
                            value={view.logical_name}
                          >
                            {view.display_name} ({view.logical_name})
                          </option>
                        ))}
                      </Select>
                    </div>
                  </>
                ) : null}

                {selectedSubArea.target.type === "dashboard" ? (
                  <div className="space-y-2">
                    <Label htmlFor="sub_area_target_dashboard">
                      Dashboard Logical Name
                    </Label>
                    <Input
                      id="sub_area_target_dashboard"
                      value={selectedSubArea.target.dashboard_logical_name}
                      onChange={(event) =>
                        updateSitemap((current) => ({
                          ...current,
                          areas: current.areas.map((area, areaIndex) => {
                            if (areaIndex !== selection.areaIndex) {
                              return area;
                            }
                            return {
                              ...area,
                              groups: area.groups.map((group, groupIndex) => {
                                if (groupIndex !== selection.groupIndex) {
                                  return group;
                                }
                                return {
                                  ...group,
                                  sub_areas: group.sub_areas.map(
                                    (subArea, subAreaIndex) => {
                                      if (
                                        subAreaIndex !== selection.subAreaIndex
                                      ) {
                                        return subArea;
                                      }
                                      if (subArea.target.type !== "dashboard") {
                                        return subArea;
                                      }
                                      return {
                                        ...subArea,
                                        target: {
                                          ...subArea.target,
                                          dashboard_logical_name:
                                            event.target.value,
                                        },
                                      };
                                    },
                                  ),
                                };
                              }),
                            };
                          }),
                        }))
                      }
                    />
                  </div>
                ) : null}

                {selectedSubArea.target.type === "custom_page" ? (
                  <div className="space-y-2">
                    <Label htmlFor="sub_area_target_url">Custom Page URL</Label>
                    <Input
                      id="sub_area_target_url"
                      value={selectedSubArea.target.url}
                      onChange={(event) =>
                        updateSitemap((current) => ({
                          ...current,
                          areas: current.areas.map((area, areaIndex) => {
                            if (areaIndex !== selection.areaIndex) {
                              return area;
                            }
                            return {
                              ...area,
                              groups: area.groups.map((group, groupIndex) => {
                                if (groupIndex !== selection.groupIndex) {
                                  return group;
                                }
                                return {
                                  ...group,
                                  sub_areas: group.sub_areas.map(
                                    (subArea, subAreaIndex) => {
                                      if (
                                        subAreaIndex !== selection.subAreaIndex
                                      ) {
                                        return subArea;
                                      }
                                      if (
                                        subArea.target.type !== "custom_page"
                                      ) {
                                        return subArea;
                                      }
                                      return {
                                        ...subArea,
                                        target: {
                                          ...subArea.target,
                                          url: event.target.value,
                                        },
                                      };
                                    },
                                  ),
                                };
                              }),
                            };
                          }),
                        }))
                      }
                    />
                  </div>
                ) : null}
              </>
            ) : null}
          </CardContent>
        </Card>
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

function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) {
    return false;
  }

  const tagName = target.tagName;
  return tagName === "INPUT" || tagName === "TEXTAREA" || tagName === "SELECT" || target.isContentEditable;
}

type DropLineProps = {
  lineId: string;
  activeLineId: string | null;
  onSetActiveLine: (lineId: string | null) => void;
  label?: string;
  onDrop: (event: DragEvent<HTMLDivElement>) => void;
};

function DropLine({ lineId, activeLineId, onSetActiveLine, label, onDrop }: DropLineProps) {
  const isActive = activeLineId === lineId;
  return (
    <div
      className={`rounded border border-dashed px-2 py-0.5 text-[10px] transition ${isActive ? "border-emerald-400 bg-emerald-100 text-emerald-900" : "border-transparent text-transparent hover:border-emerald-300 hover:bg-emerald-100 hover:text-emerald-800"}`}
      onDragOver={(event) => {
        event.preventDefault();
        onSetActiveLine(lineId);
      }}
      onDragEnter={() => onSetActiveLine(lineId)}
      onDragLeave={() => onSetActiveLine(null)}
      onDrop={(event) => {
        onSetActiveLine(null);
        onDrop(event);
      }}
      aria-hidden
    >
      {label ?? "Insert here"}
    </div>
  );
}
