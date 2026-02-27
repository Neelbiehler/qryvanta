"use client";

import { Button, Select, StatusBadge } from "@qryvanta/ui";

import type { StudioController } from "@/components/studio/hooks/use-studio-state";

type StudioToolbarProps = {
  studio: StudioController;
};

export function StudioToolbar({ studio }: StudioToolbarProps) {
  const isFormActive = studio.selection.kind === "form" && studio.formEditor !== null;
  const isViewActive = studio.selection.kind === "view" && studio.viewEditor !== null;

  return (
    <div className="flex flex-wrap items-center gap-2 rounded-lg border border-zinc-200 bg-white px-3 py-2">
      {/* App selector */}
      <Select
        value={studio.selectedApp}
        onChange={(event) => studio.setSelectedApp(event.target.value)}
        className="w-auto min-w-[140px]"
      >
        {studio.apps.map((app) => (
          <option key={app.logical_name} value={app.logical_name}>
            {app.display_name}
          </option>
        ))}
      </Select>

      <div className="h-5 w-px bg-zinc-200" />

      {/* Context badges */}
      <StatusBadge tone="neutral">
        Entities {studio.entities.length}
      </StatusBadge>

      {studio.selection.kind === "form" ? (
        <StatusBadge tone="success">
          Editing form
        </StatusBadge>
      ) : studio.selection.kind === "view" ? (
        <StatusBadge tone="success">
          Editing view
        </StatusBadge>
      ) : (
        <StatusBadge tone="neutral">
          {studio.selection.kind.charAt(0).toUpperCase() + studio.selection.kind.slice(1)}
        </StatusBadge>
      )}

      {/* Spacer */}
      <div className="flex-1" />

      <Button
        type="button"
        size="sm"
        variant="outline"
        disabled={!studio.selectedApp}
        onClick={() => {
          if (!studio.selectedApp) return;
          window.open(
            `/worker/apps/${encodeURIComponent(studio.selectedApp)}`,
            "_blank",
            "noopener,noreferrer",
          );
        }}
      >
        Play
      </Button>

      {/* Form-specific actions */}
      {isFormActive ? (
        <>
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={studio.formEditor!.undo}
            disabled={!studio.formEditor!.canUndo}
          >
            Undo
          </Button>
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={studio.formEditor!.redo}
            disabled={!studio.formEditor!.canRedo}
          >
            Redo
          </Button>

          <div className="h-5 w-px bg-zinc-200" />
        </>
      ) : null}

      {/* Save */}
      {isFormActive || isViewActive ? (
        <Button
          type="button"
          size="sm"
          onClick={() => {
            if (isFormActive) {
              void studio.handleSaveForm();
            } else {
              void studio.handleSaveView();
            }
          }}
          disabled={studio.isSaving}
        >
          {studio.isSaving ? "Saving..." : isViewActive ? "Save View" : "Save"}
        </Button>
      ) : null}
    </div>
  );
}
