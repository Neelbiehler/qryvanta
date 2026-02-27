"use client";

import { type FormEvent, useEffect, useState } from "react";

import { Notice } from "@qryvanta/ui";

import {
  type PermissionDraft,
  RolePermissionsSection,
} from "@/components/studio/security/role-permissions-section";
import {
  apiFetch,
  type AppResponse,
  type AppRoleEntityPermissionResponse,
  type EntityResponse,
  type RoleResponse,
  type SaveAppRoleEntityPermissionRequest,
} from "@/lib/api";

type StudioSecurityCanvasProps = {
  apps: AppResponse[];
  entities: EntityResponse[];
  roles: RoleResponse[];
  selectedApp: string;
  onChangeSelectedApp: (appLogicalName: string) => void;
};

export function StudioSecurityCanvas({
  apps,
  entities,
  roles,
  selectedApp,
  onChangeSelectedApp,
}: StudioSecurityCanvasProps) {
  const [permissions, setPermissions] = useState<AppRoleEntityPermissionResponse[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const [permissionDraft, setPermissionDraft] = useState<PermissionDraft>({
    roleName: roles.at(0)?.name ?? "",
    entityName: entities.at(0)?.logical_name ?? "",
    canRead: true,
    canCreate: false,
    canUpdate: false,
    canDelete: false,
  });

  useEffect(() => {
    if (!selectedApp) return;
    let isMounted = true;

    async function loadPermissions(): Promise<void> {
      setIsLoading(true);
      setErrorMessage(null);
      setStatusMessage(null);
      try {
        const response = await apiFetch(
          `/api/apps/${encodeURIComponent(selectedApp)}/permissions`,
        );
        if (!isMounted) return;

        if (!response.ok) {
          const payload = (await response.json()) as { message?: string };
          setErrorMessage(payload.message ?? "Unable to load app permissions.");
          setPermissions([]);
          return;
        }

        setPermissions((await response.json()) as AppRoleEntityPermissionResponse[]);
      } catch {
        if (!isMounted) return;
        setErrorMessage("Unable to load app permissions.");
        setPermissions([]);
      } finally {
        if (isMounted) {
          setIsLoading(false);
        }
      }
    }

    void loadPermissions();

    return () => {
      isMounted = false;
    };
  }, [selectedApp]);

  async function handleSavePermission(event: FormEvent<HTMLFormElement>): Promise<void> {
    event.preventDefault();
    if (!selectedApp) {
      setErrorMessage("Select an app first.");
      return;
    }

    setIsSaving(true);
    setErrorMessage(null);
    setStatusMessage(null);

    try {
      const payload: SaveAppRoleEntityPermissionRequest = {
        role_name: permissionDraft.roleName,
        entity_logical_name: permissionDraft.entityName,
        can_read: permissionDraft.canRead,
        can_create: permissionDraft.canCreate,
        can_update: permissionDraft.canUpdate,
        can_delete: permissionDraft.canDelete,
      };
      const response = await apiFetch(`/api/apps/${encodeURIComponent(selectedApp)}/permissions`, {
        method: "PUT",
        body: JSON.stringify(payload),
      });

      if (!response.ok) {
        const data = (await response.json()) as { message?: string };
        setErrorMessage(data.message ?? "Unable to save app permissions.");
        return;
      }

      const refreshResponse = await apiFetch(
        `/api/apps/${encodeURIComponent(selectedApp)}/permissions`,
      );
      if (refreshResponse.ok) {
        setPermissions((await refreshResponse.json()) as AppRoleEntityPermissionResponse[]);
      }
      setStatusMessage("Role permissions saved.");
    } catch {
      setErrorMessage("Unable to save app permissions.");
    } finally {
      setIsSaving(false);
    }
  }

  return (
    <div className="space-y-3 rounded-xl border border-zinc-200 bg-zinc-50 p-3">
      <RolePermissionsSection
        apps={apps}
        entities={entities}
        roles={roles}
        selectedApp={selectedApp}
        onChangeSelectedApp={onChangeSelectedApp}
        isLoadingAppData={isLoading}
        isSavingPermission={isSaving}
        permissions={permissions}
        permissionDraft={permissionDraft}
        onUpdatePermissionDraft={setPermissionDraft}
        onSavePermission={(event) => {
          void handleSavePermission(event);
        }}
      />
      {errorMessage ? <Notice tone="error">{errorMessage}</Notice> : null}
      {statusMessage ? <Notice tone="success">{statusMessage}</Notice> : null}
    </div>
  );
}
