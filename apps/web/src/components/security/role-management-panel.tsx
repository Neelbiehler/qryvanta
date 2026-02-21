"use client";

import { type FormEvent, useMemo, useState } from "react";
import { useRouter } from "next/navigation";

import { Button, Checkbox, Input, Label, Select } from "@qryvanta/ui";

import {
  apiFetch,
  type RoleAssignmentResponse,
  type RoleResponse,
  type UpdateTenantRegistrationModeRequest,
} from "@/lib/api";

const PERMISSION_OPTIONS = [
  "metadata.entity.read",
  "metadata.entity.create",
  "metadata.field.read",
  "metadata.field.write",
  "runtime.record.read",
  "runtime.record.write",
  "security.audit.read",
  "security.role.manage",
  "security.invite.send",
] as const;

type RoleManagementPanelProps = {
  roles: RoleResponse[];
  assignments: RoleAssignmentResponse[];
  registrationMode: string;
};

export function RoleManagementPanel({
  roles,
  assignments,
  registrationMode,
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

  function togglePermission(permission: string) {
    setSelectedPermissions((current) =>
      current.includes(permission)
        ? current.filter((value) => value !== permission)
        : [...current, permission],
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

      {errorMessage ? (
        <p className="md:col-span-2 rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
          {errorMessage}
        </p>
      ) : null}
    </div>
  );
}
