import { type FormEvent, useMemo, useState } from "react";
import { useRouter } from "next/navigation";

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
import {
  type EditableFieldPermission,
  type SecurityAdminSection,
} from "@/components/security/role-management/constants";

type UseRoleManagementPanelInput = {
  assignments: RoleAssignmentResponse[];
  registrationMode: string;
  roles: RoleResponse[];
  runtimeFieldPermissions: RuntimeFieldPermissionResponse[];
  temporaryAccessGrants: TemporaryAccessGrantResponse[];
};

export function useRoleManagementPanel({
  assignments,
  registrationMode,
  roles,
  runtimeFieldPermissions,
  temporaryAccessGrants,
}: UseRoleManagementPanelInput) {
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
  const [grantToRevokeId, setGrantToRevokeId] = useState<string | null>(null);
  const [revokeReason, setRevokeReason] = useState("");
  const [isRevokingTemporaryGrant, setIsRevokingTemporaryGrant] =
    useState(false);
  const [activeSection, setActiveSection] =
    useState<SecurityAdminSection>("roles");

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

  async function handleUnassign(subject: string, roleNameValue: string) {
    setErrorMessage(null);

    try {
      const response = await apiFetch("/api/security/role-unassignments", {
        method: "POST",
        body: JSON.stringify({
          subject,
          role_name: roleNameValue,
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
        formData.get("tenant_registration_mode")?.toString() ?? registrationMode;

      const payload: UpdateTenantRegistrationModeRequest = {
        registration_mode: selectedMode,
      };
      const response = await apiFetch("/api/security/registration-mode", {
        method: "PUT",
        body: JSON.stringify(payload),
      });

      if (!response.ok) {
        const payload = (await response.json()) as { message?: string };
        setErrorMessage(payload.message ?? "Unable to update registration mode.");
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

      const response = await apiFetch(
        "/api/security/runtime-field-permissions",
        {
          method: "PUT",
          body: JSON.stringify(payload),
        },
      );

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

  function startRevokeTemporaryGrant(grantId: string) {
    setGrantToRevokeId(grantId);
    setRevokeReason("");
  }

  function cancelRevokeTemporaryGrant() {
    setGrantToRevokeId(null);
    setRevokeReason("");
  }

  async function handleRevokeTemporaryGrant() {
    setErrorMessage(null);
    if (!grantToRevokeId) {
      setErrorMessage("Select a temporary grant before revoking.");
      return;
    }

    setIsRevokingTemporaryGrant(true);
    try {
      const payload = {
        revoke_reason: revokeReason.trim() ? revokeReason.trim() : null,
      };
      const response = await apiFetch(
        `/api/security/temporary-access-grants/${grantToRevokeId}/revoke`,
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

      setGrantToRevokeId(null);
      setRevokeReason("");
      router.refresh();
    } catch {
      setErrorMessage("Unable to revoke temporary access grant.");
    } finally {
      setIsRevokingTemporaryGrant(false);
    }
  }

  return {
    activeSection,
    addFieldPermissionDraft,
    assignRoleName,
    assignments,
    assignSubject,
    cancelRevokeTemporaryGrant,
    errorMessage,
    fieldPermissionCanRead,
    fieldPermissionCanWrite,
    fieldPermissionEntity,
    fieldPermissionFieldName,
    fieldPermissionSubject,
    fieldPermissionsDraft,
    grantToRevokeId,
    handleAssignSubmit,
    handleCreateTemporaryGrant,
    handleRegistrationModeSubmit,
    handleRevokeTemporaryGrant,
    handleRoleSubmit,
    handleSaveFieldPermissions,
    handleUnassign,
    isAssigning,
    isCreatingTemporaryGrant,
    isRevokingTemporaryGrant,
    isSavingFieldPermissions,
    isSubmittingRole,
    isUpdatingRegistrationMode,
    registrationMode,
    removeFieldPermissionDraft,
    revokeReason,
    roleName,
    roleNames,
    roles,
    runtimeFieldPermissions,
    selectedPermissions,
    setActiveSection,
    setAssignRoleName,
    setAssignSubject,
    setFieldPermissionCanRead,
    setFieldPermissionCanWrite,
    setFieldPermissionEntity,
    setFieldPermissionFieldName,
    setFieldPermissionSubject,
    setRevokeReason,
    setRoleName,
    setTemporaryDurationMinutes,
    setTemporaryReason,
    setTemporarySubject,
    startRevokeTemporaryGrant,
    temporaryAccessGrants,
    temporaryDurationMinutes,
    temporaryPermissions,
    temporaryReason,
    temporarySubject,
    togglePermission,
    toggleTemporaryPermission,
  };
}
