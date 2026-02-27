"use client";

import { type CSSProperties, type ReactNode, useEffect, useState } from "react";
import { Menu, PanelLeftClose, PanelLeftOpen, X } from "lucide-react";

type WorkerSplitShellProps = {
  sidebar: ReactNode;
  content: ReactNode;
  storageKey: string;
  defaultSidebarWidth?: number;
};

export function WorkerSplitShell({
  sidebar,
  content,
  storageKey,
  defaultSidebarWidth = 300,
}: WorkerSplitShellProps) {
  const [sidebarWidth, setSidebarWidth] = useState<number>(defaultSidebarWidth);
  const [isSidebarCollapsed, setIsSidebarCollapsed] = useState(false);
  const [isMobileSidebarOpen, setIsMobileSidebarOpen] = useState(false);
  const collapsedStorageKey = `${storageKey}_collapsed`;

  useEffect(() => {
    try {
      const raw = localStorage.getItem(storageKey);
      const parsed = raw ? Number.parseInt(raw, 10) : Number.NaN;
      if (Number.isNaN(parsed)) return;
      const nextWidth = clampWidth(parsed);
      queueMicrotask(() => {
        setSidebarWidth(nextWidth);
      });
    } catch {
      // ignore persistence errors
    }
  }, [storageKey]);

  useEffect(() => {
    try {
      const raw = localStorage.getItem(collapsedStorageKey);
      if (!raw) return;
      const parsed = raw === "1";
      queueMicrotask(() => {
        setIsSidebarCollapsed(parsed);
      });
    } catch {
      // ignore persistence errors
    }
  }, [collapsedStorageKey]);

  useEffect(() => {
    try {
      localStorage.setItem(storageKey, String(sidebarWidth));
    } catch {
      // ignore persistence errors
    }
  }, [sidebarWidth, storageKey]);

  useEffect(() => {
    try {
      localStorage.setItem(collapsedStorageKey, isSidebarCollapsed ? "1" : "0");
    } catch {
      // ignore persistence errors
    }
  }, [collapsedStorageKey, isSidebarCollapsed]);

  function handleDragStart(event: React.MouseEvent<HTMLDivElement>) {
    if (isSidebarCollapsed) return;
    event.preventDefault();
    const startX = event.clientX;
    const startWidth = sidebarWidth;

    function onMove(moveEvent: MouseEvent) {
      const deltaX = moveEvent.clientX - startX;
      setSidebarWidth(clampWidth(startWidth + deltaX));
    }

    function onUp() {
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    }

    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
  }

  const effectiveSidebarWidth = isSidebarCollapsed ? 48 : sidebarWidth;

  return (
    <>
      {/* Mobile toggle button */}
      <button
        type="button"
        className="fixed left-3 top-[4.25rem] z-30 rounded-md border border-emerald-200 bg-white p-2 text-emerald-700 shadow-sm xl:hidden"
        onClick={() => setIsMobileSidebarOpen(true)}
        aria-label="Open app navigation"
      >
        <Menu aria-hidden="true" className="h-4 w-4" />
      </button>

      {/* Mobile sidebar overlay */}
      {isMobileSidebarOpen ? (
        <>
          <button
            type="button"
            className="fixed inset-0 z-40 bg-zinc-900/40 xl:hidden"
            aria-label="Close app navigation"
            onClick={() => setIsMobileSidebarOpen(false)}
          />
          <aside className="fixed inset-y-0 left-0 z-50 w-[min(88vw,340px)] border-r border-emerald-100 bg-white shadow-xl xl:hidden">
            <div className="flex items-center justify-end border-b border-emerald-100 bg-emerald-50 p-2">
              <button
                type="button"
                className="rounded-md border border-emerald-200 bg-white p-1.5 text-emerald-700"
                onClick={() => setIsMobileSidebarOpen(false)}
                aria-label="Close app navigation"
              >
                <X aria-hidden="true" className="h-4 w-4" />
              </button>
            </div>
            <div className="h-[calc(100%-3rem)]">{sidebar}</div>
          </aside>
        </>
      ) : null}

      {/* Desktop split layout */}
      <div
        className="grid h-full min-h-0 grid-cols-1 bg-zinc-50 xl:[grid-template-columns:var(--worker-sidebar-width)_6px_minmax(0,1fr)]"
        style={{ "--worker-sidebar-width": `${effectiveSidebarWidth}px` } as CSSProperties}
      >
        {/* Sidebar panel */}
        <div className="hidden min-h-0 xl:block">
          {isSidebarCollapsed ? (
            <div className="flex h-full flex-col items-center border-r border-emerald-100 bg-white pt-3">
              <button
                type="button"
                className="rounded-md border border-emerald-200 bg-white p-1.5 text-emerald-700 hover:bg-emerald-50"
                onClick={() => setIsSidebarCollapsed(false)}
                aria-label="Expand sidebar"
              >
                <PanelLeftOpen aria-hidden="true" className="h-4 w-4" />
              </button>
            </div>
          ) : (
            sidebar
          )}
        </div>

        {/* Resize handle */}
        <div
          className={`relative hidden xl:block ${
            isSidebarCollapsed
              ? "cursor-pointer bg-emerald-50"
              : "cursor-col-resize bg-[var(--split-handle-bg,#d8ebdf)] hover:bg-[var(--split-handle-hover,#9dc9b4)] active:bg-[var(--split-handle-active,#2f8f63)]"
          } transition-colors`}
          onMouseDown={handleDragStart}
          role="separator"
          aria-orientation="vertical"
          aria-label="Resize sidebar"
        >
          <button
            type="button"
            className="absolute left-1/2 top-2 z-10 -translate-x-1/2 rounded-md border border-emerald-200 bg-white p-0.5 text-emerald-600 shadow-sm hover:bg-emerald-50"
            aria-label={isSidebarCollapsed ? "Expand sidebar" : "Collapse sidebar"}
            onClick={(event) => {
              event.stopPropagation();
              setIsSidebarCollapsed((current) => !current);
            }}
          >
            {isSidebarCollapsed ? (
              <PanelLeftOpen aria-hidden="true" className="h-3 w-3" />
            ) : (
              <PanelLeftClose aria-hidden="true" className="h-3 w-3" />
            )}
          </button>
          {isSidebarCollapsed ? null : (
            <span className="pointer-events-none absolute inset-y-1/2 left-1/2 h-8 w-[3px] -translate-x-1/2 -translate-y-1/2 rounded-full bg-[var(--split-handle-hover,#9dc9b4)]" />
          )}
        </div>

        {/* Content panel */}
        <div className="min-h-0 bg-zinc-50">{content}</div>
      </div>
    </>
  );
}

function clampWidth(width: number): number {
  return Math.max(240, Math.min(460, width));
}
