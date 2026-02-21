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

export default function MakerHomePage() {
  return (
    <div className="grid gap-4 lg:grid-cols-2">
      <Card>
        <CardHeader>
          <CardTitle>Entity Modeling</CardTitle>
          <CardDescription>
            Define metadata fields, publish versions, and validate runtime data.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Link href="/maker/entities" className={cn(buttonVariants())}>
            Open Entities
          </Link>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>App Studio</CardTitle>
          <CardDescription>
            Bind entities into apps and apply role-scoped runtime access.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Link
            href="/maker/apps"
            className={cn(buttonVariants({ variant: "outline" }))}
          >
            Open App Studio
          </Link>
        </CardContent>
      </Card>
    </div>
  );
}
