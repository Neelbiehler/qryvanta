import Link from "next/link";

import { CreateEntityForm } from "@/components/entities/create-entity-form";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  StatusBadge,
  buttonVariants,
} from "@qryvanta/ui";
import { cn } from "@/lib/utils";

export default function MakerNewEntityPage() {
  return (
    <div className="space-y-4">
      <Card>
        <CardHeader className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
          <div className="space-y-2">
            <p className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500">
              Maker Center
            </p>
            <CardTitle className="font-serif text-3xl">New Entity Definition</CardTitle>
            <CardDescription>
              Start with naming conventions and publish-ready metadata structure.
            </CardDescription>
          </div>
          <div className="flex items-center gap-2">
            <StatusBadge tone="neutral">Model-driven setup</StatusBadge>
            <Link
              href="/maker/entities"
              className={cn(buttonVariants({ variant: "outline" }))}
            >
              Back to library
            </Link>
          </div>
        </CardHeader>
      </Card>

      <Card className="mx-auto w-full max-w-3xl border-emerald-200 bg-white">
        <CardContent className="pt-6">
          <CreateEntityForm />
        </CardContent>
      </Card>
    </div>
  );
}
