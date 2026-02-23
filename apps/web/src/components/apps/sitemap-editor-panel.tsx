"use client";

import { type DragEvent, useEffect, useMemo, useState } from "react";
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
  ): void {
    setSitemap((current) => normalizeSitemap(mutator(current)));
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

  function onDragStart(
    payload: DragPayload,
    event: DragEvent<HTMLButtonElement>,
  ) {
    event.dataTransfer.setData("text/sitemap-node", JSON.stringify(payload));
    event.dataTransfer.effectAllowed = "move";
  }

  function onDropNode(
    targetPayload: DragPayload,
    event: DragEvent<HTMLButtonElement>,
  ) {
    event.preventDefault();
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
      if (targetPayload.kind !== "group") {
        return;
      }
      if (sourcePayload.areaIndex !== targetPayload.areaIndex) {
        return;
      }
      const areaIndex = sourcePayload.areaIndex;
      updateSitemap((current) => ({
        ...current,
        areas: current.areas.map((area, currentAreaIndex) => {
          if (currentAreaIndex !== areaIndex) {
            return area;
          }
          return {
            ...area,
            groups: reorderList(
              area.groups,
              sourcePayload.groupIndex,
              targetPayload.groupIndex,
            ),
          };
        }),
      }));
      return;
    }

    if (sourcePayload.kind === "sub_area") {
      if (targetPayload.kind !== "sub_area") {
        return;
      }
      if (
        sourcePayload.areaIndex !== targetPayload.areaIndex ||
        sourcePayload.groupIndex !== targetPayload.groupIndex
      ) {
        return;
      }
      const { areaIndex, groupIndex } = sourcePayload;
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
                sub_areas: reorderList(
                  group.sub_areas,
                  sourcePayload.subAreaIndex,
                  targetPayload.subAreaIndex,
                ),
              };
            }),
          };
        }),
      }));
    }
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
          <CardContent className="space-y-2">
            {sitemap.areas.map((area, areaIndex) => (
              <div
                key={`area-${area.logical_name}-${areaIndex}`}
                className="rounded-md border border-zinc-200 p-2"
              >
                <button
                  type="button"
                  className="w-full rounded-md border border-zinc-200 bg-zinc-50 px-2 py-1 text-left"
                  onClick={() => setSelection({ kind: "area", areaIndex })}
                  draggable
                  onDragStart={(event) =>
                    onDragStart({ kind: "area", areaIndex }, event)
                  }
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
                <div className="mt-2 space-y-2 pl-3">
                  {area.groups.map((group, groupIndex) => (
                    <div
                      key={`group-${group.logical_name}-${groupIndex}`}
                      className="rounded-md border border-zinc-100 p-2"
                    >
                      <button
                        type="button"
                        className="w-full rounded-md border border-zinc-200 bg-white px-2 py-1 text-left"
                        onClick={() =>
                          setSelection({ kind: "group", areaIndex, groupIndex })
                        }
                        draggable
                        onDragStart={(event) =>
                          onDragStart(
                            { kind: "group", areaIndex, groupIndex },
                            event,
                          )
                        }
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
                      <div className="mt-2 space-y-2 pl-3">
                        {group.sub_areas.map((subArea, subAreaIndex) => (
                          <button
                            key={`sub-area-${subArea.logical_name}-${subAreaIndex}`}
                            type="button"
                            className="w-full rounded-md border border-zinc-200 bg-white px-2 py-1 text-left"
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
                                event,
                              )
                            }
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
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            ))}
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
                  <p className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500">
                    {area.display_name}
                  </p>
                  <div className="space-y-2 pl-2">
                    {area.groups.map((group) => (
                      <details key={`preview-group-${group.logical_name}`} open>
                        <summary className="cursor-pointer text-sm font-medium text-zinc-700">
                          {group.display_name}
                        </summary>
                        <div className="mt-1 space-y-1 pl-3">
                          {group.sub_areas.map((subArea) => (
                            <p
                              key={`preview-sub-area-${subArea.logical_name}`}
                              className="text-sm text-zinc-600"
                            >
                              {subArea.display_name}
                            </p>
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

      {errorMessage ? <Notice tone="error">{errorMessage}</Notice> : null}
      {statusMessage ? <Notice tone="success">{statusMessage}</Notice> : null}
    </div>
  );
}
