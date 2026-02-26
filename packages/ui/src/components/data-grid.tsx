"use client";

import * as React from "react";

import { Checkbox } from "./checkbox";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "./table";
import { cn } from "../lib/cn";

export type DataGridSortDirection = "asc" | "desc";

export type DataGridSortState = {
  key: string;
  direction: DataGridSortDirection;
};

export type DataGridColumn<T> = {
  key: keyof T | string;
  header: React.ReactNode;
  width?: number | string;
  sortable?: boolean;
  pin?: "left" | "right";
  className?: string;
  headerClassName?: string;
  cell?: (row: T, rowIndex: number) => React.ReactNode;
};

export type DataGridProps<T> = {
  columns: Array<DataGridColumn<T>>;
  rows: T[];
  getRowId: (row: T, rowIndex: number) => string;
  selection?: Set<string>;
  defaultSelection?: Set<string>;
  onSelectionChange?: (selection: Set<string>) => void;
  sortState?: DataGridSortState | null;
  defaultSortState?: DataGridSortState | null;
  onSortStateChange?: (sortState: DataGridSortState | null) => void;
  onRowClick?: (row: T, rowIndex: number) => void;
  loading?: boolean;
  emptyState?: React.ReactNode;
  className?: string;
  rowClassName?: (row: T, rowIndex: number) => string | undefined;
};

function normalizeValue(value: unknown): number | string {
  if (typeof value === "number") {
    return value;
  }

  if (typeof value === "string") {
    return value.toLowerCase();
  }

  if (value === null || value === undefined) {
    return "";
  }

  return String(value).toLowerCase();
}

