"use client";

import { type FormEvent, useMemo, useState } from "react";
import { useRouter } from "next/navigation";

import { Button, Checkbox, Input, Label, Select } from "@qryvanta/ui";

import {
  apiFetch,
  type CreateTemporaryAccessGrantRequest,
  type RoleAssignmentResponse,
  type RoleResponse,
  type RuntimeFieldPermissionResponse,
  type SaveRuntimeFieldPermissionsRequest,
  type TemporaryAccessGrantResponse,
  type UpdateTenantRegistrationModeRequest,
} from "@/lib/api";

const PERMISSION_OPTIONS = [
  "metadata.entity.read",
  "metadata.entity.create",
  "metadata.field.read",
  "metadata.field.write",
  "runtime.record.read",
  "runtime.record.read.own",
  "runtime.record.write",
  "runtime.record.write.own",
  "security.audit.read",
  "security.role.manage",
  "security.invite.send",
] as const;

type EditableFieldPermission = {
  fieldLogicalName: string;
  canRead: boolean;
  canWrite: boolean;
};

type RoleManagementPanelProps = {
  roles: RoleResponse[];
  assignments: RoleAssignmentResponse[];
  registrationMode: string;
  runtimeFieldPermissions: RuntimeFieldPermissionResponse[];
  temporaryAccessGrants: TemporaryAccessGrantResponse[];
};

