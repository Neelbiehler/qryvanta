import { Button, StatusBadge } from "@qryvanta/ui";

import type { AppStudioSection } from "@/components/apps/app-studio/sections/types";

type AppStudioOverviewProps = {
  activeSection: AppStudioSection;
  appsCount: number;
  canOpenNavigation: boolean;
  canOpenPermissions: boolean;
  entitiesCount: number;
  hasStudioData: boolean;
  onSectionChange: (section: AppStudioSection) => void;
  rolesCount: number;
  selectedAppDisplayName: string;
};

export function AppStudioOverview({
  activeSection,
  appsCount,
  canOpenNavigation,
  canOpenPermissions,
  entitiesCount,
  hasStudioData,
  onSectionChange,
  rolesCount,
  selectedAppDisplayName,
}: AppStudioOverviewProps) {
  return (
    <>
      {!hasStudioData ? (
        <p className="rounded-md border border-amber-200 bg-amber-50 px-3 py-2 text-sm text-amber-800">
          Create at least one app, one entity, and one role before configuring app access.
        </p>
      ) : null}

      <div className="flex flex-wrap items-center gap-2 rounded-md border border-emerald-100 bg-white/90 p-3">
        <StatusBadge tone="neutral">Apps {appsCount}</StatusBadge>
        <StatusBadge tone="neutral">Entities {entitiesCount}</StatusBadge>
        <StatusBadge tone="neutral">Roles {rolesCount}</StatusBadge>
        <StatusBadge tone="success">Active {selectedAppDisplayName}</StatusBadge>
      </div>

      <div className="flex flex-wrap gap-2">
        <Button
          type="button"
          variant={activeSection === "apps" ? "default" : "outline"}
          onClick={() => onSectionChange("apps")}
        >
          App Catalog
        </Button>
        <Button
          type="button"
          variant={activeSection === "navigation" ? "default" : "outline"}
          onClick={() => onSectionChange("navigation")}
          disabled={!canOpenNavigation}
        >
          Navigation Binding
        </Button>
        <Button
          type="button"
          variant={activeSection === "permissions" ? "default" : "outline"}
          onClick={() => onSectionChange("permissions")}
          disabled={!canOpenPermissions}
        >
          Role Permissions
        </Button>
      </div>
    </>
  );
}
