"use client";

import { useCallback, useMemo, useState } from "react";

import type {
  FilterOperator,
  LogicalMode,
  SortDirection,
  ViewColumn,
  ViewFilterGroup,
  ViewSort,
} from "@/components/studio/types";
import type { FieldResponse, RuntimeRecordResponse } from "@/lib/api";

function normalizeColumns(input: unknown): ViewColumn[] {
  if (!Array.isArray(input)) return [];
  return input
    .map((candidate, index) => {
      const column = (candidate ?? {}) as Partial<ViewColumn>;
      if (typeof column.field_logical_name !== "string") return null;
      return {
        field_logical_name: column.field_logical_name,
        position: typeof column.position === "number" ? column.position : index,
        width: typeof column.width === "number" ? column.width : null,
        label_override:
          typeof column.label_override === "string" ? column.label_override : null,
      } satisfies ViewColumn;
    })
    .filter((value): value is ViewColumn => value !== null)
    .sort((a, b) => a.position - b.position)
    .map((column, index) => ({ ...column, position: index }));
}

function normalizeDefaultSort(input: unknown): ViewSort | null {
  if (!input || typeof input !== "object") return null;
  const sort = input as Partial<ViewSort>;
  if (
    typeof sort.field_logical_name !== "string" ||
    (sort.direction !== "asc" && sort.direction !== "desc")
  ) {
    return null;
  }
  return { field_logical_name: sort.field_logical_name, direction: sort.direction };
}

function normalizeFilterGroup(input: unknown): ViewFilterGroup | null {
  if (!input || typeof input !== "object") return null;
  const group = input as Partial<ViewFilterGroup>;
  if (!Array.isArray(group.conditions)) return null;

  const conditions = group.conditions
    .map((candidate) => {
      const condition = (candidate ?? {}) as Partial<ViewFilterGroup["conditions"][number]>;
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
      };
    })
    .filter((value): value is ViewFilterGroup["conditions"][number] => value !== null);

  return {
    logical_mode: group.logical_mode === "or" ? "or" : "and",
    conditions,
  };
}

