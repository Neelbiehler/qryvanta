import { type FormEvent, useMemo } from "react";

import {
  Button,
  Input,
  Label,
  Select,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@qryvanta/ui";

import {
  type AppEntityBindingResponse,
  type AppResponse,
  type EntityResponse,
  type FieldResponse,
} from "@/lib/api";
import { FieldLayoutDesigner } from "@/components/apps/app-studio/sections/field-layout-designer";
import type {
  AppEntityViewMode,
  BindingDraft,
} from "@/components/apps/app-studio/sections/types";

type NavigationBindingSectionProps = {
  apps: AppResponse[];
  bindings: AppEntityBindingResponse[];
  entities: EntityResponse[];
  selectedEntityFields: FieldResponse[];
  isLoadingEntityFields: boolean;
  isBindingEntity: boolean;
  isReorderingBinding: boolean;
  isLoadingAppData: boolean;
  onBindEntity: (event: FormEvent<HTMLFormElement>) => void;
  onReorderBinding: (entityLogicalName: string, direction: "up" | "down") => void;
  onChangeSelectedApp: (appLogicalName: string) => void;
  onUpdateBindingDraft: (next: BindingDraft) => void;
  selectedApp: string;
  selectedAppDisplayName: string;
  bindingDraft: BindingDraft;
};

export function NavigationBindingSection({
  apps,
  bindings,
  entities,
  selectedEntityFields,
  isLoadingEntityFields,
  isBindingEntity,
  isReorderingBinding,
  isLoadingAppData,
  onBindEntity,
  onReorderBinding,
  onChangeSelectedApp,
  onUpdateBindingDraft,
  selectedApp,
  selectedAppDisplayName,
  bindingDraft,
}: NavigationBindingSectionProps) {
  const orderedBindings = useMemo(
    () =>
      [...bindings].sort((left, right) => {
        if (left.navigation_order !== right.navigation_order) {
          return left.navigation_order - right.navigation_order;
        }

        return left.entity_logical_name.localeCompare(right.entity_logical_name);
      }),
    [bindings],
  );

  return (
    <div className="space-y-3 rounded-md border border-zinc-200 bg-white p-4">
      <div>
        <p className="text-sm font-semibold text-zinc-900">Sitemap Navigation</p>
        <p className="text-xs text-zinc-600">
          Bind entities to the app sitemap and configure list/form presentation defaults.
        </p>
      </div>

      <div className="space-y-2">
        <Label htmlFor="studio_app_selector">Active App</Label>
        <Select
          id="studio_app_selector"
          value={selectedApp}
          onChange={(event) => onChangeSelectedApp(event.target.value)}
        >
          {apps.map((app) => (
            <option key={app.logical_name} value={app.logical_name}>
              {app.display_name} ({app.logical_name})
            </option>
          ))}
        </Select>
      </div>

      <form className="grid gap-3 md:grid-cols-2" onSubmit={onBindEntity}>
        <div className="space-y-2">
          <Label htmlFor="bind_entity_name">Entity</Label>
          <Select
            id="bind_entity_name"
            value={bindingDraft.entityToBind}
            onChange={(event) =>
              onUpdateBindingDraft({ ...bindingDraft, entityToBind: event.target.value })
            }
          >
            {entities.map((entity) => (
              <option key={entity.logical_name} value={entity.logical_name}>
                {entity.display_name} ({entity.logical_name})
              </option>
            ))}
          </Select>
          <p className="text-[11px] text-zinc-500">
            {isLoadingEntityFields
              ? "Loading entity fields..."
              : `${selectedEntityFields.length} field(s) available for form/view design.`}
          </p>
        </div>

        <div className="space-y-2">
          <Label htmlFor="bind_navigation_label">Navigation Label</Label>
          <Input
            id="bind_navigation_label"
            value={bindingDraft.navigationLabel}
            onChange={(event) =>
              onUpdateBindingDraft({ ...bindingDraft, navigationLabel: event.target.value })
            }
            placeholder="Accounts"
          />
        </div>

        <div className="space-y-2">
          <Label htmlFor="bind_navigation_order">Navigation Order</Label>
          <Input
            id="bind_navigation_order"
            value={String(bindingDraft.navigationOrder)}
            onChange={(event) =>
              onUpdateBindingDraft({
                ...bindingDraft,
                navigationOrder: Number.parseInt(event.target.value || "0", 10),
              })
            }
            type="number"
            min={0}
          />
        </div>

        <div className="md:col-span-2">
          <FieldLayoutDesigner
            selectedEntityFields={selectedEntityFields}
            forms={bindingDraft.forms}
            listViews={bindingDraft.listViews}
            defaultFormLogicalName={bindingDraft.defaultFormLogicalName}
            defaultListViewLogicalName={bindingDraft.defaultListViewLogicalName}
            onChangeForms={(forms) => onUpdateBindingDraft({ ...bindingDraft, forms })}
            onChangeListViews={(listViews) =>
              onUpdateBindingDraft({ ...bindingDraft, listViews })
            }
            onChangeDefaultFormLogicalName={(defaultFormLogicalName) =>
              onUpdateBindingDraft({ ...bindingDraft, defaultFormLogicalName })
            }
            onChangeDefaultListViewLogicalName={(defaultListViewLogicalName) =>
              onUpdateBindingDraft({ ...bindingDraft, defaultListViewLogicalName })
            }
          />
        </div>

        <div className="space-y-2">
          <Label htmlFor="bind_default_view_mode">Default View Mode</Label>
          <Select
            id="bind_default_view_mode"
            value={bindingDraft.defaultViewMode}
            onChange={(event) =>
              onUpdateBindingDraft({
                ...bindingDraft,
                defaultViewMode: event.target.value as AppEntityViewMode,
              })
            }
          >
            <option value="grid">Grid</option>
            <option value="json">JSON</option>
          </Select>
        </div>

        <div className="md:col-span-2">
          <Button disabled={isBindingEntity || isLoadingAppData} type="submit" variant="outline">
            {isBindingEntity ? "Saving..." : `Bind Entity to ${selectedAppDisplayName}`}
          </Button>
        </div>
      </form>

      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Bound Entity</TableHead>
            <TableHead>Label</TableHead>
            <TableHead>Order</TableHead>
            <TableHead>Default View</TableHead>
            <TableHead>Presentation</TableHead>
            <TableHead className="text-right">Move</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {orderedBindings.length > 0 ? (
            orderedBindings.map((binding, index) => (
              <TableRow key={`${binding.app_logical_name}.${binding.entity_logical_name}`}>
                <TableCell className="font-mono text-xs">{binding.entity_logical_name}</TableCell>
                <TableCell>{binding.navigation_label ?? binding.entity_logical_name}</TableCell>
                <TableCell>{binding.navigation_order}</TableCell>
                <TableCell className="uppercase">{binding.default_view_mode}</TableCell>
                <TableCell className="text-xs text-zinc-600">
                  {resolveBindingFormCount(binding)} form(s) / {resolveBindingViewCount(binding)} view(s)
                </TableCell>
                <TableCell className="text-right">
                  <div className="inline-flex items-center gap-1">
                    <Button
                      type="button"
                      size="sm"
                      variant="outline"
                      disabled={isReorderingBinding || isLoadingAppData || index === 0}
                      onClick={() => onReorderBinding(binding.entity_logical_name, "up")}
                    >
                      Up
                    </Button>
                    <Button
                      type="button"
                      size="sm"
                      variant="outline"
                      disabled={
                        isReorderingBinding ||
                        isLoadingAppData ||
                        index === orderedBindings.length - 1
                      }
                      onClick={() => onReorderBinding(binding.entity_logical_name, "down")}
                    >
                      Down
                    </Button>
                  </div>
                </TableCell>
              </TableRow>
            ))
          ) : (
            <TableRow>
              <TableCell colSpan={6} className="text-zinc-500">
                No entity bindings for this app.
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>
    </div>
  );
}

function resolveBindingFormCount(binding: AppEntityBindingResponse): number {
  if (binding.forms.length > 0) {
    return binding.forms.length;
  }

  return binding.form_field_logical_names.length > 0 ? 1 : 0;
}

function resolveBindingViewCount(binding: AppEntityBindingResponse): number {
  if (binding.list_views.length > 0) {
    return binding.list_views.length;
  }

  return binding.list_field_logical_names.length > 0 ? 1 : 0;
}
