import { Notice } from "@qryvanta/ui";

import type {
  AppPublishDiffResponse,
  EntityPublishDiffResponse,
} from "@/lib/api";

type PublishDiffPanelProps = {
  publishDiff: {
    unknownEntityLogicalNames: string[];
    unknownAppLogicalNames: string[];
    entityDiffs: EntityPublishDiffResponse[];
    appDiffs: AppPublishDiffResponse[];
  };
};

export function PublishDiffPanel({ publishDiff }: PublishDiffPanelProps) {
  return (
    <div className="rounded-md border border-zinc-200 bg-zinc-50 p-3">
      <p className="text-xs font-semibold uppercase tracking-[0.14em] text-zinc-600">
        Publish Diff Preview
      </p>
      <p className="mt-1 text-xs text-zinc-500">
        Field/form/view-level draft-to-published preview for selected entities and apps.
      </p>
      <div className="mt-3 space-y-3">
        {publishDiff.unknownEntityLogicalNames.length > 0 ||
        publishDiff.unknownAppLogicalNames.length > 0 ? (
          <Notice tone="warning">
            Unknown selections: entities [{publishDiff.unknownEntityLogicalNames.join(", ") || "none"}], apps [{publishDiff.unknownAppLogicalNames.join(", ") || "none"}]
          </Notice>
        ) : null}

        <div className="rounded-md border border-zinc-200 bg-white p-2">
          <p className="text-[11px] font-semibold uppercase tracking-wide text-zinc-600">
            Entity Diffs ({publishDiff.entityDiffs.length})
          </p>
          <div className="mt-2 space-y-2">
            {publishDiff.entityDiffs.map((entityDiff) => (
              <details
                key={`entity-diff-${entityDiff.entity_logical_name}`}
                className="rounded border border-zinc-200 p-2"
                open
              >
                <summary className="cursor-pointer text-xs font-semibold text-zinc-800">
                  {entityDiff.entity_logical_name} - {entityDiff.field_diff.length} field changes, {entityDiff.forms.length} forms, {entityDiff.views.length} views
                </summary>
                <div className="mt-2 grid gap-2 text-xs text-zinc-700 md:grid-cols-3">
                  <div>
                    <p className="font-semibold uppercase tracking-wide text-zinc-500">
                      Field Changes
                    </p>
                    <ul className="mt-1 space-y-1">
                      {entityDiff.field_diff.length > 0 ? (
                        entityDiff.field_diff.map((item) => (
                          <li
                            key={`field-diff-${entityDiff.entity_logical_name}-${item.field_logical_name}`}
                          >
                            {item.field_logical_name} [{item.change_type}] {item.published_field_type ?? "-"} {"->"} {item.draft_field_type ?? "-"}
                          </li>
                        ))
                      ) : (
                        <li>No field deltas</li>
                      )}
                    </ul>
                  </div>
                  <div>
                    <p className="font-semibold uppercase tracking-wide text-zinc-500">
                      Forms
                    </p>
                    <ul className="mt-1 space-y-1">
                      {entityDiff.forms.map((form) => (
                        <li key={`entity-form-diff-${entityDiff.entity_logical_name}-${form.logical_name}`}>
                          {form.logical_name} [{form.change_type}] {form.published_item_count ?? 0} {"->"} {form.draft_item_count ?? 0} fields
                          {form.published_is_default || form.draft_is_default ? (
                            <span>
                              {" "}[default {String(form.published_is_default ?? false)} {"->"} {String(form.draft_is_default ?? false)}]
                            </span>
                          ) : null}
                        </li>
                      ))}
                    </ul>
                  </div>
                  <div>
                    <p className="font-semibold uppercase tracking-wide text-zinc-500">
                      Views
                    </p>
                    <ul className="mt-1 space-y-1">
                      {entityDiff.views.map((view) => (
                        <li key={`entity-view-diff-${entityDiff.entity_logical_name}-${view.logical_name}`}>
                          {view.logical_name} [{view.change_type}] {view.published_item_count ?? 0} {"->"} {view.draft_item_count ?? 0} columns
                          {view.published_is_default || view.draft_is_default ? (
                            <span>
                              {" "}[default {String(view.published_is_default ?? false)} {"->"} {String(view.draft_is_default ?? false)}]
                            </span>
                          ) : null}
                        </li>
                      ))}
                    </ul>
                  </div>
                </div>
              </details>
            ))}
          </div>
        </div>

        <div className="rounded-md border border-zinc-200 bg-white p-2">
          <p className="text-[11px] font-semibold uppercase tracking-wide text-zinc-600">
            App Diffs ({publishDiff.appDiffs.length})
          </p>
          <div className="mt-2 space-y-2">
            {publishDiff.appDiffs.map((appDiff) => (
              <details
                key={`app-diff-${appDiff.app_logical_name}`}
                className="rounded border border-zinc-200 p-2"
                open
              >
                <summary className="cursor-pointer text-xs font-semibold text-zinc-800">
                  {appDiff.app_logical_name} - {appDiff.bindings.length} entity bindings
                </summary>
                <ul className="mt-2 space-y-1 text-xs text-zinc-700">
                  {appDiff.bindings.map((binding) => (
                    <li
                      key={`app-binding-diff-${appDiff.app_logical_name}-${binding.entity_logical_name}`}
                    >
                      {binding.entity_logical_name} / form: {binding.default_form_logical_name} / view: {binding.default_list_view_logical_name} / forms {binding.forms.length} / views {binding.views.length}
                    </li>
                  ))}
                </ul>
              </details>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
