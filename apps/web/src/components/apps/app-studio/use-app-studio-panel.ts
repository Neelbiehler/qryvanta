import { type FormEvent, useCallback, useEffect, useState } from "react";
import { useRouter } from "next/navigation";

import {
  apiFetch,
  type AppEntityBindingResponse,
  type AppEntityFormDto,
  type AppResponse,
  type AppRoleEntityPermissionResponse,
  type AppEntityViewDto,
  type BindAppEntityRequest,
  type CreateAppRequest,
  type EntityResponse,
  type FieldResponse,
  type RoleResponse,
  type SaveAppRoleEntityPermissionRequest,
} from "@/lib/api";
import type {
  AppSurfaceDraft,
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
  const [selectedEntityFields, setSelectedEntityFields] = useState<FieldResponse[]>([]);
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
    forms: [
      {
        logicalName: "main_form",
        displayName: "Main Form",
        fieldLogicalNames: [],
      },
    ],
    listViews: [
      {
        logicalName: "main_view",
        displayName: "Main View",
        fieldLogicalNames: [],
      },
    ],
    defaultFormLogicalName: "main_form",
    defaultListViewLogicalName: "main_view",
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
  const [isLoadingEntityFields, setIsLoadingEntityFields] = useState(false);

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

  const refreshSelectedEntityFields = useCallback(async (entityLogicalName: string) => {
    if (!entityLogicalName) {
      setSelectedEntityFields([]);
      return;
    }

    setIsLoadingEntityFields(true);
    try {
      const response = await apiFetch(`/api/entities/${entityLogicalName}/fields`);
      if (!response.ok) {
        setSelectedEntityFields([]);
        setErrorMessage("Unable to load entity field catalog.");
        return;
      }

      setSelectedEntityFields((await response.json()) as FieldResponse[]);
    } catch {
      setSelectedEntityFields([]);
      setErrorMessage("Unable to load entity field catalog.");
    } finally {
      setIsLoadingEntityFields(false);
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
          forms: [
            {
              logicalName: "main_form",
              displayName: "Main Form",
              fieldLogicalNames: [],
            },
          ],
          listViews: [
            {
              logicalName: "main_view",
              displayName: "Main View",
              fieldLogicalNames: [],
            },
          ],
          defaultFormLogicalName: "main_form",
          defaultListViewLogicalName: "main_view",
          defaultViewMode: "grid",
        }));
        return;
      }

      const forms = mapSurfaceDtos(existingBinding.forms);
      const listViews = mapSurfaceDtos(existingBinding.list_views);

      const defaultFormLogicalName = forms.some(
        (form) => form.logicalName === existingBinding.default_form_logical_name,
      )
        ? existingBinding.default_form_logical_name
        : forms[0]?.logicalName ?? "main_form";
      const defaultListViewLogicalName = listViews.some(
        (view) => view.logicalName === existingBinding.default_list_view_logical_name,
      )
        ? existingBinding.default_list_view_logical_name
        : listViews[0]?.logicalName ?? "main_view";

      setBindingDraft((current) => ({
        ...current,
        navigationLabel: existingBinding.navigation_label ?? "",
        navigationOrder: existingBinding.navigation_order,
        forms,
        listViews,
        defaultFormLogicalName,
        defaultListViewLogicalName,
        defaultViewMode: existingBinding.default_view_mode,
      }));
  }, [bindings, bindingDraft.entityToBind]);

  useEffect(() => {
    void refreshSelectedEntityFields(bindingDraft.entityToBind);
  }, [bindingDraft.entityToBind, refreshSelectedEntityFields]);

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

    if (bindingDraft.forms.length === 0 || bindingDraft.listViews.length === 0) {
      setErrorMessage("At least one form and one list view are required.");
      return;
    }

    if (
      !bindingDraft.forms.some(
        (form) => form.logicalName === bindingDraft.defaultFormLogicalName,
      ) ||
      !bindingDraft.listViews.some(
        (view) => view.logicalName === bindingDraft.defaultListViewLogicalName,
      )
    ) {
      setErrorMessage("Default form and default list view must be selected.");
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
        forms: bindingDraft.forms.map((form) => ({
          logical_name: form.logicalName,
          display_name: form.displayName,
          field_logical_names: form.fieldLogicalNames,
        })),
        list_views: bindingDraft.listViews.map((view) => ({
          logical_name: view.logicalName,
          display_name: view.displayName,
          field_logical_names: view.fieldLogicalNames,
        })),
        default_form_logical_name: bindingDraft.defaultFormLogicalName,
        default_list_view_logical_name: bindingDraft.defaultListViewLogicalName,
        form_field_logical_names: resolveFallbackFieldMapping(
          bindingDraft.forms,
          bindingDraft.defaultFormLogicalName,
        ),
        list_field_logical_names: resolveFallbackFieldMapping(
          bindingDraft.listViews,
          bindingDraft.defaultListViewLogicalName,
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
    selectedEntityFields,
    selectedApp,
    selectedAppDisplayName,
    isLoadingEntityFields,
    setActiveSection,
    setBindingDraft,
    setNewAppDraft,
    setPermissionDraft,
    setSelectedApp,
  };
}

function mapSurfaceDtos(
  surfaces: AppEntityFormDto[] | AppEntityViewDto[],
): AppSurfaceDraft[] {
  if (surfaces.length === 0) {
    return [];
  }

  return surfaces.map((surface) => ({
    logicalName: surface.logical_name,
    displayName: surface.display_name,
    fieldLogicalNames: surface.field_logical_names,
  }));
}

function resolveFallbackFieldMapping(
  surfaces: AppSurfaceDraft[],
  defaultLogicalName: string,
): string[] {
  return (
    surfaces.find((surface) => surface.logicalName === defaultLogicalName)
      ?.fieldLogicalNames ?? []
  );
}
