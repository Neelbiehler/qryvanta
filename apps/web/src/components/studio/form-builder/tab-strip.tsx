"use client";

import type { FormTab } from "@/components/studio/types";

type TabStripProps = {
  tabs: FormTab[];
  activeTabIndex: number;
  onSelectTab: (index: number) => void;
  onReorderTabs: (sourceIndex: number, targetIndex: number) => void;
};

export function TabStrip({
  tabs,
  activeTabIndex,
  onSelectTab,
  onReorderTabs,
}: TabStripProps) {
  return (
    <div className="flex flex-wrap gap-1 border-b border-zinc-200 px-4 py-2">
      {tabs.map((tab, index) => (
        <button
          key={tab.logical_name}
          type="button"
          className={
            index === activeTabIndex
              ? "rounded-md bg-zinc-900 px-3 py-1 text-sm font-medium text-white"
              : "rounded-md border border-zinc-200 px-3 py-1 text-sm text-zinc-700 hover:bg-zinc-100"
          }
          onClick={() => onSelectTab(index)}
          draggable
          onDragStart={(event) => {
            event.dataTransfer.setData("text/tab-index", String(index));
          }}
          onDragOver={(event) => event.preventDefault()}
          onDrop={(event) => {
            const sourceIndex = Number.parseInt(
              event.dataTransfer.getData("text/tab-index"),
              10,
            );
            if (Number.isNaN(sourceIndex) || sourceIndex === index) return;
            onReorderTabs(sourceIndex, index);
          }}
        >
          {tab.display_name}
        </button>
      ))}
    </div>
  );
}
