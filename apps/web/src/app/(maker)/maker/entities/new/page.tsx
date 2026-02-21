import Link from "next/link";

import { CreateEntityForm } from "@/components/entities/create-entity-form";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  buttonVariants,
} from "@qryvanta/ui";
import { cn } from "@/lib/utils";

export default function MakerNewEntityPage() {
  return (
    <Card className="mx-auto w-full max-w-2xl">
      <CardHeader className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
        <div>
          <p className="text-xs uppercase tracking-[0.18em] text-zinc-500">
            Maker Center
          </p>
          <CardTitle className="font-serif text-3xl">Create Entity</CardTitle>
        </div>
        <Link
          href="/maker/entities"
          className={cn(buttonVariants({ variant: "outline" }))}
        >
          Back to entities
        </Link>
      </CardHeader>
      <CardContent>
        <CreateEntityForm />
      </CardContent>
    </Card>
  );
}
