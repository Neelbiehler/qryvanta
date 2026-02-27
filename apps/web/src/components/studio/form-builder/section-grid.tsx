"use client";

import { type DragEvent, type ReactNode } from "react";

type SectionGridProps = {
  columns: 1 | 2 | 3;
  children: ReactNode;
};

export function SectionGrid({ columns, children }: SectionGridProps) {
  return (
    <div
      className={
        columns === 1
          ? "grid gap-3"
          : columns === 2
            ? "grid gap-3 md:grid-cols-2"
            : "grid gap-3 md:grid-cols-[2fr_1fr_1fr]"
      }
    >
      {children}
    </div>
  );
}

type SectionColumnDropZoneProps = {
  title: string;
  onDropField: (event: DragEvent<HTMLDivElement>) => void;
  children: ReactNode;
};

export function SectionColumnDropZone({
  title,
  onDropField,
  children,
}: SectionColumnDropZoneProps) {
  return (
    <div
      className="min-h-24 rounded-md border border-dashed border-zinc-300 bg-white p-2"
      onDragOver={(event) => event.preventDefault()}
      onDrop={(event) => {
        event.preventDefault();
        onDropField(event);
      }}
    >
      <p className="mb-2 text-[10px] font-semibold uppercase tracking-[0.14em] text-zinc-500">
        {title}
      </p>
      <div className="space-y-1.5">{children}</div>
    </div>
  );
}

type InsertDropLineProps = {
  lineId: string;
  activeLineId: string | null;
  onSetActiveLine: (lineId: string | null) => void;
  onDrop: (event: DragEvent<HTMLDivElement>) => void;
};

export function InsertDropLine({
  lineId,
  activeLineId,
  onSetActiveLine,
  onDrop,
}: InsertDropLineProps) {
  const isActive = activeLineId === lineId;

  return (
    <div
      className={`rounded border border-dashed px-2 py-0.5 text-[10px] transition ${
        isActive
          ? "border-emerald-400 bg-emerald-100 text-emerald-900"
          : "border-transparent text-transparent hover:border-emerald-300 hover:bg-emerald-100 hover:text-emerald-800"
      }`}
      onDragOver={(event) => {
        event.preventDefault();
        onSetActiveLine(lineId);
      }}
      onDragEnter={() => onSetActiveLine(lineId)}
      onDragLeave={() => onSetActiveLine(null)}
      onDrop={(event) => {
        event.preventDefault();
        onSetActiveLine(null);
        onDrop(event);
      }}
      aria-hidden
    >
      Insert here
    </div>
  );
}
