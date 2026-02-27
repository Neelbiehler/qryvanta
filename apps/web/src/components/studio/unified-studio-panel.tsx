"use client";

import { Notice } from "@qryvanta/ui";

import type {
  AppEntityBindingResponse,
  AppResponse,
  EntityResponse,
  RoleResponse,
} from "@/lib/api";
import { useStudioState } from "@/components/studio/hooks/use-studio-state";
import { EntityTreeSidebar } from "@/components/studio/entity-tree-sidebar";
import { StudioCanvas } from "@/components/studio/studio-canvas";
import { StudioToolbar } from "@/components/studio/studio-toolbar";
import { StudioPropertiesPanel } from "@/components/studio/studio-properties-panel";

type UnifiedStudioPanelProps = {
  initialAppLogicalName: string;
  apps: AppResponse[];
  entities: EntityResponse[];
  roles: RoleResponse[];
  bindings: AppEntityBindingResponse[];
};

export function UnifiedStudioPanel({
  initialAppLogicalName,
  apps,
  entities,
  roles,
  bindings,
}: UnifiedStudioPanelProps) {
  const studio = useStudioState({
    initialAppLogicalName,
    apps,
    entities,
    roles,
    bindings,
  });

  const hasStudioData = apps.length > 0 && entities.length > 0;

  return (
    <div className="flex h-full min-h-0 flex-col gap-2 p-2">
      {!hasStudioData ? (
        <Notice tone="warning">
          Create at least one app and one entity before using the Studio.
        </Notice>
      ) : null}

      <StudioToolbar studio={studio} />

      <div className="grid min-h-0 flex-1 gap-2 xl:grid-cols-[260px_minmax(0,1fr)_280px]">
        <EntityTreeSidebar studio={studio} />
        <StudioCanvas studio={studio} />
        <StudioPropertiesPanel studio={studio} />
      </div>

      {studio.errorMessage ? (
        <Notice tone="error">{studio.errorMessage}</Notice>
      ) : null}
      {studio.statusMessage ? (
        <Notice tone="success">{studio.statusMessage}</Notice>
      ) : null}
      {studio.formEditor?.dragLabel ? (
        <Notice tone="neutral">
          Dragging &quot;{studio.formEditor.dragLabel}&quot; - drop on a highlighted insertion line.
        </Notice>
      ) : null}
    </div>
  );
}
