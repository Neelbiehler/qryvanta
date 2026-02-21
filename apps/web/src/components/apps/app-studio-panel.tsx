"use client";

import { type FormEvent, useEffect, useState } from "react";
import { useRouter } from "next/navigation";

import {
  Button,
  Checkbox,
  Input,
  Label,
  Select,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@qryvanta/ui";

import {
  apiFetch,
  type AppEntityBindingResponse,
  type AppResponse,
  type AppRoleEntityPermissionResponse,
  type BindAppEntityRequest,
  type CreateAppRequest,
  type EntityResponse,
  type RoleResponse,
  type SaveAppRoleEntityPermissionRequest,
} from "@/lib/api";

type AppStudioPanelProps = {
  apps: AppResponse[];
  entities: EntityResponse[];
  roles: RoleResponse[];
};

export function AppStudioPanel({ apps, entities, roles }: AppStudioPanelProps) {
  const router = useRouter();

  const [selectedApp, setSelectedApp] = useState(
    apps.at(0)?.logical_name ?? "",
  );
  const [bindings, setBindings] = useState<AppEntityBindingResponse[]>([]);
  const [permissions, setPermissions] = useState<
    AppRoleEntityPermissionResponse[]
  >([]);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);

  const [newAppLogicalName, setNewAppLogicalName] = useState("");
  const [newAppDisplayName, setNewAppDisplayName] = useState("");
  const [newAppDescription, setNewAppDescription] = useState("");

  const [entityToBind, setEntityToBind] = useState(
    entities.at(0)?.logical_name ?? "",
  );
  const [navigationLabel, setNavigationLabel] = useState("");
  const [navigationOrder, setNavigationOrder] = useState(0);

  const [permissionRoleName, setPermissionRoleName] = useState(
    roles.at(0)?.name ?? "",
  );
  const [permissionEntityName, setPermissionEntityName] = useState(
    entities.at(0)?.logical_name ?? "",
  );
  const [canRead, setCanRead] = useState(true);
  const [canCreate, setCanCreate] = useState(false);
  const [canUpdate, setCanUpdate] = useState(false);
  const [canDelete, setCanDelete] = useState(false);

  const [isCreatingApp, setIsCreatingApp] = useState(false);
  const [isBindingEntity, setIsBindingEntity] = useState(false);
  const [isSavingPermission, setIsSavingPermission] = useState(false);
  const [isLoadingAppData, setIsLoadingAppData] = useState(false);

  const hasStudioData =
    apps.length > 0 && entities.length > 0 && roles.length > 0;

  const selectedAppDisplayName =
    apps.find((app) => app.logical_name === selectedApp)?.display_name ??
    selectedApp;

  function resetMessages() {
    setErrorMessage(null);
    setStatusMessage(null);
  }

  async function refreshSelectedAppData(appLogicalName: string) {
    if (!appLogicalName) {
      setBindings([]);
      setPermissions([]);
      return;
    }

    setIsLoadingAppData(true);
    try {
      const [bindingsResponse, permissionsResponse] = await Promise.all([
        apiFetch(`/api/apps/${appLogicalName}/entities`),
        apiFetch(`/api/apps/${appLogicalName}/permissions`),
      ]);

      if (!bindingsResponse.ok || !permissionsResponse.ok) {
        setErrorMessage("Unable to load app studio data.");
        return;
      }

      setBindings(
        (await bindingsResponse.json()) as AppEntityBindingResponse[],
      );
      setPermissions(
        (await permissionsResponse.json()) as AppRoleEntityPermissionResponse[],
      );
    } catch {
      setErrorMessage("Unable to load app studio data.");
    } finally {
      setIsLoadingAppData(false);
    }
  }

  useEffect(() => {
    void refreshSelectedAppData(selectedApp);
  }, [selectedApp]);

  async function handleCreateApp(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    resetMessages();
    setIsCreatingApp(true);

    try {
      const payload: CreateAppRequest = {
        logical_name: newAppLogicalName,
        display_name: newAppDisplayName,
        description:
          newAppDescription.trim().length > 0 ? newAppDescription : null,
      };
      const response = await apiFetch("/api/apps", {
        method: "POST",
        body: JSON.stringify(payload),
      });

      if (!response.ok) {
        const payload = (await response.json()) as { message?: string };
        setErrorMessage(payload.message ?? "Unable to create app.");
        return;
      }

      setNewAppLogicalName("");
      setNewAppDisplayName("");
      setNewAppDescription("");
      setStatusMessage("App created.");
      router.refresh();
    } catch {
      setErrorMessage("Unable to create app.");
    } finally {
      setIsCreatingApp(false);
    }
  }

  async function handleBindEntity(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!selectedApp) {
      setErrorMessage("Select an app first.");
      return;
    }
    resetMessages();
    setIsBindingEntity(true);

    try {
      const payload: BindAppEntityRequest = {
        entity_logical_name: entityToBind,
        navigation_label:
          navigationLabel.trim().length > 0 ? navigationLabel : null,
        navigation_order: navigationOrder,
      };
      const response = await apiFetch(`/api/apps/${selectedApp}/entities`, {
        method: "POST",
        body: JSON.stringify(payload),
      });

      if (!response.ok) {
        const payload = (await response.json()) as { message?: string };
        setErrorMessage(payload.message ?? "Unable to bind entity.");
        return;
      }

      setNavigationLabel("");
      setStatusMessage("Entity binding saved.");
      await refreshSelectedAppData(selectedApp);
    } catch {
      setErrorMessage("Unable to bind entity.");
    } finally {
      setIsBindingEntity(false);
    }
  }

  async function handleSavePermission(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!selectedApp) {
      setErrorMessage("Select an app first.");
      return;
    }

    resetMessages();
    setIsSavingPermission(true);

    try {
      const payload: SaveAppRoleEntityPermissionRequest = {
        role_name: permissionRoleName,
        entity_logical_name: permissionEntityName,
        can_read: canRead,
        can_create: canCreate,
        can_update: canUpdate,
        can_delete: canDelete,
      };
      const response = await apiFetch(`/api/apps/${selectedApp}/permissions`, {
        method: "PUT",
        body: JSON.stringify(payload),
      });

      if (!response.ok) {
        const payload = (await response.json()) as { message?: string };
        setErrorMessage(payload.message ?? "Unable to save app permissions.");
        return;
      }

      setStatusMessage("Role permissions saved.");
      await refreshSelectedAppData(selectedApp);
    } catch {
      setErrorMessage("Unable to save app permissions.");
    } finally {
      setIsSavingPermission(false);
    }
  }

  return (
    <div className="space-y-6">
      {!hasStudioData ? (
        <p className="rounded-md border border-amber-200 bg-amber-50 px-3 py-2 text-sm text-amber-800">
          Create at least one app, one entity, and one role before configuring
          app access.
        </p>
      ) : null}

      <form
        className="grid gap-3 rounded-md border border-emerald-100 bg-white p-4 md:grid-cols-3"
        onSubmit={handleCreateApp}
      >
        <div className="space-y-2">
          <Label htmlFor="new_app_logical_name">App Logical Name</Label>
          <Input
            id="new_app_logical_name"
            value={newAppLogicalName}
            onChange={(event) => setNewAppLogicalName(event.target.value)}
            placeholder="sales"
            required
          />
        </div>
        <div className="space-y-2">
          <Label htmlFor="new_app_display_name">App Display Name</Label>
          <Input
            id="new_app_display_name"
            value={newAppDisplayName}
            onChange={(event) => setNewAppDisplayName(event.target.value)}
            placeholder="Sales App"
            required
          />
        </div>
        <div className="space-y-2">
          <Label htmlFor="new_app_description">Description</Label>
          <Input
            id="new_app_description"
            value={newAppDescription}
            onChange={(event) => setNewAppDescription(event.target.value)}
            placeholder="Lead and account workflows"
          />
        </div>
        <div className="md:col-span-3">
          <Button disabled={isCreatingApp} type="submit">
            {isCreatingApp ? "Creating..." : "Create App"}
          </Button>
        </div>
      </form>

      <div className="space-y-3 rounded-md border border-emerald-100 bg-white p-4">
        <div className="space-y-2">
          <Label htmlFor="studio_app_selector">Active App</Label>
          <Select
            id="studio_app_selector"
            value={selectedApp}
            onChange={(event) => setSelectedApp(event.target.value)}
          >
            {apps.map((app) => (
              <option key={app.logical_name} value={app.logical_name}>
                {app.display_name} ({app.logical_name})
              </option>
            ))}
          </Select>
        </div>

        <form className="grid gap-3 md:grid-cols-3" onSubmit={handleBindEntity}>
          <div className="space-y-2">
            <Label htmlFor="bind_entity_name">Entity</Label>
            <Select
              id="bind_entity_name"
              value={entityToBind}
              onChange={(event) => setEntityToBind(event.target.value)}
            >
              {entities.map((entity) => (
                <option key={entity.logical_name} value={entity.logical_name}>
                  {entity.display_name} ({entity.logical_name})
                </option>
              ))}
            </Select>
          </div>

          <div className="space-y-2">
            <Label htmlFor="bind_navigation_label">Navigation Label</Label>
            <Input
              id="bind_navigation_label"
              value={navigationLabel}
              onChange={(event) => setNavigationLabel(event.target.value)}
              placeholder="Accounts"
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="bind_navigation_order">Navigation Order</Label>
            <Input
              id="bind_navigation_order"
              value={String(navigationOrder)}
              onChange={(event) =>
                setNavigationOrder(
                  Number.parseInt(event.target.value || "0", 10),
                )
              }
              type="number"
              min={0}
            />
          </div>

          <div className="md:col-span-3">
            <Button
              disabled={isBindingEntity || isLoadingAppData}
              type="submit"
              variant="outline"
            >
              {isBindingEntity
                ? "Saving..."
                : `Bind Entity to ${selectedAppDisplayName}`}
            </Button>
          </div>
        </form>

        <form
          className="grid gap-3 md:grid-cols-4"
          onSubmit={handleSavePermission}
        >
          <div className="space-y-2">
            <Label htmlFor="permission_role_name">Role</Label>
            <Select
              id="permission_role_name"
              value={permissionRoleName}
              onChange={(event) => setPermissionRoleName(event.target.value)}
            >
              {roles.map((role) => (
                <option key={role.role_id} value={role.name}>
                  {role.name}
                </option>
              ))}
            </Select>
          </div>

          <div className="space-y-2">
            <Label htmlFor="permission_entity_name">Entity</Label>
            <Select
              id="permission_entity_name"
              value={permissionEntityName}
              onChange={(event) => setPermissionEntityName(event.target.value)}
            >
              {entities.map((entity) => (
                <option key={entity.logical_name} value={entity.logical_name}>
                  {entity.display_name} ({entity.logical_name})
                </option>
              ))}
            </Select>
          </div>

          <div className="space-y-1 pt-6 md:col-span-2">
            <div className="mr-3 inline-flex items-center gap-1 text-sm">
              <Checkbox
                id="permission_can_read"
                checked={canRead}
                onChange={(event) => setCanRead(event.target.checked)}
              />
              <Label htmlFor="permission_can_read">Read</Label>
            </div>
            <div className="mr-3 inline-flex items-center gap-1 text-sm">
              <Checkbox
                id="permission_can_create"
                checked={canCreate}
                onChange={(event) => setCanCreate(event.target.checked)}
              />
              <Label htmlFor="permission_can_create">Create</Label>
            </div>
            <div className="mr-3 inline-flex items-center gap-1 text-sm">
              <Checkbox
                id="permission_can_update"
                checked={canUpdate}
                onChange={(event) => setCanUpdate(event.target.checked)}
              />
              <Label htmlFor="permission_can_update">Update</Label>
            </div>
            <div className="inline-flex items-center gap-1 text-sm">
              <Checkbox
                id="permission_can_delete"
                checked={canDelete}
                onChange={(event) => setCanDelete(event.target.checked)}
              />
              <Label htmlFor="permission_can_delete">Delete</Label>
            </div>
          </div>

          <div className="md:col-span-4">
            <Button
              disabled={isSavingPermission || isLoadingAppData}
              type="submit"
              variant="outline"
            >
              {isSavingPermission
                ? "Saving..."
                : "Save Role Entity Permissions"}
            </Button>
          </div>
        </form>

        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Bound Entity</TableHead>
              <TableHead>Label</TableHead>
              <TableHead>Order</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {bindings.length > 0 ? (
              bindings.map((binding) => (
                <TableRow
                  key={`${binding.app_logical_name}.${binding.entity_logical_name}`}
                >
                  <TableCell className="font-mono text-xs">
                    {binding.entity_logical_name}
                  </TableCell>
                  <TableCell>
                    {binding.navigation_label ?? binding.entity_logical_name}
                  </TableCell>
                  <TableCell>{binding.navigation_order}</TableCell>
                </TableRow>
              ))
            ) : (
              <TableRow>
                <TableCell colSpan={3} className="text-zinc-500">
                  No entity bindings for this app.
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>

        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Role</TableHead>
              <TableHead>Entity</TableHead>
              <TableHead>Permissions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {permissions.length > 0 ? (
              permissions.map((permission) => (
                <TableRow
                  key={`${permission.app_logical_name}.${permission.role_name}.${permission.entity_logical_name}`}
                >
                  <TableCell>{permission.role_name}</TableCell>
                  <TableCell className="font-mono text-xs">
                    {permission.entity_logical_name}
                  </TableCell>
                  <TableCell className="font-mono text-xs">
                    {[
                      permission.can_read ? "read" : null,
                      permission.can_create ? "create" : null,
                      permission.can_update ? "update" : null,
                      permission.can_delete ? "delete" : null,
                    ]
                      .filter((value): value is string => value !== null)
                      .join(", ") || "none"}
                  </TableCell>
                </TableRow>
              ))
            ) : (
              <TableRow>
                <TableCell colSpan={3} className="text-zinc-500">
                  No role entity permissions configured for this app.
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>

      {errorMessage ? (
        <p className="rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
          {errorMessage}
        </p>
      ) : null}
      {statusMessage ? (
        <p className="rounded-md border border-emerald-200 bg-emerald-50 px-3 py-2 text-sm text-emerald-700">
          {statusMessage}
        </p>
      ) : null}
    </div>
  );
}
