import { Button, Input } from "@qryvanta/ui";

import type { AppSurfaceDraft } from "@/components/apps/app-studio/sections/types";

type SurfaceCatalogEditorProps = {
  title: string;
  surfaces: AppSurfaceDraft[];
  defaultLogicalName: string;
  onSetDefault: (logicalName: string) => void;
  onRename: (
    logicalName: string,
    key: "logicalName" | "displayName",
    value: string,
  ) => void;
  onDelete: (logicalName: string) => void;
};

export function SurfaceCatalogEditor({
  title,
  surfaces,
  defaultLogicalName,
  onSetDefault,
  onRename,
  onDelete,
}: SurfaceCatalogEditorProps) {
  return (
    <div className="space-y-2 rounded-md border border-zinc-200 bg-white p-2">
      <p className="text-xs font-semibold uppercase tracking-wide text-zinc-500">{title}</p>
      <div className="space-y-2">
        {surfaces.map((surface) => (
          <div
            key={`${title.toLowerCase()}-catalog-${surface.logicalName}`}
            className="rounded-md border border-zinc-200 p-2"
          >
            <div className="grid gap-2 md:grid-cols-[1fr_1fr_auto_auto]">
              <Input
                value={surface.displayName}
                onChange={(event) =>
                  onRename(surface.logicalName, "displayName", event.target.value)
                }
                placeholder="Display Name"
              />
              <Input
                value={surface.logicalName}
                onChange={(event) =>
                  onRename(surface.logicalName, "logicalName", event.target.value)
                }
                placeholder="logical_name"
              />
              <Button
                type="button"
                size="sm"
                variant={surface.logicalName === defaultLogicalName ? "default" : "outline"}
                onClick={() => onSetDefault(surface.logicalName)}
              >
                {surface.logicalName === defaultLogicalName ? "Default" : "Make Default"}
              </Button>
              <Button
                type="button"
                size="sm"
                variant="outline"
                onClick={() => onDelete(surface.logicalName)}
                disabled={surfaces.length <= 1 || surface.logicalName === defaultLogicalName}
              >
                Delete
              </Button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
