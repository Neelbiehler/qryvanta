"use client";

import { useMemo, useState } from "react";
import { useRouter } from "next/navigation";

import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Checkbox,
  Input,
  Label,
  Notice,
  Select,
  StatusBadge,
} from "@qryvanta/ui";

import {
  apiFetch,
  type CreateViewRequest,
  type PublishedSchemaResponse,
  type RuntimeRecordResponse,
  type ViewResponse,
} from "@/lib/api";

type ViewDesignerPanelProps = {
  entityLogicalName: string;
  initialView: ViewResponse | null;
  initialViews: ViewResponse[];
  publishedSchema: PublishedSchemaResponse | null;
  initialPreviewRecords: RuntimeRecordResponse[];
};

type SortDirection = "asc" | "desc";
type LogicalMode = "and" | "or";
type FilterOperator = "eq" | "neq" | "gt" | "gte" | "lt" | "lte" | "contains" | "in";

type ViewColumn = {
  field_logical_name: string;
  position: number;
  width: number | null;
  label_override: string | null;
};

type ViewSort = {
  field_logical_name: string;
  direction: SortDirection;
};

type ViewFilterCondition = {
  field_logical_name: string;
  operator: FilterOperator;
  value: string;
};

type ViewFilterGroup = {
  logical_mode: LogicalMode;
  conditions: ViewFilterCondition[];
};

function normalizeColumns(input: unknown[] | undefined): ViewColumn[] {
  if (!Array.isArray(input) || input.length === 0) {
    return [];
  }

  return input
    .map((candidate, index) => {
      const column = (candidate ?? {}) as Partial<ViewColumn>;
      if (typeof column.field_logical_name !== "string") {
        return null;
      }
      return {
        field_logical_name: column.field_logical_name,
        position: typeof column.position === "number" ? column.position : index,
        width: typeof column.width === "number" ? column.width : null,
        label_override:
          typeof column.label_override === "string" ? column.label_override : null,
      } satisfies ViewColumn;
    })
    .filter((value): value is ViewColumn => value !== null);
}

function normalizeDefaultSort(input: unknown): ViewSort | null {
  if (!input || typeof input !== "object") {
    return null;
  }
  const sort = input as Partial<ViewSort>;
  if (
    typeof sort.field_logical_name !== "string" ||
    (sort.direction !== "asc" && sort.direction !== "desc")
  ) {
    return null;
  }

  return {
    field_logical_name: sort.field_logical_name,
    direction: sort.direction,
  };
}

function normalizeFilterGroup(input: unknown): ViewFilterGroup | null {
  if (!input || typeof input !== "object") {
    return null;
  }

  const group = input as Partial<ViewFilterGroup>;
  const logicalMode: LogicalMode = group.logical_mode === "or" ? "or" : "and";
  if (!Array.isArray(group.conditions)) {
    return null;
  }

  const conditions = group.conditions
    .map((candidate) => {
      const condition = (candidate ?? {}) as Partial<ViewFilterCondition>;
      if (
        typeof condition.field_logical_name !== "string" ||
        typeof condition.operator !== "string"
      ) {
        return null;
      }
      return {
        field_logical_name: condition.field_logical_name,
        operator: condition.operator as FilterOperator,
        value:
          typeof condition.value === "string"
            ? condition.value
            : JSON.stringify(condition.value ?? ""),
      } satisfies ViewFilterCondition;
    })
    .filter((value): value is ViewFilterCondition => value !== null);

  return { logical_mode: logicalMode, conditions };
}

function parseFilterValue(value: string): unknown {
  const trimmed = value.trim();
  if (!trimmed) {
    return "";
  }
  if (trimmed === "true") {
    return true;
  }
  if (trimmed === "false") {
    return false;
  }
  if (!Number.isNaN(Number(trimmed))) {
    return Number(trimmed);
  }
  if (
    (trimmed.startsWith("{") && trimmed.endsWith("}")) ||
    (trimmed.startsWith("[") && trimmed.endsWith("]")) ||
    (trimmed.startsWith('"') && trimmed.endsWith('"'))
  ) {
    try {
      return JSON.parse(trimmed);
    } catch {
      return trimmed;
    }
  }
  return trimmed;
}

function compareValues(left: unknown, right: unknown, direction: SortDirection): number {
  let result = 0;
  if (typeof left === "number" && typeof right === "number") {
    result = left - right;
  } else {
    result = String(left ?? "").localeCompare(String(right ?? ""));
  }
  return direction === "asc" ? result : result * -1;
}

