import { type DragEvent, type KeyboardEvent } from "react";

import { Button, Select } from "@qryvanta/ui";

type FieldDropZoneProps = {
  title: string;
  helperText: string;
  dragSource: "form" | "view";
  fieldLogicalNames: string[];
  fieldLabelByLogicalName: Map<string, string>;
  onAddField: (logicalName: string) => void;
  onMoveField: (logicalName: string, direction: "up" | "down") => void;
  onRemoveField: (logicalName: string) => void;
  onDrop: (event: DragEvent<HTMLDivElement>) => void;
  onDropAtIndex: (event: DragEvent<HTMLDivElement>, index: number) => void;
  dropLinePrefix: string;
  activeDropLineId: string | null;
  onSetActiveDropLineId: (value: string | null) => void;
  onDragStart: (
    event: DragEvent<HTMLButtonElement>,
    logicalName: string,
    source: "available" | "form" | "view",
  ) => void;
  onDragEnd: () => void;
};

export function FieldDropZone({
  title,
  helperText,
  dragSource,
  fieldLogicalNames,
  fieldLabelByLogicalName,
  onAddField,
  onMoveField,
  onRemoveField,
  onDrop,
  onDropAtIndex,
  dropLinePrefix,
  activeDropLineId,
  onSetActiveDropLineId,
  onDragStart,
  onDragEnd,
}: FieldDropZoneProps) {
  return (
    <div
      className="space-y-2 rounded-md border border-zinc-200 bg-white p-2"
      onDragOver={(event) => {
        event.preventDefault();
        event.dataTransfer.dropEffect = "move";
      }}
      onDrop={onDrop}
    >
      <p className="text-xs font-semibold uppercase tracking-wide text-zinc-500">{title}</p>
      <p className="text-[11px] text-zinc-500">{helperText}</p>

      <div className="max-h-48 space-y-1 overflow-y-auto pr-1">
        {fieldLogicalNames.length > 0 ? (
          <>
            <DropInsertionLine
              lineId={`${dropLinePrefix}-insert-0`}
              activeLineId={activeDropLineId}
              onSetActiveLine={onSetActiveDropLineId}
              onDrop={(event) => onDropAtIndex(event, 0)}
            />
            {fieldLogicalNames.map((logicalName, index) => {
              const label = fieldLabelByLogicalName.get(logicalName) ?? logicalName;
              return (
                <div key={`${title}-${logicalName}-${index}`} className="space-y-1">
                  <div className="rounded-md border border-zinc-200 bg-zinc-50 px-2 py-1">
                    <button
                      type="button"
                      draggable
                      onKeyDown={(event: KeyboardEvent<HTMLButtonElement>) => {
                        if (event.altKey && (event.key === "ArrowUp" || event.key === "ArrowLeft")) {
                          event.preventDefault();
                          onMoveField(logicalName, "up");
                        }
                        if (
                          event.altKey &&
                          (event.key === "ArrowDown" || event.key === "ArrowRight")
                        ) {
                          event.preventDefault();
                          onMoveField(logicalName, "down");
                        }
                      }}
                      onDragStart={(event) => onDragStart(event, logicalName, dragSource)}
                      onDragEnd={onDragEnd}
                      className="w-full text-left"
                    >
                      <p className="text-xs font-medium text-zinc-900">{label}</p>
                    </button>
                    <div className="mt-1 flex flex-wrap gap-1">
                      <Button
                        type="button"
                        size="sm"
                        variant="outline"
                        onClick={() => onMoveField(logicalName, "up")}
                        disabled={index === 0}
                      >
                        Up
                      </Button>
                      <Button
                        type="button"
                        size="sm"
                        variant="outline"
                        onClick={() => onMoveField(logicalName, "down")}
                        disabled={index === fieldLogicalNames.length - 1}
                      >
                        Down
                      </Button>
                      <Button
                        type="button"
                        size="sm"
                        variant="outline"
                        onClick={() => onRemoveField(logicalName)}
                      >
                        Remove
                      </Button>
                    </div>
                  </div>
                  <DropInsertionLine
                    lineId={`${dropLinePrefix}-insert-${index + 1}`}
                    activeLineId={activeDropLineId}
                    onSetActiveLine={onSetActiveDropLineId}
                    onDrop={(event) => onDropAtIndex(event, index + 1)}
                  />
                </div>
              );
            })}
          </>
        ) : (
          <p className="text-[11px] text-zinc-500">Drop fields here.</p>
        )}
      </div>

      <Select
        value=""
        onChange={(event) => {
          const logicalName = event.target.value;
          if (!logicalName) {
            return;
          }
          onAddField(logicalName);
        }}
      >
        <option value="">Add field...</option>
        {Array.from(fieldLabelByLogicalName.keys()).map((logicalName) => (
          <option key={`${title}-select-${logicalName}`} value={logicalName}>
            {fieldLabelByLogicalName.get(logicalName)}
          </option>
        ))}
      </Select>
    </div>
  );
}

type DropInsertionLineProps = {
  lineId?: string;
  activeLineId?: string | null;
  onSetActiveLine?: (value: string | null) => void;
  label?: string;
  onDrop: (event: DragEvent<HTMLDivElement>) => void;
};

function DropInsertionLine({
  lineId,
  activeLineId,
  onSetActiveLine,
  label,
  onDrop,
}: DropInsertionLineProps) {
  const isActive = lineId !== undefined && activeLineId === lineId;
  return (
    <div
      className={`rounded border border-dashed px-2 py-0.5 text-[10px] transition ${isActive ? "border-emerald-400 bg-emerald-100 text-emerald-900" : "border-transparent text-transparent hover:border-emerald-300 hover:bg-emerald-100 hover:text-emerald-800"}`}
      onDragOver={(event) => {
        event.preventDefault();
        if (lineId && onSetActiveLine) {
          onSetActiveLine(lineId);
        }
      }}
      onDragEnter={() => {
        if (lineId && onSetActiveLine) {
          onSetActiveLine(lineId);
        }
      }}
      onDragLeave={() => {
        if (onSetActiveLine) {
          onSetActiveLine(null);
        }
      }}
      onDrop={(event) => {
        if (onSetActiveLine) {
          onSetActiveLine(null);
        }
        onDrop(event);
      }}
      aria-hidden
    >
      {label ?? "Insert here"}
    </div>
  );
}
