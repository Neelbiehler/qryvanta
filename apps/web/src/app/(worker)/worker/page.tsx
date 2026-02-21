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

export default function WorkerHomePage() {
  return (
    <div className="grid gap-4 lg:grid-cols-2">
      <Card>
        <CardHeader>
          <CardTitle>Assigned Apps</CardTitle>
          <CardDescription>
            Open business applications that are mapped to your runtime role.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Link href="/worker/apps" className={cn(buttonVariants())}>
            Open My Apps
          </Link>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Daily Workflow</CardTitle>
          <CardDescription>
            Enter app entities, create records, and complete operational tasks.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-zinc-600">
            Start from My Apps, then choose an entity workspace for your active
            job.
          </p>
        </CardContent>
      </Card>
    </div>
  );
}
