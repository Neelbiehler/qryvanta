export type AppEntityViewMode = "grid" | "json";
export type AppStudioSection = "apps" | "navigation" | "permissions";

export type NewAppDraft = {
  logicalName: string;
  displayName: string;
  description: string;
};

export type BindingDraft = {
  entityToBind: string;
  navigationLabel: string;
  navigationOrder: number;
  forms: AppSurfaceDraft[];
  listViews: AppSurfaceDraft[];
  defaultFormLogicalName: string;
  defaultListViewLogicalName: string;
  defaultViewMode: AppEntityViewMode;
};

export type AppSurfaceDraft = {
  logicalName: string;
  displayName: string;
  fieldLogicalNames: string[];
};

export type PermissionDraft = {
  roleName: string;
  entityName: string;
  canRead: boolean;
  canCreate: boolean;
  canUpdate: boolean;
  canDelete: boolean;
};
