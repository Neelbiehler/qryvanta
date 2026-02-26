import type { DragEvent, KeyboardEvent } from "react";

import { Button, Card, CardContent, CardDescription, CardHeader, CardTitle } from "@qryvanta/ui";

import { DropLine } from "@/components/apps/sitemap-editor/drop-line";
import type { DragPayload, SelectionState } from "@/components/apps/sitemap-editor/types";
import type { AppSitemapResponse } from "@/lib/api";

type SitemapTreeCardProps = {
  sitemap: AppSitemapResponse;
  selection: SelectionState | null;
  activeDropLineId: string | null;
  onSetActiveDropLineId: (value: string | null) => void;
  onCanvasKeyDown: (event: KeyboardEvent<HTMLDivElement>) => void;
  onSelectNode: (selection: SelectionState) => void;
  onDragStart: (
    payload: DragPayload,
    label: string,
    event: DragEvent<HTMLButtonElement>,
  ) => void;
  onDragEnd: () => void;
  onDropNode: (targetPayload: DragPayload, event: DragEvent<HTMLButtonElement>) => void;
  onDropLine: (targetPayload: DragPayload, event: DragEvent<HTMLDivElement>) => void;
  onAddGroupToArea: (areaIndex: number) => void;
  onAddSubAreaToGroup: (areaIndex: number, groupIndex: number) => void;
};

