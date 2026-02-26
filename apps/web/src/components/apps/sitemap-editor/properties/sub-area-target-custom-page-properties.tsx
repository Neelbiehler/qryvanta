import { Input, Label } from "@qryvanta/ui";

import { updateSubArea } from "@/components/apps/sitemap-editor/properties/sub-area-properties-utils";
import type {
  SubAreaSelection,
  UpdateSitemap,
} from "@/components/apps/sitemap-editor/properties/types";
import type { AppSitemapTargetDto } from "@/lib/api";

type SubAreaCustomPageTargetPropertiesProps = {
  selection: SubAreaSelection;
  target: Extract<AppSitemapTargetDto, { type: "custom_page" }>;
  onUpdateSitemap: UpdateSitemap;
};

export function SubAreaCustomPageTargetProperties({
  selection,
  target,
  onUpdateSitemap,
}: SubAreaCustomPageTargetPropertiesProps) {
  return (
    <div className="space-y-2">
      <Label htmlFor="sub_area_target_url">Custom Page URL</Label>
      <Input
        id="sub_area_target_url"
        value={target.url}
        onChange={(event) =>
          onUpdateSitemap((current) =>
            updateSubArea(current, selection, (subArea) => {
              if (subArea.target.type !== "custom_page") {
                return subArea;
              }

              return {
                ...subArea,
                target: {
                  ...subArea.target,
                  url: event.target.value,
                },
              };
            }),
          )
        }
      />
    </div>
  );
}
