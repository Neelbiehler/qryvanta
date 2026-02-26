import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@qryvanta/ui";

import type { SelectionState } from "@/components/apps/sitemap-editor/types";
import type { AppSitemapResponse } from "@/lib/api";

type SitemapPreviewCardProps = {
  sitemap: AppSitemapResponse;
  onSelectNode: (selection: SelectionState) => void;
};

export function SitemapPreviewCard({ sitemap, onSelectNode }: SitemapPreviewCardProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-base">Preview</CardTitle>
        <CardDescription>
          Worker sidebar preview updates as you modify the sitemap tree.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-3">
        <div className="space-y-3 rounded-md border border-zinc-200 bg-zinc-50 p-3">
          {sitemap.areas.map((area) => (
            <div key={`preview-area-${area.logical_name}`} className="space-y-2">
              <button
                type="button"
                className="text-left text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500 hover:text-zinc-700"
                onClick={() => onSelectNode({ kind: "area", areaIndex: area.position })}
              >
                {area.display_name}
              </button>
              <div className="space-y-2 pl-2">
                {area.groups.map((group) => (
                  <details key={`preview-group-${group.logical_name}`} open>
                    <summary
                      className="cursor-pointer text-sm font-medium text-zinc-700"
                      onClick={() =>
                        onSelectNode({
                          kind: "group",
                          areaIndex: area.position,
                          groupIndex: group.position,
                        })
                      }
                    >
                      {group.display_name}
                    </summary>
                    <div className="mt-1 space-y-1 pl-3">
                      {group.sub_areas.map((subArea) => (
                        <button
                          key={`preview-sub-area-${subArea.logical_name}`}
                          type="button"
                          className="text-left text-sm text-zinc-600 hover:text-zinc-900"
                          onClick={() =>
                            onSelectNode({
                              kind: "sub_area",
                              areaIndex: area.position,
                              groupIndex: group.position,
                              subAreaIndex: subArea.position,
                            })
                          }
                        >
                          {subArea.display_name}
                        </button>
                      ))}
                    </div>
                  </details>
                ))}
              </div>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}
