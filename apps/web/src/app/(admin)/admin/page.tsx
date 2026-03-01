import Link from "next/link";

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  buttonVariants,
} from "@qryvanta/ui";

import { cn } from "@/lib/utils";

export default function AdminHomePage() {
  return (
    <div className="grid gap-4 lg:grid-cols-3">
      <Card>
        <CardHeader>
          <CardTitle>Role Governance</CardTitle>
          <CardDescription>
            Maintain role definitions, assignments, and registration mode.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Link href="/admin/roles" className={cn(buttonVariants())}>
            Open Roles
          </Link>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Audit Control</CardTitle>
          <CardDescription>
            Inspect and export security activity for tenant compliance.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Link href="/admin/audit" className={cn(buttonVariants())}>
            Open Audit Log
          </Link>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Account Security</CardTitle>
          <CardDescription>
            Manage password, MFA setup, invites, and verification status.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Link
            href="/admin/account"
            className={cn(buttonVariants({ variant: "outline" }))}
          >
            Open Security Settings
          </Link>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Search Sync Health</CardTitle>
          <CardDescription>
            Monitor Qrywell indexing queue, retries, and recent failed jobs.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Link
            href="/admin/search-sync"
            className={cn(buttonVariants({ variant: "outline" }))}
          >
            Open Sync Health
          </Link>
        </CardContent>
      </Card>
    </div>
  );
}
