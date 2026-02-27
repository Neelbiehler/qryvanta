import type { Metadata } from "next";
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

export const metadata: Metadata = {
  title: "Maker Center",
  description: "Configure entities, app bindings, and workflow automation.",
};

export default function MakerHomePage() {
  return (
    <div className="space-y-6">
      <Card className="border-emerald-200 bg-gradient-to-r from-emerald-50/60 to-white">
        <CardHeader>
          <CardTitle className="text-lg">Studio</CardTitle>
          <CardDescription>
            The unified low-code workspace. Design forms, configure views, and
            compose your app — all in one place.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Link href="/maker/studio/_default" className={cn(buttonVariants())}>
            Open Studio
          </Link>
        </CardContent>
      </Card>

      <div className="grid gap-4 lg:grid-cols-3">
        <Card>
          <CardHeader>
            <CardTitle>Entity Modeling</CardTitle>
            <CardDescription>
              Define metadata fields, publish versions, and validate runtime
              data.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Link
              href="/maker/entities"
              className={cn(buttonVariants({ variant: "outline" }))}
            >
              Open Entities
            </Link>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Studio</CardTitle>
            <CardDescription>
              Unified app, form, view, security, and publish composition workspace.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Link
              href="/maker/studio/_default"
              className={cn(buttonVariants({ variant: "outline" }))}
            >
              Open Unified Studio
            </Link>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Automation</CardTitle>
            <CardDescription>
              Configure internal triggers/actions and inspect execution history.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Link
              href="/maker/automation"
              className={cn(buttonVariants({ variant: "outline" }))}
            >
              Open Automation
            </Link>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
