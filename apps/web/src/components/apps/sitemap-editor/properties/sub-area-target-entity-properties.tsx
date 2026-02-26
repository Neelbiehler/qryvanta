import { Label, Select } from "@qryvanta/ui";

import { updateSubArea } from "@/components/apps/sitemap-editor/properties/sub-area-properties-utils";
import type {
  SubAreaEntityContext,
  SubAreaSelection,
  UpdateSitemap,
} from "@/components/apps/sitemap-editor/properties/types";
import type { AppSitemapTargetDto } from "@/lib/api";

type SubAreaEntityTargetPropertiesProps = {
  selection: SubAreaSelection;
  target: Extract<AppSitemapTargetDto, { type: "entity" }>;
  onUpdateSitemap: UpdateSitemap;
} & SubAreaEntityContext;

export function SubAreaEntityTargetProperties({
  selection,
  target,
  entities,
  selectedEntityForms,
  selectedEntityViews,
  isLoadingTargetMetadata,
  onUpdateSitemap,
}: SubAreaEntityTargetPropertiesProps) {
  return (
    <>
      <div className="space-y-2">
        <Label htmlFor="sub_area_target_entity">Entity</Label>
        <Select
          id="sub_area_target_entity"
          value={target.entity_logical_name}
          onChange={(event) =>
            onUpdateSitemap((current) =>
              updateSubArea(current, selection, (subArea) => {
                if (subArea.target.type !== "entity") {
                  return subArea;
                }

                return {
                  ...subArea,
                  target: {
                    ...subArea.target,
                    entity_logical_name: event.target.value,
                    default_form: null,
                    default_view: null,
                  },
                };
              }),
            )
          }
        >
          {entities.map((entity) => (
            <option key={entity.logical_name} value={entity.logical_name}>
              {entity.display_name} ({entity.logical_name})
            </option>
          ))}
        </Select>
      </div>

      <div className="space-y-2">
        <Label htmlFor="sub_area_target_form">Default Form</Label>
        <Select
          id="sub_area_target_form"
          value={target.default_form ?? ""}
          disabled={isLoadingTargetMetadata}
          onChange={(event) =>
            onUpdateSitemap((current) =>
              updateSubArea(current, selection, (subArea) => {
                if (subArea.target.type !== "entity") {
                  return subArea;
                }

                return {
                  ...subArea,
                  target: {
                    ...subArea.target,
                    default_form: event.target.value.trim().length > 0 ? event.target.value : null,
                  },
                };
              }),
            )
          }
        >
          <option value="">None</option>
          {selectedEntityForms.map((form) => (
            <option key={form.logical_name} value={form.logical_name}>
              {form.display_name} ({form.logical_name})
            </option>
          ))}
        </Select>
      </div>

      <div className="space-y-2">
        <Label htmlFor="sub_area_target_view">Default View</Label>
        <Select
          id="sub_area_target_view"
          value={target.default_view ?? ""}
          disabled={isLoadingTargetMetadata}
          onChange={(event) =>
            onUpdateSitemap((current) =>
              updateSubArea(current, selection, (subArea) => {
                if (subArea.target.type !== "entity") {
                  return subArea;
                }

                return {
                  ...subArea,
                  target: {
                    ...subArea.target,
                    default_view: event.target.value.trim().length > 0 ? event.target.value : null,
                  },
                };
              }),
            )
          }
        >
          <option value="">None</option>
          {selectedEntityViews.map((view) => (
            <option key={view.logical_name} value={view.logical_name}>
              {view.display_name} ({view.logical_name})
            </option>
          ))}
        </Select>
      </div>
    </>
  );
}
