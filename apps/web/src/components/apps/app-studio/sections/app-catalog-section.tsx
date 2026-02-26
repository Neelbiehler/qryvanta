import type { FormEvent } from "react";
import Link from "next/link";

import {
  Button,
  Input,
  Label,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
  buttonVariants,
} from "@qryvanta/ui";

import type { AppResponse } from "@/lib/api";
import { cn } from "@/lib/utils";
import type { NewAppDraft } from "@/components/apps/app-studio/sections/types";

type AppCatalogSectionProps = {
  apps: AppResponse[];
  isCreatingApp: boolean;
  newAppDraft: NewAppDraft;
  onCreateApp: (event: FormEvent<HTMLFormElement>) => void;
  onUpdateDraft: (next: NewAppDraft) => void;
};

export function AppCatalogSection({
  apps,
  isCreatingApp,
  newAppDraft,
  onCreateApp,
  onUpdateDraft,
}: AppCatalogSectionProps) {
  return (
    <div className="space-y-3 rounded-md border border-zinc-200 bg-white p-4">
      <div>
        <p className="text-sm font-semibold text-zinc-900">App Catalog</p>
        <p className="text-xs text-zinc-600">
          Define application shells before sitemap and role matrix configuration.
        </p>
      </div>

      <form className="grid gap-3 md:grid-cols-3" onSubmit={onCreateApp}>
        <div className="space-y-2">
          <Label htmlFor="new_app_logical_name">App Logical Name</Label>
          <Input
            id="new_app_logical_name"
            value={newAppDraft.logicalName}
            onChange={(event) =>
              onUpdateDraft({ ...newAppDraft, logicalName: event.target.value })
            }
            placeholder="sales"
            required
          />
        </div>
        <div className="space-y-2">
          <Label htmlFor="new_app_display_name">App Display Name</Label>
          <Input
            id="new_app_display_name"
            value={newAppDraft.displayName}
            onChange={(event) =>
              onUpdateDraft({ ...newAppDraft, displayName: event.target.value })
            }
            placeholder="Sales App"
            required
          />
        </div>
        <div className="space-y-2">
          <Label htmlFor="new_app_description">Description</Label>
          <Input
            id="new_app_description"
            value={newAppDraft.description}
            onChange={(event) =>
              onUpdateDraft({ ...newAppDraft, description: event.target.value })
            }
            placeholder="Lead and account workflows"
          />
        </div>
        <div className="md:col-span-3">
          <Button disabled={isCreatingApp} type="submit">
            {isCreatingApp ? "Creating..." : "Create App"}
          </Button>
        </div>
      </form>

      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>App</TableHead>
            <TableHead>Description</TableHead>
            <TableHead>Logical Name</TableHead>
            <TableHead className="text-right">Actions</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {apps.length > 0 ? (
            apps.map((app) => (
              <TableRow key={app.logical_name}>
                <TableCell>{app.display_name}</TableCell>
                <TableCell>{app.description ?? "-"}</TableCell>
                <TableCell className="font-mono text-xs">{app.logical_name}</TableCell>
                <TableCell className="text-right">
                  <Link
                    href={`/maker/apps/${encodeURIComponent(app.logical_name)}/sitemap`}
                    className={cn(buttonVariants({ size: "sm", variant: "outline" }))}
                  >
                    Open Sitemap
                  </Link>
                </TableCell>
              </TableRow>
            ))
          ) : (
            <TableRow>
              <TableCell colSpan={4} className="text-zinc-500">
                No apps yet.
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>
    </div>
  );
}