function parseFilterValue(value: string): unknown {
  const trimmed = value.trim();
  if (!trimmed) return "";
  if (trimmed === "true") return true;
  if (trimmed === "false") return false;
  if (!Number.isNaN(Number(trimmed))) return Number(trimmed);
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

export type ViewEditorState = {
  columns: ViewColumn[];
  defaultSort: ViewSort | null;
  filterGroup: ViewFilterGroup | null;
  selectedColumnIndex: number | null;
  previewRows: RuntimeRecordResponse[];
  selectedColumnNames: Set<string>;
  resetFromView: (input: {
    columns: unknown;
    default_sort: unknown;
    filter_criteria: unknown;
  }) => void;
  addColumn: (fieldLogicalName: string) => void;
  removeColumn: (index: number) => void;
  reorderColumn: (sourceIndex: number, targetIndex: number) => void;
  setSelectedColumnIndex: (index: number | null) => void;
  updateColumn: (index: number, patch: Partial<ViewColumn>) => void;
  setDefaultSort: (sort: ViewSort | null) => void;
  setFilterGroup: (group: ViewFilterGroup | null) => void;
  addFilterRule: (firstFieldLogicalName: string) => void;
  updateFilterRule: (
    index: number,
    patch: Partial<ViewFilterGroup["conditions"][number]>,
  ) => void;
  removeFilterRule: (index: number) => void;
};

type UseViewEditorStateInput = {
  initialColumns: unknown;
  initialDefaultSort: unknown;
  initialFilterCriteria: unknown;
  previewRecords: RuntimeRecordResponse[];
  publishedFields: FieldResponse[];
};

export function useViewEditorState({
  initialColumns,
  initialDefaultSort,
  initialFilterCriteria,
  previewRecords,
  publishedFields: _publishedFields,
}: UseViewEditorStateInput): ViewEditorState {
  const [columns, setColumns] = useState<ViewColumn[]>(() => normalizeColumns(initialColumns));
  const [defaultSort, setDefaultSort] = useState<ViewSort | null>(() =>
    normalizeDefaultSort(initialDefaultSort),
  );
  const [filterGroup, setFilterGroup] = useState<ViewFilterGroup | null>(() =>
    normalizeFilterGroup(initialFilterCriteria),
  );
  const [selectedColumnIndex, setSelectedColumnIndex] = useState<number | null>(null);

  const selectedColumnNames = useMemo(
    () => new Set(columns.map((column) => column.field_logical_name)),
    [columns],
  );

  const previewRows = useMemo(() => {
    const filtered = previewRecords.filter((record) => {
      if (!filterGroup || filterGroup.conditions.length === 0) return true;
      const matches = filterGroup.conditions.map((condition) => {
        const value = (record.data as Record<string, unknown>)[condition.field_logical_name];
        const expected = parseFilterValue(condition.value);
        switch (condition.operator) {
          case "eq":
            return value === expected;
          case "neq":
            return value !== expected;
          case "gt":
            return Number(value ?? 0) > Number(expected ?? 0);
          case "gte":
            return Number(value ?? 0) >= Number(expected ?? 0);
          case "lt":
            return Number(value ?? 0) < Number(expected ?? 0);
          case "lte":
            return Number(value ?? 0) <= Number(expected ?? 0);
          case "contains":
            return String(value ?? "").toLowerCase().includes(String(expected).toLowerCase());
          case "in":
            return Array.isArray(expected) ? expected.includes(value) : false;
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

    return sorted.slice(0, 30);
  }, [defaultSort, filterGroup, previewRecords]);

  const addColumn = useCallback((fieldLogicalName: string) => {
    setColumns((current) => {
      if (current.some((column) => column.field_logical_name === fieldLogicalName)) {
        return current;
      }
      return [
        ...current,
        {
          field_logical_name: fieldLogicalName,
          position: current.length,
          width: null,
          label_override: null,
        },
      ];
    });
  }, []);

  const removeColumn = useCallback((index: number) => {
    setColumns((current) =>
      current
        .filter((_, currentIndex) => currentIndex !== index)
        .map((column, currentIndex) => ({ ...column, position: currentIndex })),
    );
    setSelectedColumnIndex(null);
  }, []);

  const reorderColumn = useCallback((sourceIndex: number, targetIndex: number) => {
    setColumns((current) => {
      if (
        sourceIndex < 0 ||
        targetIndex < 0 ||
        sourceIndex >= current.length ||
        targetIndex >= current.length
      ) {
        return current;
      }
      const next = [...current];
      const [moved] = next.splice(sourceIndex, 1);
      next.splice(targetIndex, 0, moved);
      return next.map((column, index) => ({ ...column, position: index }));
    });
    setSelectedColumnIndex(targetIndex);
  }, []);

  const updateColumn = useCallback((index: number, patch: Partial<ViewColumn>) => {
    setColumns((current) =>
      current.map((column, currentIndex) =>
        currentIndex === index ? { ...column, ...patch } : column,
      ),
    );
  }, []);

  const addFilterRule = useCallback((firstFieldLogicalName: string) => {
    setFilterGroup((current) => ({
      logical_mode: current?.logical_mode ?? "and",
      conditions: [
        ...(current?.conditions ?? []),
        {
          field_logical_name: firstFieldLogicalName,
          operator: "eq",
          value: "",
        },
      ],
    }));
  }, []);

  const updateFilterRule = useCallback(
    (index: number, patch: Partial<ViewFilterGroup["conditions"][number]>) => {
      setFilterGroup((current) => {
        if (!current) return current;
        return {
          ...current,
          conditions: current.conditions.map((condition, currentIndex) =>
            currentIndex === index ? { ...condition, ...patch } : condition,
          ),
        };
      });
    },
    [],
  );

  const removeFilterRule = useCallback((index: number) => {
    setFilterGroup((current) => {
      if (!current) return current;
      return {
        ...current,
        conditions: current.conditions.filter((_, currentIndex) => currentIndex !== index),
      };
    });
  }, []);

  const resetFromView = useCallback(
    (input: { columns: unknown; default_sort: unknown; filter_criteria: unknown }) => {
      setColumns(normalizeColumns(input.columns));
      setDefaultSort(normalizeDefaultSort(input.default_sort));
      setFilterGroup(normalizeFilterGroup(input.filter_criteria));
      setSelectedColumnIndex(null);
    },
    [],
  );

  return {
    columns,
    defaultSort,
    filterGroup,
    selectedColumnIndex,
    previewRows,
    selectedColumnNames,
    resetFromView,
    addColumn,
    removeColumn,
    reorderColumn,
    setSelectedColumnIndex,
    updateColumn,
    setDefaultSort,
    setFilterGroup,
    addFilterRule,
    updateFilterRule,
    removeFilterRule,
  };
}
