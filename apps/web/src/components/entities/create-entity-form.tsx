"use client";

import { type FormEvent, useState } from "react";
import { useRouter } from "next/navigation";

import { Button, Input, Label } from "@qryvanta/ui";
import { apiFetch } from "@/lib/api";

export function CreateEntityForm() {
  const router = useRouter();
  const [logicalName, setLogicalName] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setIsSubmitting(true);
    setErrorMessage(null);

    try {
      const response = await apiFetch("/api/entities", {
        method: "POST",
        body: JSON.stringify({
          logical_name: logicalName,
          display_name: displayName,
        }),
      });

      if (!response.ok) {
        const payload = (await response.json()) as { message?: string };
        setErrorMessage(payload.message ?? "Unable to create entity.");
        return;
      }

      router.push("/maker/entities");
      router.refresh();
    } catch {
      setErrorMessage("Unable to create entity.");
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <form className="space-y-4" onSubmit={handleSubmit}>
      <div className="space-y-2">
        <Label htmlFor="logical_name">Logical Name</Label>
        <Input
          id="logical_name"
          name="logical_name"
          placeholder="contact"
          value={logicalName}
          onChange={(event) => setLogicalName(event.target.value)}
          required
        />
      </div>

      <div className="space-y-2">
        <Label htmlFor="display_name">Display Name</Label>
        <Input
          id="display_name"
          name="display_name"
          placeholder="Contact"
          value={displayName}
          onChange={(event) => setDisplayName(event.target.value)}
          required
        />
      </div>

      {errorMessage ? (
        <p className="rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700">
          {errorMessage}
        </p>
      ) : null}

      <Button disabled={isSubmitting} type="submit">
        {isSubmitting ? "Creating..." : "Create Entity"}
      </Button>
    </form>
  );
}
