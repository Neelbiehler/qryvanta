import { type FormEvent, useCallback, useEffect, useState } from "react";
import { useRouter } from "next/navigation";

import {
  apiFetch,
  type AppEntityBindingResponse,
  type AppEntityFormDto,
  type AppPublishChecksResponse,
  type AppPublishDiffResponse,
  type EntityPublishDiffResponse,
  type PublishCheckIssueResponse,
  type AppResponse,
  type RunWorkspacePublishRequest,
  type RunWorkspacePublishResponse,
  type WorkspacePublishHistoryEntryResponse,
  type AppRoleEntityPermissionResponse,
  type AppEntityViewDto,
  type BindAppEntityRequest,
  type CreateAppRequest,
  type EntityResponse,
  type FieldResponse,
  type RoleResponse,
  type SaveAppRoleEntityPermissionRequest,
  type WorkspacePublishChecksResponse,
  type WorkspacePublishDiffRequest,
  type WorkspacePublishDiffResponse,
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
  isReorderingBinding: boolean;
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

type WorkspacePublishDraft = {
  entityLogicalNames: string[];
  appLogicalNames: string[];
};

type PublishRunHistoryEntry = {
  runId: string;
  runAt: string;
  subject: string;
  requestedEntities: number;
  requestedApps: number;
  requestedEntityLogicalNames: string[];
  requestedAppLogicalNames: string[];
  publishedEntities: string[];
  validatedApps: string[];
  issueCount: number;
  isPublishable: boolean;
};

type SelectionValidationState = {
  selectionKey: string | null;
  isPublishable: boolean;
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
  const [selectedEntityFields, setSelectedEntityFields] = useState<
    FieldResponse[]
  >([]);
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
    isReorderingBinding: false,
    isSavingPermission: false,
    isLoadingAppData: false,
  });
  const [isLoadingEntityFields, setIsLoadingEntityFields] = useState(false);
  const [isRunningPublishChecks, setIsRunningPublishChecks] = useState(false);
  const [publishCheckErrors, setPublishCheckErrors] = useState<string[]>([]);
  const [isRunningWorkspaceChecks, setIsRunningWorkspaceChecks] =
    useState(false);
  const [isRunningSelectionChecks, setIsRunningSelectionChecks] =
    useState(false);
  const [workspaceIssues, setWorkspaceIssues] = useState<
    PublishCheckIssueResponse[]
  >([]);
  const [workspaceCheckSummary, setWorkspaceCheckSummary] = useState<{
    checkedEntities: number;
    checkedApps: number;
  } | null>(null);
  const [workspacePublishDraft, setWorkspacePublishDraft] =
    useState<WorkspacePublishDraft>({
      entityLogicalNames: entities.map((entity) => entity.logical_name),
      appLogicalNames: apps.map((app) => app.logical_name),
    });
  const [isRunningSelectivePublish, setIsRunningSelectivePublish] =
    useState(false);
  const [selectionValidation, setSelectionValidation] =
    useState<SelectionValidationState>({
      selectionKey: null,
      isPublishable: false,
    });
  const [publishHistory, setPublishHistory] = useState<
    PublishRunHistoryEntry[]
  >([]);
  const [publishDiff, setPublishDiff] = useState<{
    unknownEntityLogicalNames: string[];
    unknownAppLogicalNames: string[];
    entityDiffs: EntityPublishDiffResponse[];
    appDiffs: AppPublishDiffResponse[];
  }>({
    unknownEntityLogicalNames: [],
    unknownAppLogicalNames: [],
    entityDiffs: [],
    appDiffs: [],
  });

  const selectedApp = selectionState.selectedApp;
  const activeSection = selectionState.activeSection;

  const hasStudioData =
    apps.length > 0 && entities.length > 0 && roles.length > 0;

  const workspaceSelectionKey = buildWorkspaceSelectionKey(
    workspacePublishDraft,
  );

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

  function clearPublishChecks() {
    setPublishCheckErrors([]);
  }

  function clearWorkspaceChecks() {
    setWorkspaceIssues([]);
    setWorkspaceCheckSummary(null);
  }

  const refreshPublishDiff = useCallback(async (): Promise<void> => {
    const payload: WorkspacePublishDiffRequest = {
      entity_logical_names: workspacePublishDraft.entityLogicalNames,
      app_logical_names: workspacePublishDraft.appLogicalNames,
    };

    const response = await apiFetch("/api/publish/diff", {
      method: "POST",
      body: JSON.stringify(payload),
    });
    if (!response.ok) {
      throw new Error("Unable to load publish diff.");
    }

    const result = (await response.json()) as WorkspacePublishDiffResponse;
    setPublishDiff({
      unknownEntityLogicalNames: result.unknown_entity_logical_names,
      unknownAppLogicalNames: result.unknown_app_logical_names,
      entityDiffs: result.entity_diffs,
      appDiffs: result.app_diffs,
    });
  }, [
    workspacePublishDraft.appLogicalNames,
    workspacePublishDraft.entityLogicalNames,
  ]);

  const refreshPublishHistory = useCallback(async () => {
    try {
      const response = await apiFetch("/api/publish/history?limit=12");
      if (!response.ok) {
        setPublishHistory([]);
        return;
      }

      const entries =
        (await response.json()) as WorkspacePublishHistoryEntryResponse[];
      setPublishHistory(
        entries.map((entry) => ({
          runId: entry.run_id,
          runAt: entry.run_at,
          subject: entry.subject,
          requestedEntities: entry.requested_entities,
          requestedApps: entry.requested_apps,
          requestedEntityLogicalNames: entry.requested_entity_logical_names,
          requestedAppLogicalNames: entry.requested_app_logical_names,
          publishedEntities: entry.published_entities,
          validatedApps: entry.validated_apps,
          issueCount: entry.issue_count,
          isPublishable: entry.is_publishable,
        })),
      );
    } catch {
      setPublishHistory([]);
    }
  }, []);

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

  const refreshSelectedEntityFields = useCallback(
    async (entityLogicalName: string) => {
      if (!entityLogicalName) {
        setSelectedEntityFields([]);
        return;
      }

      setIsLoadingEntityFields(true);
      try {
        const response = await apiFetch(
          `/api/entities/${entityLogicalName}/fields`,
        );
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
    },
    [],
  );

  useEffect(() => {
    void refreshSelectedAppData(selectedApp);
  }, [refreshSelectedAppData, selectedApp]);

  useEffect(() => {
    setWorkspacePublishDraft((current) => ({
      entityLogicalNames: reconcileSelections(
        current.entityLogicalNames,
        entities.map((entity) => entity.logical_name),
      ),
      appLogicalNames: reconcileSelections(
        current.appLogicalNames,
        apps.map((app) => app.logical_name),
      ),
    }));
  }, [apps, entities]);

  useEffect(() => {
    setSelectionValidation((current) => {
      if (current.selectionKey === workspaceSelectionKey) {
        return current;
      }

      return {
        selectionKey: null,
        isPublishable: false,
      };
    });
  }, [workspaceSelectionKey]);

  useEffect(() => {
    void refreshPublishHistory();
  }, [refreshPublishHistory]);

  useEffect(() => {
    void refreshPublishDiff().catch(() => {
      setPublishDiff({
        unknownEntityLogicalNames: [],
        unknownAppLogicalNames: [],
        entityDiffs: [],
        appDiffs: [],
      });
    });
  }, [refreshPublishDiff]);

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
      : (forms[0]?.logicalName ?? "main_form");
    const defaultListViewLogicalName = listViews.some(
      (view) =>
        view.logicalName === existingBinding.default_list_view_logical_name,
    )
      ? existingBinding.default_list_view_logical_name
      : (listViews[0]?.logicalName ?? "main_view");

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
    clearPublishChecks();
    clearWorkspaceChecks();
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

    if (
      bindingDraft.forms.length === 0 ||
      bindingDraft.listViews.length === 0
    ) {
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
    clearPublishChecks();
    clearWorkspaceChecks();
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
    clearPublishChecks();
    clearWorkspaceChecks();
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

  async function handleReorderBinding(
    entityLogicalName: string,
    direction: "up" | "down",
  ) {
    if (!selectedApp) {
      setErrorMessage("Select an app first.");
      return;
    }

    const orderedBindings = [...bindings].sort((left, right) => {
      if (left.navigation_order !== right.navigation_order) {
        return left.navigation_order - right.navigation_order;
      }

      return left.entity_logical_name.localeCompare(right.entity_logical_name);
    });

    const currentIndex = orderedBindings.findIndex(
      (binding) => binding.entity_logical_name === entityLogicalName,
    );
    if (currentIndex < 0) {
      return;
    }

    const targetIndex =
      direction === "up" ? currentIndex - 1 : currentIndex + 1;
    if (targetIndex < 0 || targetIndex >= orderedBindings.length) {
      return;
    }

    const currentBinding = orderedBindings[currentIndex];
    const targetBinding = orderedBindings[targetIndex];

    resetMessages();
    clearPublishChecks();
    clearWorkspaceChecks();
    setPendingState((current) => ({ ...current, isReorderingBinding: true }));

    try {
      const saveBinding = async (
        binding: AppEntityBindingResponse,
        navigationOrder: number,
      ) => {
        const payload: BindAppEntityRequest = {
          entity_logical_name: binding.entity_logical_name,
          navigation_label: binding.navigation_label,
          navigation_order: navigationOrder,
          forms: binding.forms,
          list_views: binding.list_views,
          default_form_logical_name: binding.default_form_logical_name,
          default_list_view_logical_name:
            binding.default_list_view_logical_name,
          form_field_logical_names: resolveFallbackFieldMapping(
            mapSurfaceDtos(binding.forms),
            binding.default_form_logical_name,
          ),
          list_field_logical_names: resolveFallbackFieldMapping(
            mapSurfaceDtos(binding.list_views),
            binding.default_list_view_logical_name,
          ),
          default_view_mode: binding.default_view_mode,
        };

        const response = await apiFetch(`/api/apps/${selectedApp}/entities`, {
          method: "POST",
          body: JSON.stringify(payload),
        });

        if (!response.ok) {
          const errorPayload = (await response.json()) as { message?: string };
          throw new Error(
            errorPayload.message ?? "Unable to reorder sitemap binding.",
          );
        }
      };

      await saveBinding(currentBinding, targetBinding.navigation_order);
      await saveBinding(targetBinding, currentBinding.navigation_order);

      setStatusMessage("Sitemap order updated.");
      await refreshSelectedAppData(selectedApp);
    } catch {
      setErrorMessage("Unable to reorder sitemap binding.");
    } finally {
      setPendingState((current) => ({
        ...current,
        isReorderingBinding: false,
      }));
    }
  }

  async function handleRunPublishChecks() {
    if (!selectedApp) {
      setErrorMessage("Select an app first.");
      return;
    }

    resetMessages();
    clearWorkspaceChecks();
    setIsRunningPublishChecks(true);

    try {
      const response = await apiFetch(
        `/api/apps/${selectedApp}/publish-checks`,
      );
      if (!response.ok) {
        const payload = (await response.json()) as { message?: string };
        setErrorMessage(payload.message ?? "Unable to run app publish checks.");
        return;
      }

      const payload = (await response.json()) as AppPublishChecksResponse;
      setPublishCheckErrors(payload.errors);
      if (payload.is_publishable) {
        setStatusMessage("App publish checks passed.");
      } else {
        setErrorMessage(
          "App publish checks found issues. Resolve them before publishing.",
        );
      }
    } catch {
      setErrorMessage("Unable to run app publish checks.");
    } finally {
      setIsRunningPublishChecks(false);
    }
  }

  async function handleRunWorkspaceChecks() {
    resetMessages();
    clearPublishChecks();
    setIsRunningWorkspaceChecks(true);

    try {
      const response = await apiFetch("/api/publish/checks");
      if (!response.ok) {
        const payload = (await response.json()) as { message?: string };
        setErrorMessage(
          payload.message ?? "Unable to run workspace publish checks.",
        );
        return;
      }

      const payload = (await response.json()) as WorkspacePublishChecksResponse;
      setWorkspaceIssues(payload.issues);
      setWorkspaceCheckSummary({
        checkedEntities: payload.checked_entities,
        checkedApps: payload.checked_apps,
      });

      if (payload.is_publishable) {
        setStatusMessage("Workspace publish checks passed.");
      } else {
        setErrorMessage("Workspace publish checks found issues.");
      }

      await refreshPublishDiff();
    } catch {
      setErrorMessage("Unable to run workspace publish checks.");
    } finally {
      setIsRunningWorkspaceChecks(false);
    }
  }

  async function handleRunSelectionChecks() {
    resetMessages();
    clearPublishChecks();
    setIsRunningSelectionChecks(true);

    try {
      const payload: RunWorkspacePublishRequest = {
        entity_logical_names: workspacePublishDraft.entityLogicalNames,
        app_logical_names: workspacePublishDraft.appLogicalNames,
        dry_run: true,
      };

      const response = await apiFetch("/api/publish/checks", {
        method: "POST",
        body: JSON.stringify(payload),
      });
      if (!response.ok) {
        const body = (await response.json()) as { message?: string };
        setErrorMessage(
          body.message ?? "Unable to validate selective publish.",
        );
        return;
      }

      const result = (await response.json()) as RunWorkspacePublishResponse;
      setWorkspaceIssues(result.issues);
      setWorkspaceCheckSummary({
        checkedEntities: result.requested_entities,
        checkedApps: result.requested_apps,
      });

      setSelectionValidation({
        selectionKey: workspaceSelectionKey,
        isPublishable: result.is_publishable,
      });

      if (result.is_publishable) {
        setStatusMessage("Selection checks passed. Ready to publish.");
      } else {
        setErrorMessage("Selection checks found publish issues.");
      }

      await refreshPublishDiff();
    } catch {
      setErrorMessage("Unable to validate selective publish.");
    } finally {
      setIsRunningSelectionChecks(false);
    }
  }

  async function handleRunSelectivePublish() {
    if (selectionValidation.selectionKey !== workspaceSelectionKey) {
      setErrorMessage("Run selection checks before publishing.");
      return;
    }

    if (!selectionValidation.isPublishable) {
      setErrorMessage(
        "Selection is not publishable. Resolve issues before publishing.",
      );
      return;
    }

    resetMessages();
    clearPublishChecks();
    setIsRunningSelectivePublish(true);

    try {
      const payload: RunWorkspacePublishRequest = {
        entity_logical_names: workspacePublishDraft.entityLogicalNames,
        app_logical_names: workspacePublishDraft.appLogicalNames,
        dry_run: false,
      };

      const response = await apiFetch("/api/publish/checks", {
        method: "POST",
        body: JSON.stringify(payload),
      });
      if (!response.ok) {
        const body = (await response.json()) as { message?: string };
        setErrorMessage(body.message ?? "Unable to run selective publish.");
        return;
      }

      const result = (await response.json()) as RunWorkspacePublishResponse;
      setWorkspaceIssues(result.issues);
      setWorkspaceCheckSummary({
        checkedEntities: result.requested_entities,
        checkedApps: result.requested_apps,
      });

      await refreshPublishHistory();
      await refreshPublishDiff();
      setSelectionValidation({
        selectionKey: workspaceSelectionKey,
        isPublishable: result.is_publishable,
      });

      if (result.is_publishable) {
        setStatusMessage(
          `Selective publish complete: ${result.published_entities.length} entities published, ${result.validated_apps.length} apps validated.`,
        );
        return;
      }

      setErrorMessage("Selective publish blocked by publish issues.");
    } catch {
      setErrorMessage("Unable to run selective publish.");
    } finally {
      setIsRunningSelectivePublish(false);
    }
  }

  function applyWorkspacePublishSelection(
    entityLogicalNames: string[],
    appLogicalNames: string[],
  ) {
    setWorkspacePublishDraft({
      entityLogicalNames: reconcileSelections(
        entityLogicalNames,
        entities.map((entity) => entity.logical_name),
      ),
      appLogicalNames: reconcileSelections(
        appLogicalNames,
        apps.map((app) => app.logical_name),
      ),
    });
  }

  return {
    activeSection,
    bindingDraft,
    bindings,
    handleBindEntity,
    handleCreateApp,
    handleSavePermission,
    handleRunPublishChecks,
    handleReorderBinding,
    handleRunWorkspaceChecks,
    handleRunSelectionChecks,
    handleRunSelectivePublish,
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
    isRunningPublishChecks,
    isRunningWorkspaceChecks,
    isRunningSelectionChecks,
    isRunningSelectivePublish,
    selectionValidation,
    publishHistory,
    publishDiff,
    publishCheckErrors,
    workspacePublishDraft,
    workspaceIssues,
    workspaceCheckSummary,
    refreshPublishDiff,
    setActiveSection,
    setBindingDraft,
    setNewAppDraft,
    setPermissionDraft,
    setSelectedApp,
    applyWorkspacePublishSelection,
    setWorkspacePublishDraft,
  };
}

function buildWorkspaceSelectionKey(selection: WorkspacePublishDraft): string {
  const normalizedEntities = [...selection.entityLogicalNames].sort();
  const normalizedApps = [...selection.appLogicalNames].sort();

  return JSON.stringify({
    entityLogicalNames: normalizedEntities,
    appLogicalNames: normalizedApps,
  });
}

function reconcileSelections(
  selected: string[],
  available: string[],
): string[] {
  const availableSet = new Set(available);
  const retained = selected.filter((value) => availableSet.has(value));
  if (retained.length > 0) {
    return retained;
  }

  return available;
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