export function DataGrid<T>({
  columns,
  rows,
  getRowId,
  selection,
  defaultSelection,
  onSelectionChange,
  sortState,
  defaultSortState,
  onSortStateChange,
  onRowClick,
  loading = false,
  emptyState,
  className,
  rowClassName,
}: DataGridProps<T>) {
  const [internalSelection, setInternalSelection] = React.useState<Set<string>>(
    defaultSelection ?? new Set<string>(),
  );
  const [internalSortState, setInternalSortState] = React.useState<DataGridSortState | null>(
    defaultSortState ?? null,
  );

  const resolvedSelection = selection ?? internalSelection;
  const resolvedSortState = sortState ?? internalSortState;

  const setSelection = React.useCallback(
    (nextSelection: Set<string>) => {
      if (selection === undefined) {
        setInternalSelection(nextSelection);
      }
      onSelectionChange?.(nextSelection);
    },
    [onSelectionChange, selection],
  );

  const setSortState = React.useCallback(
    (nextSortState: DataGridSortState | null) => {
      if (sortState === undefined) {
        setInternalSortState(nextSortState);
      }
      onSortStateChange?.(nextSortState);
    },
    [onSortStateChange, sortState],
  );

  const rowDescriptors = React.useMemo(
    () => rows.map((row, index) => ({ id: getRowId(row, index), row, index })),
    [getRowId, rows],
  );

  const sortedRows = React.useMemo(() => {
    if (!resolvedSortState) {
      return rowDescriptors;
    }

    const targetColumn = columns.find((column) => String(column.key) === resolvedSortState.key);
    if (!targetColumn) {
      return rowDescriptors;
    }

    const getColumnValue = (descriptor: { row: T; index: number }) => {
      if (targetColumn.cell) {
        return normalizeValue(targetColumn.cell(descriptor.row, descriptor.index));
      }

      return normalizeValue(descriptor.row[targetColumn.key as keyof T]);
    };

    const sorted = [...rowDescriptors].sort((left, right) => {
      const leftValue = getColumnValue(left);
      const rightValue = getColumnValue(right);

      if (leftValue < rightValue) {
        return resolvedSortState.direction === "asc" ? -1 : 1;
      }

      if (leftValue > rightValue) {
        return resolvedSortState.direction === "asc" ? 1 : -1;
      }

      return 0;
    });

    return sorted;
  }, [columns, resolvedSortState, rowDescriptors]);

  const allSelected = sortedRows.length > 0 && sortedRows.every((item) => resolvedSelection.has(item.id));
  const someSelected = sortedRows.some((item) => resolvedSelection.has(item.id));

  return (
    <Table className={cn("rounded-lg border border-[var(--grid-border,#d8ebdf)] bg-white", className)}>
      <TableHeader className="bg-[var(--grid-header-bg,#f5faf7)]">
        <TableRow className="hover:bg-transparent">
          <TableHead className="w-12">
            <Checkbox
              aria-label="Select all rows"
              checked={allSelected}
              ref={(element) => {
                if (!element) {
                  return;
                }

                element.indeterminate = !allSelected && someSelected;
              }}
              onChange={(event) => {
                if (event.currentTarget.checked) {
                  setSelection(new Set(sortedRows.map((item) => item.id)));
                  return;
                }

                setSelection(new Set());
              }}
            />
          </TableHead>
          {columns.map((column) => {
            const columnKey = String(column.key);
            const sorted = resolvedSortState?.key === columnKey;

            return (
              <TableHead
                key={columnKey}
                className={cn(
                  column.pin === "left" && "sticky left-0 z-10 bg-[var(--grid-header-bg,#f5faf7)]",
                  column.pin === "right" && "sticky right-0 z-10 bg-[var(--grid-header-bg,#f5faf7)]",
                  column.headerClassName,
                )}
                style={column.width ? { width: column.width } : undefined}
              >
                {column.sortable ? (
                  <button
                    type="button"
                    className="inline-flex items-center gap-1 text-left"
                    onClick={() => {
                      if (!sorted) {
                        setSortState({ key: columnKey, direction: "asc" });
                        return;
                      }

                      if (resolvedSortState?.direction === "asc") {
                        setSortState({ key: columnKey, direction: "desc" });
                        return;
                      }

                      setSortState(null);
                    }}
                  >
                    {column.header}
                    <span aria-hidden className="text-[10px] text-zinc-500">
                      {sorted ? (resolvedSortState?.direction === "asc" ? "▲" : "▼") : "↕"}
                    </span>
                  </button>
                ) : (
                  column.header
                )}
              </TableHead>
            );
          })}
        </TableRow>
      </TableHeader>

      <TableBody>
        {loading
          ? Array.from({ length: 4 }).map((_, index) => (
              <TableRow key={`loading-${index}`} className="hover:bg-transparent">
                <TableCell colSpan={columns.length + 1}>
                  <div className="h-5 w-full animate-pulse rounded bg-zinc-100" />
                </TableCell>
              </TableRow>
            ))
          : null}

        {!loading && sortedRows.length === 0 ? (
          <TableRow className="hover:bg-transparent">
            <TableCell colSpan={columns.length + 1} className="py-10">
              {emptyState ?? (
                <div className="text-center text-sm text-zinc-500">No records found.</div>
              )}
            </TableCell>
          </TableRow>
        ) : null}

        {!loading
          ? sortedRows.map((descriptor) => {
              const selected = resolvedSelection.has(descriptor.id);

              return (
                <TableRow
                  key={descriptor.id}
                  className={cn(
                    "cursor-pointer hover:bg-[var(--grid-row-hover,#f6fbf8)]",
                    selected && "bg-[var(--grid-row-selected,#e8f7ef)]",
                    rowClassName?.(descriptor.row, descriptor.index),
                  )}
                  onClick={() => onRowClick?.(descriptor.row, descriptor.index)}
                >
                  <TableCell onClick={(event) => event.stopPropagation()}>
                    <Checkbox
                      aria-label={`Select row ${descriptor.index + 1}`}
                      checked={selected}
                      onChange={(event) => {
                        const nextSelection = new Set(resolvedSelection);
                        if (event.currentTarget.checked) {
                          nextSelection.add(descriptor.id);
                        } else {
                          nextSelection.delete(descriptor.id);
                        }

                        setSelection(nextSelection);
                      }}
                    />
                  </TableCell>
                  {columns.map((column) => {
                    const columnKey = String(column.key);
                    const renderedCell = column.cell
                      ? column.cell(descriptor.row, descriptor.index)
                      : (descriptor.row[column.key as keyof T] as React.ReactNode);

                    return (
                      <TableCell
                        key={`${descriptor.id}-${columnKey}`}
                        className={cn(
                          column.pin === "left" && "sticky left-0 z-[1] bg-white",
                          column.pin === "right" && "sticky right-0 z-[1] bg-white",
                          column.className,
                        )}
                        style={column.width ? { width: column.width } : undefined}
                      >
                        {renderedCell}
                      </TableCell>
                    );
                  })}
                </TableRow>
              );
            })
          : null}
      </TableBody>
    </Table>
  );
}
