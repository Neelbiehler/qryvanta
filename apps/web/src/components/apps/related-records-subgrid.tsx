"use client";

import { useEffect, useMemo, useState } from "react";

import { Notice, Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@qryvanta/ui";

import {
  apiFetch,
  type FieldResponse,
  type PublishedSchemaResponse,
  type QueryRuntimeRecordsRequest,
  type RuntimeRecordResponse,
} from "@/lib/api";

type RelatedRecordsSubgridProps = {
  appLogicalName: string;
  currentRecordId: string;
  displayName: string;
  targetEntityLogicalName: string;
  relationFieldLogicalName: string;
  columns: string[];
};

export function RelatedRecordsSubgrid({
  appLogicalName,
  currentRecordId,
  displayName,
  targetEntityLogicalName,
  relationFieldLogicalName,
  columns,
}: RelatedRecordsSubgridProps) {
  const [schema, setSchema] = useState<PublishedSchemaResponse | null>(null);
  const [records, setRecords] = useState<RuntimeRecordResponse[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    let isMounted = true;

    async function loadSubgrid() {
      if (!targetEntityLogicalName || !relationFieldLogicalName) {
        if (isMounted) {
          setSchema(null);
          setRecords([]);
        }
        return;
      }

      setIsLoading(true);
      setError(null);

      try {
        const schemaResponse = await apiFetch(
          `/api/workspace/apps/${appLogicalName}/entities/${targetEntityLogicalName}/schema`,
        );

        if (!schemaResponse.ok) {
          if (isMounted) {
            setError(`Unable to load schema for ${targetEntityLogicalName}.`);
          }
          return;
        }

        const nextSchema = (await schemaResponse.json()) as PublishedSchemaResponse;

        const payload: QueryRuntimeRecordsRequest = {
          limit: 20,
          offset: 0,
          logical_mode: "and",
          where: null,
          conditions: [
            {
              scope_alias: null,
              field_logical_name: relationFieldLogicalName,
              operator: "eq",
              field_value: currentRecordId,
            },
          ],
          link_entities: null,
          sort: [],
          filters: null,
        };

        const recordsResponse = await apiFetch(
          `/api/workspace/apps/${appLogicalName}/entities/${targetEntityLogicalName}/records/query`,
          {
            method: "POST",
            body: JSON.stringify(payload),
          },
        );

        if (!recordsResponse.ok) {
          if (isMounted) {
            setError(`Unable to query related records for ${targetEntityLogicalName}.`);
          }
          return;
        }

        const nextRecords = (await recordsResponse.json()) as RuntimeRecordResponse[];

        if (isMounted) {
          setSchema(nextSchema);
          setRecords(nextRecords);
        }
      } finally {
        if (isMounted) {
          setIsLoading(false);
        }
      }
    }

    void loadSubgrid();

    return () => {
      isMounted = false;
    };
  }, [appLogicalName, currentRecordId, relationFieldLogicalName, targetEntityLogicalName]);

  const visibleFields = useMemo((): FieldResponse[] => {
    if (!schema) {
      return [];
    }

    if (columns.length > 0) {
      const fieldMap = new Map(schema.fields.map((field) => [field.logical_name, field]));
      return columns
        .map((column) => fieldMap.get(column) ?? null)
        .filter((field): field is FieldResponse => field !== null);
    }

    return schema.fields
      .filter((field) => field.logical_name !== relationFieldLogicalName && field.field_type !== "json")
      .slice(0, 4);
  }, [columns, relationFieldLogicalName, schema]);

  return (
    <div className="space-y-2 rounded-md border border-zinc-200 bg-zinc-50 p-3">
      <p className="text-xs font-semibold uppercase tracking-[0.14em] text-zinc-500">
        {displayName}
      </p>

      {isLoading ? <p className="text-xs text-zinc-500">Loading related records...</p> : null}
      {error ? <Notice tone="warning">{error}</Notice> : null}

      {!isLoading && !error ? (
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Record ID</TableHead>
              {visibleFields.map((field) => (
                <TableHead key={field.logical_name}>{field.display_name}</TableHead>
              ))}
            </TableRow>
          </TableHeader>
          <TableBody>
            {records.length > 0 ? (
              records.map((record) => (
                <TableRow key={record.record_id}>
                  <TableCell className="font-mono text-xs">{record.record_id}</TableCell>
                  {visibleFields.map((field) => (
                    <TableCell key={`${record.record_id}-${field.logical_name}`}>
                      {String(record.data[field.logical_name] ?? "-")}
                    </TableCell>
                  ))}
                </TableRow>
              ))
            ) : (
              <TableRow>
                <TableCell className="text-zinc-500" colSpan={visibleFields.length + 1}>
                  No related records found.
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      ) : null}
    </div>
  );
}
