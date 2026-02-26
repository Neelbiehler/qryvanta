import { Input, Label } from "@qryvanta/ui";

import { updateSubArea } from "@/components/apps/sitemap-editor/properties/sub-area-properties-utils";
import type {
  SubAreaSelection,
  UpdateSitemap,
} from "@/components/apps/sitemap-editor/properties/types";
import type { AppSitemapTargetDto } from "@/lib/api";

type SubAreaDashboardTargetPropertiesProps = {
  selection: SubAreaSelection;
  target: Extract<AppSitemapTargetDto, { type: "dashboard" }>;
  onUpdateSitemap: UpdateSitemap;
};

export function SubAreaDashboardTargetProperties({
  selection,
  target,
  onUpdateSitemap,
}: SubAreaDashboardTargetPropertiesProps) {
  return (
    <div className="space-y-2">
      <Label htmlFor="sub_area_target_dashboard">Dashboard Logical Name</Label>
      <Input
        id="sub_area_target_dashboard"
        value={target.dashboard_logical_name}
        onChange={(event) =>
          onUpdateSitemap((current) =>
            updateSubArea(current, selection, (subArea) => {
              if (subArea.target.type !== "dashboard") {
                return subArea;
              }

              return {
                ...subArea,
                target: {
                  ...subArea.target,
                  dashboard_logical_name: event.target.value,
                },
              };
            }),
          )
        }
      />
    </div>
  );
}
