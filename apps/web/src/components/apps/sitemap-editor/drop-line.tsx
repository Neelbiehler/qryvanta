import type { DragEvent } from "react";

type DropLineProps = {
  lineId: string;
  activeLineId: string | null;
  onSetActiveLine: (lineId: string | null) => void;
  label?: string;
  onDrop: (event: DragEvent<HTMLDivElement>) => void;
};

export function DropLine({
  lineId,
  activeLineId,
  onSetActiveLine,
  label,
  onDrop,
}: DropLineProps) {
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