export function RoleManagementPanel({
  roles,
  assignments,
  registrationMode,
  runtimeFieldPermissions,
  temporaryAccessGrants,
}: RoleManagementPanelProps) {
  const router = useRouter();
  const roleNames = useMemo(() => roles.map((role) => role.name), [roles]);

  const [roleName, setRoleName] = useState("");
  const [selectedPermissions, setSelectedPermissions] = useState<string[]>([
    "metadata.entity.read",
  ]);
  const [assignSubject, setAssignSubject] = useState("");
  const [assignRoleName, setAssignRoleName] = useState("");
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [isSubmittingRole, setIsSubmittingRole] = useState(false);
  const [isAssigning, setIsAssigning] = useState(false);
  const [isUpdatingRegistrationMode, setIsUpdatingRegistrationMode] =
    useState(false);

  const [fieldPermissionSubject, setFieldPermissionSubject] = useState("");
  const [fieldPermissionEntity, setFieldPermissionEntity] = useState("");
  const [fieldPermissionFieldName, setFieldPermissionFieldName] = useState("");
  const [fieldPermissionCanRead, setFieldPermissionCanRead] = useState(true);
  const [fieldPermissionCanWrite, setFieldPermissionCanWrite] = useState(false);
  const [fieldPermissionsDraft, setFieldPermissionsDraft] = useState<
    EditableFieldPermission[]
  >([]);
  const [isSavingFieldPermissions, setIsSavingFieldPermissions] =
    useState(false);

  const [temporarySubject, setTemporarySubject] = useState("");
  const [temporaryReason, setTemporaryReason] = useState("");
  const [temporaryDurationMinutes, setTemporaryDurationMinutes] =
    useState("60");
  const [temporaryPermissions, setTemporaryPermissions] = useState<string[]>([
    "runtime.record.read",
  ]);
  const [isCreatingTemporaryGrant, setIsCreatingTemporaryGrant] =
    useState(false);

  function togglePermission(permission: string) {
    setSelectedPermissions((current) =>
      current.includes(permission)
        ? current.filter((value) => value !== permission)
        : [...current, permission],
    );
  }

  function toggleTemporaryPermission(permission: string) {
    setTemporaryPermissions((current) =>
      current.includes(permission)
        ? current.filter((value) => value !== permission)
        : [...current, permission],
    );
  }

  function addFieldPermissionDraft() {
    if (!fieldPermissionFieldName.trim()) {
      setErrorMessage("Field logical name is required.");
      return;
    }

    setFieldPermissionsDraft((current) => {
      const withoutExisting = current.filter(
        (entry) =>
          entry.fieldLogicalName.toLowerCase() !==
          fieldPermissionFieldName.trim().toLowerCase(),
      );

      return [
        ...withoutExisting,
        {
          fieldLogicalName: fieldPermissionFieldName.trim(),
          canRead: fieldPermissionCanRead,
          canWrite: fieldPermissionCanWrite,
        },
      ];
    });

    setFieldPermissionFieldName("");
    setErrorMessage(null);
  }

  function removeFieldPermissionDraft(fieldLogicalName: string) {
    setFieldPermissionsDraft((current) =>
      current.filter((entry) => entry.fieldLogicalName !== fieldLogicalName),
    );
  }

  async function handleRoleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setErrorMessage(null);
    setIsSubmittingRole(true);

    try {
      const response = await apiFetch("/api/security/roles", {
        method: "POST",
        body: JSON.stringify({
          name: roleName,
          permissions: selectedPermissions,
        }),
      });

      if (!response.ok) {
        const payload = (await response.json()) as { message?: string };
        setErrorMessage(payload.message ?? "Unable to create role.");
        return;
      }

      setRoleName("");
      router.refresh();
    } catch {
      setErrorMessage("Unable to create role.");
    } finally {
      setIsSubmittingRole(false);
    }
  }

  async function handleAssignSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setErrorMessage(null);
    setIsAssigning(true);

    try {
      const response = await apiFetch("/api/security/role-assignments", {
        method: "POST",
        body: JSON.stringify({
          subject: assignSubject,
          role_name: assignRoleName,
        }),
      });

      if (!response.ok) {
        const payload = (await response.json()) as { message?: string };
        setErrorMessage(payload.message ?? "Unable to assign role.");
        return;
      }

      setAssignSubject("");
      setAssignRoleName("");
      router.refresh();
    } catch {
      setErrorMessage("Unable to assign role.");
    } finally {
      setIsAssigning(false);
    }
  }

  async function handleUnassign(subject: string, roleName: string) {
    setErrorMessage(null);

    try {
      const response = await apiFetch("/api/security/role-unassignments", {
        method: "POST",
        body: JSON.stringify({
          subject,
          role_name: roleName,
        }),
      });

      if (!response.ok) {
        const payload = (await response.json()) as { message?: string };
        setErrorMessage(payload.message ?? "Unable to remove role assignment.");
        return;
      }

      router.refresh();
    } catch {
      setErrorMessage("Unable to remove role assignment.");
    }
  }

  async function handleRegistrationModeSubmit(
    event: FormEvent<HTMLFormElement>,
  ) {
    event.preventDefault();
    setErrorMessage(null);
    setIsUpdatingRegistrationMode(true);

    try {
      const formData = new FormData(event.currentTarget);
      const selectedMode =
        formData.get("tenant_registration_mode")?.toString() ??
        registrationMode;

      const payload: UpdateTenantRegistrationModeRequest = {
        registration_mode: selectedMode,
      };
      const response = await apiFetch("/api/security/registration-mode", {
        method: "PUT",
        body: JSON.stringify(payload),
      });

      if (!response.ok) {
        const payload = (await response.json()) as { message?: string };
        setErrorMessage(
          payload.message ?? "Unable to update registration mode.",
        );
        return;
      }
      router.refresh();
    } catch {
      setErrorMessage("Unable to update registration mode.");
    } finally {
      setIsUpdatingRegistrationMode(false);
    }
  }

  async function handleSaveFieldPermissions(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setErrorMessage(null);

    if (!fieldPermissionSubject.trim() || !fieldPermissionEntity.trim()) {
      setErrorMessage("Subject and entity logical name are required.");
      return;
    }

    if (fieldPermissionsDraft.length === 0) {
      setErrorMessage("Add at least one field permission before saving.");
      return;
    }

    setIsSavingFieldPermissions(true);
    try {
      const payload: SaveRuntimeFieldPermissionsRequest = {
        subject: fieldPermissionSubject.trim(),
        entity_logical_name: fieldPermissionEntity.trim(),
        fields: fieldPermissionsDraft.map((entry) => ({
          field_logical_name: entry.fieldLogicalName,
          can_read: entry.canRead,
          can_write: entry.canWrite,
        })),
      };

      const response = await apiFetch("/api/security/runtime-field-permissions", {
        method: "PUT",
        body: JSON.stringify(payload),
      });

      if (!response.ok) {
        const payload = (await response.json()) as { message?: string };
        setErrorMessage(payload.message ?? "Unable to save field permissions.");
        return;
      }

      setFieldPermissionsDraft([]);
      setFieldPermissionFieldName("");
      router.refresh();
    } catch {
      setErrorMessage("Unable to save field permissions.");
    } finally {
      setIsSavingFieldPermissions(false);
    }
  }

  async function handleCreateTemporaryGrant(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setErrorMessage(null);

    if (temporaryPermissions.length === 0) {
      setErrorMessage("Select at least one permission for temporary access.");
      return;
    }

    const durationMinutes = Number.parseInt(temporaryDurationMinutes, 10);
    if (Number.isNaN(durationMinutes) || durationMinutes <= 0) {
      setErrorMessage("Duration must be a positive number of minutes.");
      return;
    }

    setIsCreatingTemporaryGrant(true);
    try {
      const payload: CreateTemporaryAccessGrantRequest = {
        subject: temporarySubject.trim(),
        permissions: temporaryPermissions,
        reason: temporaryReason.trim(),
        duration_minutes: durationMinutes,
      };

      const response = await apiFetch("/api/security/temporary-access-grants", {
        method: "POST",
        body: JSON.stringify(payload),
      });

      if (!response.ok) {
        const payload = (await response.json()) as { message?: string };
        setErrorMessage(
          payload.message ?? "Unable to create temporary access grant.",
        );
        return;
      }

      setTemporarySubject("");
      setTemporaryReason("");
      setTemporaryDurationMinutes("60");
      router.refresh();
    } catch {
      setErrorMessage("Unable to create temporary access grant.");
    } finally {
      setIsCreatingTemporaryGrant(false);
    }
  }

  async function handleRevokeTemporaryGrant(grantId: string) {
    setErrorMessage(null);

    const revokeReason = window.prompt(
      "Optional revoke reason (leave empty to skip):",
      "",
    );

    try {
      const payload = {
        revoke_reason: revokeReason?.trim() ? revokeReason.trim() : null,
      };
      const response = await apiFetch(
        `/api/security/temporary-access-grants/${grantId}/revoke`,
        {
          method: "POST",
          body: JSON.stringify(payload),
        },
      );

      if (!response.ok) {
        const payload = (await response.json()) as { message?: string };
        setErrorMessage(
          payload.message ?? "Unable to revoke temporary access grant.",
        );
        return;
      }

      router.refresh();
    } catch {
      setErrorMessage("Unable to revoke temporary access grant.");
    }
  }

  return (
    <div className="grid gap-8 md:grid-cols-2">
      <form className="space-y-4" onSubmit={handleRoleSubmit}>
        <div className="space-y-2">
          <Label htmlFor="role_name">Role Name</Label>
          <Input
            id="role_name"
            value={roleName}
            onChange={(event) => setRoleName(event.target.value)}
            placeholder="operations_editor"
            required
          />
        </div>

        <fieldset className="space-y-2">
          <legend className="text-sm font-medium text-zinc-800">
            Permissions
          </legend>
          <div className="space-y-2 rounded-md border border-emerald-100 bg-white p-3">
            {PERMISSION_OPTIONS.map((permission) => (
              <label
                key={permission}
                className="flex items-center gap-2 text-sm text-zinc-700"
              >
                <Checkbox
                  checked={selectedPermissions.includes(permission)}
                  onChange={() => togglePermission(permission)}
                />
                <span className="font-mono text-xs">{permission}</span>
              </label>
            ))}
          </div>
        </fieldset>

        <Button disabled={isSubmittingRole} type="submit">
          {isSubmittingRole ? "Creating..." : "Create Role"}
        </Button>
      </form>

      <form className="space-y-4" onSubmit={handleAssignSubmit}>
        <div className="space-y-2">
          <Label htmlFor="assign_subject">Subject</Label>
          <Input
            id="assign_subject"
            value={assignSubject}
            onChange={(event) => setAssignSubject(event.target.value)}
            placeholder="alice"
            required
          />
        </div>

        <div className="space-y-2">
          <Label htmlFor="assign_role_name">Role Name</Label>
          <Input
            id="assign_role_name"
            value={assignRoleName}
            onChange={(event) => setAssignRoleName(event.target.value)}
            list="role_names"
            placeholder="tenant_owner"
            required
          />
          <datalist id="role_names">
            {roleNames.map((name) => (
              <option key={name} value={name} />
            ))}
          </datalist>
        </div>

        <Button disabled={isAssigning} type="submit" variant="outline">
          {isAssigning ? "Assigning..." : "Assign Role"}
        </Button>
      </form>

      <form
        className="space-y-4 md:col-span-2"
        onSubmit={handleRegistrationModeSubmit}
      >
        <div className="space-y-2">
          <Label htmlFor="tenant_registration_mode">
            Tenant Registration Mode
          </Label>
          <p className="text-sm text-zinc-600">
            Control whether users can self-register or only join by invite.
          </p>
          <Select
            id="tenant_registration_mode"
            defaultValue={registrationMode}
            name="tenant_registration_mode"
          >
            <option value="invite_only">Invite only</option>
            <option value="open">Open registration</option>
          </Select>
        </div>

        <Button
          disabled={isUpdatingRegistrationMode}
          type="submit"
          variant="outline"
        >
          {isUpdatingRegistrationMode ? "Saving..." : "Save Registration Mode"}
        </Button>
      </form>

      <form
        className="space-y-4 rounded-md border border-emerald-100 bg-white p-4 md:col-span-2"
        onSubmit={handleSaveFieldPermissions}
      >
        <p className="text-sm font-medium text-zinc-900">Runtime Field Permissions</p>
        <div className="grid gap-3 md:grid-cols-2">
          <div className="space-y-2">
            <Label htmlFor="field_permission_subject">Subject</Label>
            <Input
              id="field_permission_subject"
              value={fieldPermissionSubject}
              onChange={(event) => setFieldPermissionSubject(event.target.value)}
              placeholder="alice"
              required
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="field_permission_entity">Entity</Label>
            <Input
              id="field_permission_entity"
              value={fieldPermissionEntity}
              onChange={(event) => setFieldPermissionEntity(event.target.value)}
              placeholder="contact"
              required
            />
          </div>
        </div>

        <div className="grid gap-3 rounded-md border border-emerald-100 bg-emerald-50/40 p-3 md:grid-cols-4">
          <div className="space-y-2 md:col-span-2">
            <Label htmlFor="field_permission_field_name">Field</Label>
            <Input
              id="field_permission_field_name"
              value={fieldPermissionFieldName}
              onChange={(event) => setFieldPermissionFieldName(event.target.value)}
              placeholder="email"
            />
          </div>
          <label className="flex items-center gap-2 text-sm text-zinc-700 md:mt-7">
            <Checkbox
              checked={fieldPermissionCanRead}
              onChange={() => setFieldPermissionCanRead((current) => !current)}
            />
            Read
          </label>
          <label className="flex items-center gap-2 text-sm text-zinc-700 md:mt-7">
            <Checkbox
              checked={fieldPermissionCanWrite}
              onChange={() => setFieldPermissionCanWrite((current) => !current)}
            />
            Write
          </label>
          <Button
            className="md:col-span-4"
            onClick={addFieldPermissionDraft}
            type="button"
            variant="outline"
          >
            Add Field Rule
          </Button>
        </div>

        <div className="space-y-2">
          {fieldPermissionsDraft.map((entry) => (
            <div
              key={entry.fieldLogicalName}
              className="flex items-center justify-between rounded-md border border-emerald-100 px-3 py-2"
            >
              <p className="font-mono text-xs text-zinc-700">
                {entry.fieldLogicalName} (read={String(entry.canRead)}, write=
                {String(entry.canWrite)})
              </p>
              <Button
                onClick={() => removeFieldPermissionDraft(entry.fieldLogicalName)}
                type="button"
                variant="outline"
              >
                Remove
              </Button>
            </div>
          ))}
          {fieldPermissionsDraft.length === 0 ? (
            <p className="text-sm text-zinc-500">No field rules staged.</p>
          ) : null}
        </div>

        <Button disabled={isSavingFieldPermissions} type="submit">
          {isSavingFieldPermissions ? "Saving..." : "Save Field Permissions"}
        </Button>
      </form>

      <form
        className="space-y-4 rounded-md border border-emerald-100 bg-white p-4 md:col-span-2"
        onSubmit={handleCreateTemporaryGrant}
      >
        <p className="text-sm font-medium text-zinc-900">Temporary Access Grants</p>

        <div className="grid gap-3 md:grid-cols-3">
          <div className="space-y-2">
            <Label htmlFor="temporary_subject">Subject</Label>
            <Input
              id="temporary_subject"
              value={temporarySubject}
              onChange={(event) => setTemporarySubject(event.target.value)}
              placeholder="oncall-user"
              required
            />
          </div>
          <div className="space-y-2 md:col-span-2">
            <Label htmlFor="temporary_reason">Reason</Label>
            <Input
              id="temporary_reason"
              value={temporaryReason}
              onChange={(event) => setTemporaryReason(event.target.value)}
              placeholder="Incident triage"
              required
            />
          </div>
        </div>

        <div className="space-y-2">
          <Label htmlFor="temporary_duration">Duration (minutes)</Label>
          <Input
            id="temporary_duration"
            value={temporaryDurationMinutes}
            onChange={(event) => setTemporaryDurationMinutes(event.target.value)}
            placeholder="60"
            type="number"
          />
        </div>

        <fieldset className="space-y-2">
          <legend className="text-sm font-medium text-zinc-800">Permissions</legend>
          <div className="grid gap-2 rounded-md border border-emerald-100 bg-emerald-50/40 p-3 md:grid-cols-2">
            {PERMISSION_OPTIONS.map((permission) => (
              <label
                key={`temporary-${permission}`}
                className="flex items-center gap-2 text-sm text-zinc-700"
              >
                <Checkbox
                  checked={temporaryPermissions.includes(permission)}
                  onChange={() => toggleTemporaryPermission(permission)}
                />
                <span className="font-mono text-xs">{permission}</span>
              </label>
            ))}
          </div>
        </fieldset>

        <Button disabled={isCreatingTemporaryGrant} type="submit" variant="outline">
          {isCreatingTemporaryGrant ? "Creating..." : "Create Temporary Grant"}
        </Button>
      </form>

      <div className="space-y-3 md:col-span-2">
        <p className="text-sm font-medium text-zinc-800">Quick Unassign</p>
        <div className="grid gap-2">
          {assignments.slice(0, 8).map((assignment) => (
            <div
              key={`${assignment.subject}-${assignment.role_id}`}
              className="flex items-center justify-between rounded-md border border-emerald-100 bg-white px-3 py-2"
            >
              <div>
                <p className="text-sm text-zinc-900">{assignment.subject}</p>
                <p className="font-mono text-xs text-zinc-500">
                  {assignment.role_name}
                </p>
              </div>
              <Button
                type="button"
                variant="outline"
                onClick={() =>
                  handleUnassign(assignment.subject, assignment.role_name)
                }
              >
                Remove
              </Button>
            </div>
          ))}

          {assignments.length === 0 ? (
            <p className="text-sm text-zinc-500">No assignments available.</p>
          ) : null}
        </div>
      </div>

      <div className="space-y-3 md:col-span-2">
        <p className="text-sm font-medium text-zinc-800">Temporary Grants</p>
        <div className="grid gap-2">
          {temporaryAccessGrants.slice(0, 12).map((grant) => (
            <div
              key={grant.grant_id}
              className="flex items-center justify-between rounded-md border border-emerald-100 bg-white px-3 py-2"
            >
              <div className="space-y-1">
                <p className="text-sm text-zinc-900">{grant.subject}</p>
                <p className="font-mono text-xs text-zinc-500">
                  {grant.permissions.join(", ")} | expires {grant.expires_at}
                </p>
                <p className="text-xs text-zinc-600">{grant.reason}</p>
              </div>
              {grant.revoked_at ? (
                <p className="text-xs text-zinc-500">Revoked</p>
              ) : (
                <Button
                  onClick={() => handleRevokeTemporaryGrant(grant.grant_id)}
                  type="button"
                  variant="outline"
                >
                  Revoke
                </Button>
              )}
            </div>
          ))}
          {temporaryAccessGrants.length === 0 ? (
            <p className="text-sm text-zinc-500">No temporary grants found.</p>
          ) : null}
        </div>
      </div>

      <div className="space-y-3 md:col-span-2">
        <p className="text-sm font-medium text-zinc-800">Runtime Field Permission Entries</p>
        <div className="grid gap-2">
          {runtimeFieldPermissions.slice(0, 20).map((entry) => (
            <div
              key={`${entry.subject}-${entry.entity_logical_name}-${entry.field_logical_name}`}
              className="rounded-md border border-emerald-100 bg-white px-3 py-2"
            >
              <p className="text-sm text-zinc-900">
                {entry.subject} / {entry.entity_logical_name}
              </p>
              <p className="font-mono text-xs text-zinc-600">
                {entry.field_logical_name} (read={String(entry.can_read)}, write=
                {String(entry.can_write)})
              </p>
            </div>
          ))}
          {runtimeFieldPermissions.length === 0 ? (
            <p className="text-sm text-zinc-500">No runtime field permissions found.</p>
          ) : null}
        </div>
      </div>

      {errorMessage ? (
        <p className="md:col-span-2 rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
          {errorMessage}
        </p>
      ) : null}
    </div>
  );
}
