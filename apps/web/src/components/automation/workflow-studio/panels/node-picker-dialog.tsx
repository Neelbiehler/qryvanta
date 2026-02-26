import type { RefObject } from "react";

import { Button, Input, Select } from "@qryvanta/ui";

import type {
  CatalogInsertMode,
  FlowTemplateCategory,
  FlowTemplateId,
} from "@/components/automation/workflow-studio/model";

type TemplateOption = {
  id: FlowTemplateId;
  label: string;
  description: string;
  category: FlowTemplateCategory;
};

type NodePickerDialogProps = {
  open: boolean;
  inputRef: RefObject<HTMLInputElement | null>;
  query: string;
  category: "all" | FlowTemplateCategory;
  insertMode: CatalogInsertMode;
  canInsertIntoConditionBranch: boolean;
  templates: TemplateOption[];
  onQueryChange: (query: string) => void;
  onCategoryChange: (category: "all" | FlowTemplateCategory) => void;
  onInsertModeChange: (mode: CatalogInsertMode) => void;
  onInsert: (templateId: FlowTemplateId) => void;
  onClose: () => void;
};

export function NodePickerDialog({
  open,
  inputRef,
  query,
  category,
  insertMode,
  canInsertIntoConditionBranch,
  templates,
  onQueryChange,
  onCategoryChange,
  onInsertModeChange,
  onInsert,
  onClose,
}: NodePickerDialogProps) {
  if (!open) {
    return null;
  }

  return (
    <div
      className="absolute inset-0 z-50 flex items-start justify-center bg-zinc-900/35 pt-24"
      onClick={onClose}
    >
      <div
        className="w-[min(760px,calc(100%-2rem))] rounded-xl border border-zinc-200 bg-white shadow-2xl"
        onClick={(event) => event.stopPropagation()}
      >
        <div className="border-b border-zinc-200 p-3">
          <p className="text-xs font-semibold uppercase tracking-wide text-zinc-600">
            Node Picker
          </p>
          <Input
            ref={inputRef}
            value={query}
            onChange={(event) => onQueryChange(event.target.value)}
            placeholder="Search functions..."
          />
          <div className="mt-2 grid grid-cols-2 gap-2">
            <Select
              value={category}
              onChange={(event) =>
                onCategoryChange(event.target.value as "all" | FlowTemplateCategory)
              }
            >
              <option value="all">All</option>
              <option value="trigger">Trigger</option>
              <option value="logic">Logic</option>
              <option value="integration">Integration</option>
              <option value="data">Data</option>
              <option value="operations">Operations</option>
            </Select>

            <Select
              value={insertMode}
              onChange={(event) =>
                onInsertModeChange(event.target.value as CatalogInsertMode)
              }
            >
              <option value="after_selected">After selected</option>
              <option value="root">Append root</option>
              <option
                value="then_selected"
                disabled={!canInsertIntoConditionBranch}
              >
                Condition: yes
              </option>
              <option
                value="else_selected"
                disabled={!canInsertIntoConditionBranch}
              >
                Condition: no
              </option>
            </Select>
          </div>
        </div>

        <div className="max-h-[420px] space-y-2 overflow-y-auto p-3">
          {templates.length > 0 ? (
            templates.map((template, index) => (
              <button
                key={template.id}
                type="button"
                className={`w-full rounded-md border px-3 py-2 text-left transition ${
                  index === 0
                    ? "border-emerald-400 bg-emerald-50"
                    : "border-zinc-200 bg-white hover:border-emerald-300"
                }`}
                onClick={() => onInsert(template.id)}
              >
                <p className="text-sm font-semibold text-zinc-900">
                  {template.label}
                  <span className="ml-2 text-[10px] uppercase tracking-wide text-zinc-500">
                    {template.category}
                  </span>
                </p>
                <p className="text-xs text-zinc-600">{template.description}</p>
              </button>
            ))
          ) : (
            <p className="text-sm text-zinc-500">No functions match your search.</p>
          )}
        </div>

        <div className="flex items-center justify-between border-t border-zinc-200 px-3 py-2 text-xs text-zinc-600">
          <span>`A` open picker • `Enter` insert top match • `Esc` close</span>
          <Button type="button" size="sm" variant="outline" onClick={onClose}>
            Close
          </Button>
        </div>
      </div>
    </div>
  );
}