export function ViewDesignerPanel({
  entityLogicalName,
  initialView,
  initialViews,
  publishedSchema,
  initialPreviewRecords,
}: ViewDesignerPanelProps) {
  const router = useRouter();
  const isEditMode = initialView !== null;
  const [logicalName, setLogicalName] = useState(initialView?.logical_name ?? "main_view");
  const [displayName, setDisplayName] = useState(initialView?.display_name ?? "Main View");
  const [viewType, setViewType] = useState(initialView?.view_type ?? "grid");
  const [isDefault, setIsDefault] = useState(initialView?.is_default ?? false);
  const [columns, setColumns] = useState<ViewColumn[]>(() =>
    normalizeColumns(initialView?.columns),
  );
  const [defaultSort, setDefaultSort] = useState<ViewSort | null>(() =>
    normalizeDefaultSort(initialView?.default_sort),
  );
  const [filterGroup, setFilterGroup] = useState<ViewFilterGroup | null>(() =>
    normalizeFilterGroup(initialView?.filter_criteria),
  );
  const [paletteQuery, setPaletteQuery] = useState("");
  const [isSaving, setIsSaving] = useState(false);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const publishedFields = useMemo(
    () => publishedSchema?.fields ?? [],
    [publishedSchema],
  );
  const hasPublishedSchema = publishedSchema !== null;

  const filteredPaletteFields = useMemo(() => {
    const query = paletteQuery.trim().toLowerCase();
    if (!query) {
      return publishedFields;
    }
    return publishedFields.filter((field) => {
      const haystack =
        `${field.logical_name} ${field.display_name} ${field.field_type}`.toLowerCase();
      return haystack.includes(query);
    });
  }, [paletteQuery, publishedFields]);

  const selectedColumnNames = useMemo(
    () => new Set(columns.map((column) => column.field_logical_name)),
    [columns],
  );

  const previewRows = useMemo(() => {
    const filtered = initialPreviewRecords.filter((record) => {
      if (!filterGroup || filterGroup.conditions.length === 0) {
        return true;
      }

      const matches = filterGroup.conditions.map((condition) => {
        const recordValue =
          record.data && typeof record.data === "object"
            ? (record.data as Record<string, unknown>)[condition.field_logical_name]
            : undefined;
        const expected = parseFilterValue(condition.value);

        switch (condition.operator) {
          case "eq":
            return recordValue === expected;
          case "neq":
            return recordValue !== expected;
          case "gt":
            return Number(recordValue ?? 0) > Number(expected ?? 0);
          case "gte":
            return Number(recordValue ?? 0) >= Number(expected ?? 0);
          case "lt":
            return Number(recordValue ?? 0) < Number(expected ?? 0);
          case "lte":
            return Number(recordValue ?? 0) <= Number(expected ?? 0);
          case "contains":
            return String(recordValue ?? "").toLowerCase().includes(String(expected).toLowerCase());
          case "in":
            return Array.isArray(expected) ? expected.includes(recordValue) : false;
          default:
            return false;
        }
      });

      return filterGroup.logical_mode === "and"
        ? matches.every(Boolean)
        : matches.some(Boolean);
    });

    const sorted = [...filtered];
    if (defaultSort) {
      sorted.sort((left, right) => {
        const leftValue = (left.data as Record<string, unknown>)[defaultSort.field_logical_name];
        const rightValue = (right.data as Record<string, unknown>)[defaultSort.field_logical_name];
        return compareValues(leftValue, rightValue, defaultSort.direction);
      });
    }

    return sorted.slice(0, 5);
  }, [defaultSort, filterGroup, initialPreviewRecords]);

  function addColumn(fieldLogicalName: string): void {
    if (selectedColumnNames.has(fieldLogicalName)) {
      return;
    }
    setColumns((current) => [
      ...current,
      {
        field_logical_name: fieldLogicalName,
        position: current.length,
        width: null,
        label_override: null,
      },
    ]);
  }

  function removeColumn(index: number): void {
    setColumns((current) =>
      current
        .filter((_, currentIndex) => currentIndex !== index)
        .map((column, currentIndex) => ({ ...column, position: currentIndex })),
    );
  }

  function moveColumn(index: number, direction: -1 | 1): void {
    setColumns((current) => {
      const targetIndex = index + direction;
      if (targetIndex < 0 || targetIndex >= current.length) {
        return current;
      }
      const next = [...current];
      const [moved] = next.splice(index, 1);
      next.splice(targetIndex, 0, moved);
      return next.map((column, currentIndex) => ({ ...column, position: currentIndex }));
    });
  }

  async function handleSave(): Promise<void> {
    setStatusMessage(null);
    setErrorMessage(null);

    if (!hasPublishedSchema) {
      setErrorMessage("Publish the entity schema before saving views.");
      return;
    }

    if (columns.length === 0) {
      setErrorMessage("Add at least one column before saving.");
      return;
    }

    setIsSaving(true);
    try {
      const payload: CreateViewRequest = {
        logical_name: logicalName,
        display_name: displayName,
        view_type: viewType,
        columns: columns as unknown[],
        default_sort: defaultSort as unknown | null,
        filter_criteria:
          filterGroup && filterGroup.conditions.length > 0 ? (filterGroup as unknown) : null,
        is_default: isDefault,
      };
      const path = isEditMode
        ? `/api/entities/${entityLogicalName}/views/${initialView.logical_name}`
        : `/api/entities/${entityLogicalName}/views`;
      const response = await apiFetch(path, {
        method: isEditMode ? "PUT" : "POST",
        body: JSON.stringify(payload),
      });

      if (!response.ok) {
        const payload = (await response.json()) as { message?: string };
        setErrorMessage(payload.message ?? "Unable to save view.");
        return;
      }

      setStatusMessage("View saved.");
      if (!isEditMode) {
        router.replace(
          `/maker/entities/${encodeURIComponent(entityLogicalName)}/views/${encodeURIComponent(logicalName)}`,
        );
      } else {
        router.refresh();
      }
    } catch {
      setErrorMessage("Unable to save view.");
    } finally {
      setIsSaving(false);
    }
  }

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
          <div className="space-y-2">
            <CardTitle>{isEditMode ? "View Designer" : "New View"}</CardTitle>
            <CardDescription>
              Configure columns, default sort, and query filters for entity record lists.
            </CardDescription>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <StatusBadge tone="neutral">Views {initialViews.length}</StatusBadge>
            <StatusBadge tone={hasPublishedSchema ? "success" : "warning"}>
              {hasPublishedSchema ? "Published schema ready" : "Publish required"}
            </StatusBadge>
            <Button type="button" disabled={isSaving} onClick={handleSave}>
              {isSaving ? "Saving..." : "Save View"}
            </Button>
          </div>
        </CardHeader>
        <CardContent className="grid gap-3 md:grid-cols-4">
          <div className="space-y-2">
            <Label htmlFor="view_logical_name">Logical Name</Label>
            <Input
              id="view_logical_name"
              value={logicalName}
              onChange={(event) => setLogicalName(event.target.value)}
              disabled={isEditMode}
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="view_display_name">Display Name</Label>
            <Input
              id="view_display_name"
              value={displayName}
              onChange={(event) => setDisplayName(event.target.value)}
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="view_type">View Type</Label>
            <Select id="view_type" value={viewType} onChange={(event) => setViewType(event.target.value)}>
              <option value="grid">Grid</option>
              <option value="card">Card</option>
            </Select>
          </div>
          <div className="flex items-end gap-2">
            <Checkbox
              id="view_is_default"
              checked={isDefault}
              onChange={(event) => setIsDefault(event.target.checked)}
            />
            <Label htmlFor="view_is_default">Default View</Label>
          </div>
        </CardContent>
      </Card>

      <div className="grid gap-4 xl:grid-cols-[280px_1fr]">
        <Card className="h-fit">
          <CardHeader>
            <CardTitle className="text-base">Available Fields</CardTitle>
            <CardDescription>Add fields as active columns for this view.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-2">
            <Input
              value={paletteQuery}
              onChange={(event) => setPaletteQuery(event.target.value)}
              placeholder="Search fields"
            />
            <div className="max-h-[420px] space-y-2 overflow-y-auto">
              {filteredPaletteFields.map((field) => (
                <div
                  key={field.logical_name}
                  className="flex items-center justify-between rounded-md border border-zinc-200 px-3 py-2"
                >
                  <div>
                    <p className="text-sm font-medium">{field.display_name}</p>
                    <p className="font-mono text-xs text-zinc-500">{field.logical_name}</p>
                  </div>
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    disabled={selectedColumnNames.has(field.logical_name)}
                    onClick={() => addColumn(field.logical_name)}
                  >
                    {selectedColumnNames.has(field.logical_name) ? "Added" : "Add"}
                  </Button>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>

        <div className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="text-base">Active Columns</CardTitle>
              <CardDescription>Reorder and configure widths/labels for each column.</CardDescription>
            </CardHeader>
            <CardContent className="space-y-2">
              {columns.length > 0 ? (
                columns.map((column, index) => (
                  <div
                    key={`${column.field_logical_name}-${index}`}
                    className="grid gap-2 rounded-md border border-zinc-200 p-3 md:grid-cols-[1fr_120px_1fr_auto]"
                  >
                    <Input
                      value={column.field_logical_name}
                      onChange={(event) =>
                        setColumns((current) =>
                          current.map((candidate, candidateIndex) =>
                            candidateIndex === index
                              ? { ...candidate, field_logical_name: event.target.value }
                              : candidate,
                          ),
                        )
                      }
                    />
                    <Input
                      type="number"
                      min={60}
                      placeholder="Width"
                      value={column.width ?? ""}
                      onChange={(event) =>
                        setColumns((current) =>
                          current.map((candidate, candidateIndex) =>
                            candidateIndex === index
                              ? {
                                  ...candidate,
                                  width:
                                    event.target.value.trim().length === 0
                                      ? null
                                      : Number.parseInt(event.target.value, 10),
                                }
                              : candidate,
                          ),
                        )
                      }
                    />
                    <Input
                      placeholder="Label override"
                      value={column.label_override ?? ""}
                      onChange={(event) =>
                        setColumns((current) =>
                          current.map((candidate, candidateIndex) =>
                            candidateIndex === index
                              ? {
                                  ...candidate,
                                  label_override:
                                    event.target.value.trim().length === 0
                                      ? null
                                      : event.target.value,
                                }
                              : candidate,
                          ),
                        )
                      }
                    />
                    <div className="flex items-center gap-1">
                      <Button type="button" size="sm" variant="outline" onClick={() => moveColumn(index, -1)}>
                        Up
                      </Button>
                      <Button type="button" size="sm" variant="outline" onClick={() => moveColumn(index, 1)}>
                        Down
                      </Button>
                      <Button type="button" size="sm" variant="ghost" onClick={() => removeColumn(index)}>
                        Remove
                      </Button>
                    </div>
                  </div>
                ))
              ) : (
                <p className="text-sm text-zinc-500">No columns configured.</p>
              )}
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="text-base">Sort and Filter</CardTitle>
              <CardDescription>Choose default sort and define filter rules.</CardDescription>
            </CardHeader>
            <CardContent className="space-y-3">
              <div className="grid gap-2 md:grid-cols-2">
                <div className="space-y-2">
                  <Label htmlFor="default_sort_field">Default Sort Field</Label>
                  <Select
                    id="default_sort_field"
                    value={defaultSort?.field_logical_name ?? ""}
                    onChange={(event) => {
                      const field = event.target.value;
                      if (!field) {
                        setDefaultSort(null);
                        return;
                      }
                      setDefaultSort({
                        field_logical_name: field,
                        direction: defaultSort?.direction ?? "asc",
                      });
                    }}
                  >
                    <option value="">None</option>
                    {publishedFields.map((field) => (
                      <option key={field.logical_name} value={field.logical_name}>
                        {field.display_name}
                      </option>
                    ))}
                  </Select>
                </div>
                <div className="space-y-2">
                  <Label htmlFor="default_sort_direction">Direction</Label>
                  <Select
                    id="default_sort_direction"
                    value={defaultSort?.direction ?? "asc"}
                    onChange={(event) =>
                      setDefaultSort((current) =>
                        current
                          ? {
                              ...current,
                              direction: event.target.value as SortDirection,
                            }
                          : null,
                      )
                    }
                    disabled={!defaultSort}
                  >
                    <option value="asc">Ascending</option>
                    <option value="desc">Descending</option>
                  </Select>
                </div>
              </div>

              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <Label htmlFor="filter_logical_mode">Filter Logical Mode</Label>
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    onClick={() =>
                      setFilterGroup((current) => ({
                        logical_mode: current?.logical_mode ?? "and",
                        conditions: [
                          ...(current?.conditions ?? []),
                          {
                            field_logical_name: publishedFields[0]?.logical_name ?? "",
                            operator: "eq",
                            value: "",
                          },
                        ],
                      }))
                    }
                  >
                    Add Rule
                  </Button>
                </div>
                <Select
                  id="filter_logical_mode"
                  value={filterGroup?.logical_mode ?? "and"}
                  onChange={(event) =>
                    setFilterGroup((current) => ({
                      logical_mode: event.target.value as LogicalMode,
                      conditions: current?.conditions ?? [],
                    }))
                  }
                >
                  <option value="and">AND</option>
                  <option value="or">OR</option>
                </Select>
              </div>

              {(filterGroup?.conditions ?? []).map((condition, index) => (
                <div key={`${condition.field_logical_name}-${index}`} className="grid gap-2 md:grid-cols-[1fr_120px_1fr_auto]">
                  <Select
                    value={condition.field_logical_name}
                    onChange={(event) =>
                      setFilterGroup((current) =>
                        current
                          ? {
                              ...current,
                              conditions: current.conditions.map((candidate, candidateIndex) =>
                                candidateIndex === index
                                  ? { ...candidate, field_logical_name: event.target.value }
                                  : candidate,
                              ),
                            }
                          : current,
                      )
                    }
                  >
                    {publishedFields.map((field) => (
                      <option key={field.logical_name} value={field.logical_name}>
                        {field.display_name}
                      </option>
                    ))}
                  </Select>
                  <Select
                    value={condition.operator}
                    onChange={(event) =>
                      setFilterGroup((current) =>
                        current
                          ? {
                              ...current,
                              conditions: current.conditions.map((candidate, candidateIndex) =>
                                candidateIndex === index
                                  ? { ...candidate, operator: event.target.value as FilterOperator }
                                  : candidate,
                              ),
                            }
                          : current,
                      )
                    }
                  >
                    <option value="eq">eq</option>
                    <option value="neq">neq</option>
                    <option value="gt">gt</option>
                    <option value="gte">gte</option>
                    <option value="lt">lt</option>
                    <option value="lte">lte</option>
                    <option value="contains">contains</option>
                    <option value="in">in</option>
                  </Select>
                  <Input
                    value={condition.value}
                    onChange={(event) =>
                      setFilterGroup((current) =>
                        current
                          ? {
                              ...current,
                              conditions: current.conditions.map((candidate, candidateIndex) =>
                                candidateIndex === index
                                  ? { ...candidate, value: event.target.value }
                                  : candidate,
                              ),
                            }
                          : current,
                      )
                    }
                    placeholder='Value, e.g. "active" or 5'
                  />
                  <Button
                    type="button"
                    variant="ghost"
                    onClick={() =>
                      setFilterGroup((current) =>
                        current
                          ? {
                              ...current,
                              conditions: current.conditions.filter((_, i) => i !== index),
                            }
                          : current,
                      )
                    }
                  >
                    Remove
                  </Button>
                </div>
              ))}
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="text-base">Live Preview</CardTitle>
              <CardDescription>
                Preview first 5 records with current column/sort/filter settings.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-2">
              {previewRows.length > 0 && columns.length > 0 ? (
                <div className="overflow-x-auto rounded-md border border-zinc-200">
                  <table className="w-full text-sm">
                    <thead className="bg-zinc-50">
                      <tr>
                        {columns.map((column) => {
                          const field = publishedFields.find(
                            (candidate) => candidate.logical_name === column.field_logical_name,
                          );
                          return (
                            <th
                              key={`header-${column.field_logical_name}`}
                              className="border-b border-zinc-200 px-3 py-2 text-left font-semibold text-zinc-700"
                              style={column.width ? { width: `${column.width}px` } : undefined}
                            >
                              {column.label_override?.trim() || field?.display_name || column.field_logical_name}
                            </th>
                          );
                        })}
                      </tr>
                    </thead>
                    <tbody>
                      {previewRows.map((record) => (
                        <tr key={record.record_id} className="border-b border-zinc-100">
                          {columns.map((column) => (
                            <td key={`${record.record_id}-${column.field_logical_name}`} className="px-3 py-2 align-top">
                              {JSON.stringify(
                                (record.data as Record<string, unknown>)[column.field_logical_name] ?? null,
                              )}
                            </td>
                          ))}
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              ) : (
                <p className="text-sm text-zinc-500">
                  {columns.length === 0
                    ? "Add columns to render preview."
                    : "No preview records available for current filter."}
                </p>
              )}
            </CardContent>
          </Card>
        </div>
      </div>

      {!hasPublishedSchema ? (
        <Notice tone="warning">
          This entity does not have a published schema yet. Publish the entity before saving view definitions.
        </Notice>
      ) : null}
      {errorMessage ? <Notice tone="error">{errorMessage}</Notice> : null}
      {statusMessage ? <Notice tone="success">{statusMessage}</Notice> : null}
    </div>
  );
}

