import { type FormEvent, useCallback, useEffect, useState } from "react";
import { useRouter } from "next/navigation";

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
import { parseLogicalNameList } from "@/components/apps/app-studio/helpers";
import type {
  AppStudioSection,
  BindingDraft,
  NewAppDraft,
  PermissionDraft,
} from "@/components/apps/app-studio/sections";

type PanelMessages = {
  errorMessage: string | null;
  statusMessage: string | null;
};

type PendingState = {
  isCreatingApp: boolean;
  isBindingEntity: boolean;
  isSavingPermission: boolean;
  isLoadingAppData: boolean;
};

type SelectionState = {
  activeSection: AppStudioSection;
  selectedApp: string;
};

type UseAppStudioPanelInput = {
  apps: AppResponse[];
  entities: EntityResponse[];
  roles: RoleResponse[];
};

export function useAppStudioPanel({
  apps,
  entities,
  roles,
}: UseAppStudioPanelInput) {
  const router = useRouter();

  const [selectionState, setSelectionState] = useState<SelectionState>({
    selectedApp: apps.at(0)?.logical_name ?? "",
    activeSection: apps.length > 0 ? "navigation" : "apps",
  });
  const [bindings, setBindings] = useState<AppEntityBindingResponse[]>([]);
  const [permissions, setPermissions] = useState<
    AppRoleEntityPermissionResponse[]
  >([]);
  const [messages, setMessages] = useState<PanelMessages>({
    errorMessage: null,
    statusMessage: null,
  });
  const [newAppDraft, setNewAppDraft] = useState<NewAppDraft>({
    logicalName: "",
    displayName: "",
    description: "",
  });
  const [bindingDraft, setBindingDraft] = useState<BindingDraft>({
    entityToBind: entities.at(0)?.logical_name ?? "",
    navigationLabel: "",
    navigationOrder: 0,
    formFieldLogicalNames: "",
    listFieldLogicalNames: "",
    defaultViewMode: "grid",
  });
  const [permissionDraft, setPermissionDraft] = useState<PermissionDraft>({
    roleName: roles.at(0)?.name ?? "",
    entityName: entities.at(0)?.logical_name ?? "",
    canRead: true,
    canCreate: false,
    canUpdate: false,
    canDelete: false,
  });
  const [pendingState, setPendingState] = useState<PendingState>({
    isCreatingApp: false,
    isBindingEntity: false,
    isSavingPermission: false,
    isLoadingAppData: false,
  });

  const selectedApp = selectionState.selectedApp;
  const activeSection = selectionState.activeSection;

  const hasStudioData =
    apps.length > 0 && entities.length > 0 && roles.length > 0;

  const selectedAppDisplayName =
    apps.find((app) => app.logical_name === selectedApp)?.display_name ??
    selectedApp;

  function setSelectedApp(selectedAppValue: string) {
    setSelectionState((current) => ({
      ...current,
      selectedApp: selectedAppValue,
    }));
  }

  function setActiveSection(activeSectionValue: AppStudioSection) {
    setSelectionState((current) => ({
      ...current,
      activeSection: activeSectionValue,
    }));
  }

  function setErrorMessage(next: string | null) {
    setMessages((current) => ({ ...current, errorMessage: next }));
  }

  function setStatusMessage(next: string | null) {
    setMessages((current) => ({ ...current, statusMessage: next }));
  }

  function resetMessages() {
    setMessages({ errorMessage: null, statusMessage: null });
  }

  const refreshSelectedAppData = useCallback(async (appLogicalName: string) => {
    if (!appLogicalName) {
      setBindings([]);
      setPermissions([]);
      return;
    }

    setPendingState((current) => ({ ...current, isLoadingAppData: true }));
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
      setPendingState((current) => ({ ...current, isLoadingAppData: false }));
    }
  }, []);

  useEffect(() => {
    void refreshSelectedAppData(selectedApp);
  }, [refreshSelectedAppData, selectedApp]);

  useEffect(() => {
    const existingBinding = bindings.find(
      (binding) => binding.entity_logical_name === bindingDraft.entityToBind,
    );

    if (!existingBinding) {
      setBindingDraft((current) => ({
        ...current,
        navigationLabel: "",
        navigationOrder: 0,
        formFieldLogicalNames: "",
        listFieldLogicalNames: "",
        defaultViewMode: "grid",
      }));
      return;
    }

    setBindingDraft((current) => ({
      ...current,
      navigationLabel: existingBinding.navigation_label ?? "",
      navigationOrder: existingBinding.navigation_order,
      formFieldLogicalNames: existingBinding.form_field_logical_names.join(", "),
      listFieldLogicalNames: existingBinding.list_field_logical_names.join(", "),
      defaultViewMode: existingBinding.default_view_mode,
    }));
  }, [bindings, bindingDraft.entityToBind]);

  async function handleCreateApp(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    resetMessages();
    setPendingState((current) => ({ ...current, isCreatingApp: true }));

    try {
      const payload: CreateAppRequest = {
        logical_name: newAppDraft.logicalName,
        display_name: newAppDraft.displayName,
        description:
          newAppDraft.description.trim().length > 0
            ? newAppDraft.description
            : null,
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

      setNewAppDraft({ logicalName: "", displayName: "", description: "" });
      setStatusMessage("App created.");
      router.refresh();
    } catch {
      setErrorMessage("Unable to create app.");
    } finally {
      setPendingState((current) => ({ ...current, isCreatingApp: false }));
    }
  }

  async function handleBindEntity(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!selectedApp) {
      setErrorMessage("Select an app first.");
      return;
    }

    resetMessages();
    setPendingState((current) => ({ ...current, isBindingEntity: true }));

    try {
      const payload: BindAppEntityRequest = {
        entity_logical_name: bindingDraft.entityToBind,
        navigation_label:
          bindingDraft.navigationLabel.trim().length > 0
            ? bindingDraft.navigationLabel
            : null,
        navigation_order: bindingDraft.navigationOrder,
        form_field_logical_names: parseLogicalNameList(
          bindingDraft.formFieldLogicalNames,
        ),
        list_field_logical_names: parseLogicalNameList(
          bindingDraft.listFieldLogicalNames,
        ),
        default_view_mode: bindingDraft.defaultViewMode,
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

      setBindingDraft((current) => ({ ...current, navigationLabel: "" }));
      setStatusMessage("Entity binding saved.");
      await refreshSelectedAppData(selectedApp);
    } catch {
      setErrorMessage("Unable to bind entity.");
    } finally {
      setPendingState((current) => ({ ...current, isBindingEntity: false }));
    }
  }

  async function handleSavePermission(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!selectedApp) {
      setErrorMessage("Select an app first.");
      return;
    }

    resetMessages();
    setPendingState((current) => ({ ...current, isSavingPermission: true }));

    try {
      const payload: SaveAppRoleEntityPermissionRequest = {
        role_name: permissionDraft.roleName,
        entity_logical_name: permissionDraft.entityName,
        can_read: permissionDraft.canRead,
        can_create: permissionDraft.canCreate,
        can_update: permissionDraft.canUpdate,
        can_delete: permissionDraft.canDelete,
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
      setPendingState((current) => ({ ...current, isSavingPermission: false }));
    }
  }

  return {
    activeSection,
    bindingDraft,
    bindings,
    handleBindEntity,
    handleCreateApp,
    handleSavePermission,
    hasStudioData,
    messages,
    newAppDraft,
    pendingState,
    permissionDraft,
    permissions,
    selectedApp,
    selectedAppDisplayName,
    setActiveSection,
    setBindingDraft,
    setNewAppDraft,
    setPermissionDraft,
    setSelectedApp,
  };
}
