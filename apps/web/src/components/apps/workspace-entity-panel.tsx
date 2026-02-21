"use client";

import { type FormEvent, useMemo, useState } from "react";
import { useRouter } from "next/navigation";

import {
  Button,
  Input,
  Label,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@qryvanta/ui";

import {
  apiFetch,
  type AppEntityCapabilitiesResponse,
  type CreateRuntimeRecordRequest,
  type PublishedSchemaResponse,
  type RuntimeRecordResponse,
} from "@/lib/api";

type WorkspaceEntityPanelProps = {
  appLogicalName: string;
  entityLogicalName: string;
  schema: PublishedSchemaResponse;
  capabilities: AppEntityCapabilitiesResponse;
  records: RuntimeRecordResponse[];
};

export function WorkspaceEntityPanel({
  appLogicalName,
  entityLogicalName,
  schema,
  capabilities,
  records,
}: WorkspaceEntityPanelProps) {
  const router = useRouter();

  const initialValues = useMemo(() => {
    const values: Record<string, unknown> = {};
    for (const field of schema.fields) {
      if (field.default_value !== null) {
        values[field.logical_name] = field.default_value;
        continue;
      }

      if (field.field_type === "boolean") {
        values[field.logical_name] = false;
      } else {
        values[field.logical_name] = "";
      }
    }
    return values;
  }, [schema.fields]);

  const [formValues, setFormValues] = useState<Record<string, unknown>>(initialValues);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const [isSaving, setIsSaving] = useState(false);
  const [deletingRecordId, setDeletingRecordId] = useState<string | null>(null);

  function setFieldValue(fieldLogicalName: string, value: unknown) {
    setFormValues((current) => ({
      ...current,
      [fieldLogicalName]: value,
    }));
  }

  function buildPayloadFromForm() {
    const payload: Record<string, unknown> = {};

    for (const field of schema.fields) {
      const value = formValues[field.logical_name];
      if (field.field_type === "boolean") {
        payload[field.logical_name] = Boolean(value);
        continue;
      }

      if (typeof value === "string") {
        const trimmed = value.trim();
        if (!trimmed) {
          continue;
        }

        if (field.field_type === "number") {
          payload[field.logical_name] = Number.parseFloat(trimmed);
          continue;
        }

        if (field.field_type === "json") {
          payload[field.logical_name] = JSON.parse(trimmed);
          continue;
        }

        payload[field.logical_name] = trimmed;
        continue;
      }

      if (value !== null && value !== undefined) {
        payload[field.logical_name] = value;
      }
    }

    return payload;
  }

  async function handleCreateRecord(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!capabilities.can_create) {
      setErrorMessage("You do not have create permission for this entity in this app.");
      return;
    }

    setErrorMessage(null);
    setStatusMessage(null);
    setIsSaving(true);

    try {
      const payload: CreateRuntimeRecordRequest = {
        data: buildPayloadFromForm(),
      };
      const response = await apiFetch(
        `/api/workspace/apps/${appLogicalName}/entities/${entityLogicalName}/records`,
        {
          method: "POST",
          body: JSON.stringify(payload),
        },
      );

      if (!response.ok) {
        const payload = (await response.json()) as { message?: string };
        setErrorMessage(payload.message ?? "Unable to create record.");
        return;
      }

      setStatusMessage("Record created.");
      setFormValues(initialValues);
      router.refresh();
    } catch {
      setErrorMessage("Unable to create record.");
    } finally {
      setIsSaving(false);
    }
  }

  async function handleDeleteRecord(recordId: string) {
    if (!capabilities.can_delete) {
      setErrorMessage("You do not have delete permission for this entity in this app.");
      return;
    }

    setDeletingRecordId(recordId);
    setErrorMessage(null);
    setStatusMessage(null);

    try {
      const response = await apiFetch(
        `/api/workspace/apps/${appLogicalName}/entities/${entityLogicalName}/records/${recordId}`,
        {
          method: "DELETE",
        },
      );

      if (!response.ok) {
        const payload = (await response.json()) as { message?: string };
        setErrorMessage(payload.message ?? "Unable to delete record.");
        return;
      }

      setStatusMessage("Record deleted.");
      router.refresh();
    } catch {
      setErrorMessage("Unable to delete record.");
    } finally {
      setDeletingRecordId(null);
    }
  }

  return (
    <div className="space-y-6">
      <form className="space-y-4 rounded-md border border-emerald-100 bg-white p-4" onSubmit={handleCreateRecord}>
        <div>
          <p className="text-sm font-medium text-zinc-800">New {schema.entity_display_name} Record</p>
          <p className="text-xs text-zinc-500">Schema version {schema.version}</p>
        </div>

        <div className="grid gap-4 md:grid-cols-2">
          {schema.fields.map((field) => {
            const fieldId = `field_${field.logical_name}`;
            const value = formValues[field.logical_name];

            if (field.field_type === "boolean") {
              return (
                <div className="space-y-2" key={field.logical_name}>
                  <Label htmlFor={fieldId}>{field.display_name}</Label>
                  <label className="inline-flex items-center gap-2 text-sm text-zinc-700">
                    <input
                      id={fieldId}
                      type="checkbox"
                      checked={Boolean(value)}
                      onChange={(event) => setFieldValue(field.logical_name, event.target.checked)}
                    />
                    {field.display_name}
                  </label>
                </div>
              );
            }

            if (field.field_type === "json") {
              return (
                <div className="space-y-2 md:col-span-2" key={field.logical_name}>
                  <Label htmlFor={fieldId}>{field.display_name}</Label>
                  <textarea
                    id={fieldId}
                    className="min-h-24 w-full rounded-md border border-zinc-200 bg-white px-3 py-2 font-mono text-xs text-zinc-900"
                    value={String(value ?? "")}
                    onChange={(event) => setFieldValue(field.logical_name, event.target.value)}
                    placeholder='{"value":"example"}'
                  />
                </div>
              );
            }

            return (
              <div className="space-y-2" key={field.logical_name}>
                <Label htmlFor={fieldId}>{field.display_name}</Label>
                <Input
                  id={fieldId}
                  type={field.field_type === "number" ? "number" : field.field_type === "date" ? "date" : field.field_type === "datetime" ? "datetime-local" : "text"}
                  value={String(value ?? "")}
                  onChange={(event) => setFieldValue(field.logical_name, event.target.value)}
                  required={field.is_required}
                />
              </div>
            );
          })}
        </div>

        <Button disabled={!capabilities.can_create || isSaving} type="submit">
          {isSaving ? "Saving..." : "Create Record"}
        </Button>
      </form>

      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Record ID</TableHead>
            <TableHead>Data</TableHead>
            <TableHead>Actions</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {records.length > 0 ? (
            records.map((record) => (
              <TableRow key={record.record_id}>
                <TableCell className="font-mono text-xs">{record.record_id}</TableCell>
                <TableCell className="font-mono text-xs">{JSON.stringify(record.data)}</TableCell>
                <TableCell>
                  {capabilities.can_delete ? (
                    <Button
                      disabled={deletingRecordId === record.record_id}
                      variant="outline"
                      size="sm"
                      type="button"
                      onClick={() => handleDeleteRecord(record.record_id)}
                    >
                      {deletingRecordId === record.record_id ? "Deleting..." : "Delete"}
                    </Button>
                  ) : (
                    <span className="text-xs text-zinc-500">No delete access</span>
                  )}
                </TableCell>
              </TableRow>
            ))
          ) : (
            <TableRow>
              <TableCell className="text-zinc-500" colSpan={3}>
                No records yet.
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>

      {errorMessage ? (
        <p className="rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">{errorMessage}</p>
      ) : null}
      {statusMessage ? (
        <p className="rounded-md border border-emerald-200 bg-emerald-50 px-3 py-2 text-sm text-emerald-700">{statusMessage}</p>
      ) : null}
    </div>
  );
}