export function SitemapTreeCard({
  sitemap,
  selection,
  activeDropLineId,
  onSetActiveDropLineId,
  onCanvasKeyDown,
  onSelectNode,
  onDragStart,
  onDragEnd,
  onDropNode,
  onDropLine,
  onAddGroupToArea,
  onAddSubAreaToGroup,
}: SitemapTreeCardProps) {
  return (
    <Card className="h-fit">
      <CardHeader>
        <CardTitle className="text-base">Tree</CardTitle>
        <CardDescription>
          Drag and drop to reorder nodes within each hierarchy level.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-2" tabIndex={0} onKeyDown={onCanvasKeyDown}>
        <DropLine
          lineId="area-insert-0"
          activeLineId={activeDropLineId}
          onSetActiveLine={onSetActiveDropLineId}
          onDrop={(event) => onDropLine({ kind: "area", areaIndex: 0 }, event)}
        />
        {sitemap.areas.map((area, areaIndex) => (
          <div key={`area-${area.logical_name}-${areaIndex}`} className="rounded-md border border-zinc-200 p-2">
            <button
              type="button"
              className={`w-full rounded-md border px-2 py-1 text-left ${selection?.kind === "area" && selection.areaIndex === areaIndex ? "border-emerald-400 bg-emerald-50" : "border-zinc-200 bg-zinc-50"}`}
              onClick={() => onSelectNode({ kind: "area", areaIndex })}
              draggable
              onDragStart={(event) => onDragStart({ kind: "area", areaIndex }, area.display_name, event)}
              onDragEnd={onDragEnd}
              onDragOver={(event) => event.preventDefault()}
              onDrop={(event) => onDropNode({ kind: "area", areaIndex }, event)}
            >
              <p className="text-sm font-semibold">{area.display_name}</p>
              <p className="font-mono text-xs text-zinc-500">{area.logical_name}</p>
            </button>
            <div className="mt-2">
              <Button type="button" size="sm" variant="outline" onClick={() => onAddGroupToArea(areaIndex)}>
                + Group
              </Button>
            </div>
            <div className="mt-2 space-y-2 pl-3">
              <DropLine
                lineId={`group-insert-${areaIndex}-0`}
                activeLineId={activeDropLineId}
                onSetActiveLine={onSetActiveDropLineId}
                onDrop={(event) =>
                  onDropLine({ kind: "group", areaIndex, groupIndex: 0 }, event)
                }
              />
              {area.groups.map((group, groupIndex) => (
                <div key={`group-${group.logical_name}-${groupIndex}`} className="rounded-md border border-zinc-100 p-2">
                  <button
                    type="button"
                    className={`w-full rounded-md border px-2 py-1 text-left ${selection?.kind === "group" && selection.areaIndex === areaIndex && selection.groupIndex === groupIndex ? "border-emerald-400 bg-emerald-50" : "border-zinc-200 bg-white"}`}
                    onClick={() => onSelectNode({ kind: "group", areaIndex, groupIndex })}
                    draggable
                    onDragStart={(event) => onDragStart({ kind: "group", areaIndex, groupIndex }, group.display_name, event)}
                    onDragEnd={onDragEnd}
                    onDragOver={(event) => event.preventDefault()}
                    onDrop={(event) => onDropNode({ kind: "group", areaIndex, groupIndex }, event)}
                  >
                    <p className="text-sm font-medium">{group.display_name}</p>
                    <p className="font-mono text-xs text-zinc-500">{group.logical_name}</p>
                  </button>
                  <div className="mt-2">
                    <Button
                      type="button"
                      size="sm"
                      variant="outline"
                      onClick={() => onAddSubAreaToGroup(areaIndex, groupIndex)}
                    >
                      + Sub Area
                    </Button>
                  </div>
                  <div className="mt-2 space-y-2 pl-3">
                    <DropLine
                      lineId={`subarea-insert-${areaIndex}-${groupIndex}-0`}
                      activeLineId={activeDropLineId}
                      onSetActiveLine={onSetActiveDropLineId}
                      onDrop={(event) =>
                        onDropLine({ kind: "sub_area", areaIndex, groupIndex, subAreaIndex: 0 }, event)
                      }
                    />
                    {group.sub_areas.map((subArea, subAreaIndex) => (
                      <button
                        key={`sub-area-${subArea.logical_name}-${subAreaIndex}`}
                        type="button"
                        className={`w-full rounded-md border px-2 py-1 text-left ${selection?.kind === "sub_area" && selection.areaIndex === areaIndex && selection.groupIndex === groupIndex && selection.subAreaIndex === subAreaIndex ? "border-emerald-400 bg-emerald-50" : "border-zinc-200 bg-white"}`}
                        onClick={() => onSelectNode({ kind: "sub_area", areaIndex, groupIndex, subAreaIndex })}
                        draggable
                        onDragStart={(event) => onDragStart({ kind: "sub_area", areaIndex, groupIndex, subAreaIndex }, subArea.display_name, event)}
                        onDragEnd={onDragEnd}
                        onDragOver={(event) => event.preventDefault()}
                        onDrop={(event) => onDropNode({ kind: "sub_area", areaIndex, groupIndex, subAreaIndex }, event)}
                      >
                        <p className="text-sm">{subArea.display_name}</p>
                        <p className="font-mono text-xs text-zinc-500">{subArea.logical_name}</p>
                      </button>
                    ))}
                    <DropLine
                      lineId={`subarea-insert-${areaIndex}-${groupIndex}-${group.sub_areas.length}`}
                      activeLineId={activeDropLineId}
                      onSetActiveLine={onSetActiveDropLineId}
                      onDrop={(event) =>
                        onDropLine({ kind: "sub_area", areaIndex, groupIndex, subAreaIndex: group.sub_areas.length }, event)
                      }
                    />
                  </div>
                </div>
              ))}
              <DropLine
                lineId={`group-insert-${areaIndex}-${area.groups.length}`}
                activeLineId={activeDropLineId}
                onSetActiveLine={onSetActiveDropLineId}
                onDrop={(event) =>
                  onDropLine({ kind: "group", areaIndex, groupIndex: area.groups.length }, event)
                }
              />
            </div>
          </div>
        ))}
        <DropLine
          lineId={`area-insert-${sitemap.areas.length}`}
          activeLineId={activeDropLineId}
          onSetActiveLine={onSetActiveDropLineId}
          onDrop={(event) => onDropLine({ kind: "area", areaIndex: sitemap.areas.length }, event)}
        />
      </CardContent>
    </Card>
  );
}
