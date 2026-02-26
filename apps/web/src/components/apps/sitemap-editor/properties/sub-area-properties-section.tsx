import { Input, Label, Select } from "@qryvanta/ui";

import type {
  SubAreaEntityContext,
  SubAreaSelection,
  SitemapSubArea,
  UpdateSitemap,
} from "@/components/apps/sitemap-editor/properties/types";
import { updateSubArea } from "@/components/apps/sitemap-editor/properties/sub-area-properties-utils";
import { SubAreaCustomPageTargetProperties } from "@/components/apps/sitemap-editor/properties/sub-area-target-custom-page-properties";
import { SubAreaDashboardTargetProperties } from "@/components/apps/sitemap-editor/properties/sub-area-target-dashboard-properties";
import { SubAreaEntityTargetProperties } from "@/components/apps/sitemap-editor/properties/sub-area-target-entity-properties";
import type { AppSitemapTargetDto } from "@/lib/api";

type SubAreaPropertiesSectionProps = {
  selection: SubAreaSelection;
  selectedSubArea: SitemapSubArea;
  onUpdateSitemap: UpdateSitemap;
} & SubAreaEntityContext;

export function SubAreaPropertiesSection({
  selection,
  selectedSubArea,
  entities,
  selectedEntityForms,
  selectedEntityViews,
  isLoadingTargetMetadata,
  onUpdateSitemap,
}: SubAreaPropertiesSectionProps) {
  return (
    <>
      <div className="space-y-2">
        <Label htmlFor="sub_area_display_name">Sub Area Display Name</Label>
        <Input
          id="sub_area_display_name"
          value={selectedSubArea.display_name}
          onChange={(event) =>
            onUpdateSitemap((current) =>
              updateSubArea(current, selection, (subArea) => ({
                ...subArea,
                display_name: event.target.value,
              })),
            )
          }
        />
      </div>
      <div className="space-y-2">
        <Label htmlFor="sub_area_icon">Sub Area Icon</Label>
        <Input
          id="sub_area_icon"
          value={selectedSubArea.icon ?? ""}
          onChange={(event) =>
            onUpdateSitemap((current) =>
              updateSubArea(current, selection, (subArea) => ({
                ...subArea,
                icon: event.target.value.trim().length > 0 ? event.target.value : null,
              })),
            )
          }
        />
      </div>
      <div className="space-y-2">
        <Label htmlFor="sub_area_target_type">Target Type</Label>
        <Select
          id="sub_area_target_type"
          value={selectedSubArea.target.type}
          onChange={(event) =>
            onUpdateSitemap((current) => {
              const nextType = event.target.value as AppSitemapTargetDto["type"];
              const nextTarget: AppSitemapTargetDto =
                nextType === "entity"
                  ? {
                      type: "entity",
                      entity_logical_name: entities[0]?.logical_name ?? "",
                      default_form: null,
                      default_view: null,
                    }
                  : nextType === "dashboard"
                    ? {
                        type: "dashboard",
                        dashboard_logical_name: "",
                      }
                    : {
                        type: "custom_page",
                        url: "",
                      };
              return updateSubArea(current, selection, (subArea) => ({
                ...subArea,
                target: nextTarget,
              }));
            })
          }
        >
          <option value="entity">Entity</option>
          <option value="dashboard">Dashboard</option>
          <option value="custom_page">Custom Page</option>
        </Select>
      </div>

      {selectedSubArea.target.type === "entity" ? (
        <SubAreaEntityTargetProperties
          selection={selection}
          target={selectedSubArea.target}
          entities={entities}
          selectedEntityForms={selectedEntityForms}
          selectedEntityViews={selectedEntityViews}
          isLoadingTargetMetadata={isLoadingTargetMetadata}
          onUpdateSitemap={onUpdateSitemap}
        />
      ) : null}

      {selectedSubArea.target.type === "dashboard" ? (
        <SubAreaDashboardTargetProperties
          selection={selection}
          target={selectedSubArea.target}
          onUpdateSitemap={onUpdateSitemap}
        />
      ) : null}

      {selectedSubArea.target.type === "custom_page" ? (
        <SubAreaCustomPageTargetProperties
          selection={selection}
          target={selectedSubArea.target}
          onUpdateSitemap={onUpdateSitemap}
        />
      ) : null}
    </>
  );
}
