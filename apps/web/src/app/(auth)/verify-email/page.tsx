"use client";

import { useSearchParams } from "next/navigation";
import { useState } from "react";

import { Button, Card, CardContent, CardHeader, CardTitle } from "@qryvanta/ui";
import { API_BASE_URL, type GenericMessageResponse } from "@/lib/api";

type ErrorResponse = { message?: string };

export default function VerifyEmailPage() {
  const token = useSearchParams().get("token") ?? "";
  const [status, setStatus] = useState("");
  const [loading, setLoading] = useState(false);

  async function verify() {
    if (!token) {
      setStatus("Missing verification token.");
      return;
    }

    setLoading(true);
    setStatus("");
    try {
      const response = await fetch(`${API_BASE_URL}/auth/verify-email`, {
        method: "POST",
        credentials: "include",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ token }),
      });

      if (!response.ok) {
        const payload = (await response.json().catch(() => ({}))) as ErrorResponse;
        setStatus(payload.message ?? "Verification failed.");
        return;
      }

      const body = (await response.json()) as GenericMessageResponse;
      setStatus(body.message);
    } catch {
      setStatus("Verification request failed.");
    } finally {
      setLoading(false);
    }
  }

  return (
    <main className="grid min-h-screen place-items-center bg-app px-6 py-12">
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle className="font-serif text-3xl">Verify Email</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <Button onClick={verify} disabled={loading} className="w-full">
            Verify email
          </Button>
          {status ? <p className="text-sm text-zinc-600">{status}</p> : null}
        </CardContent>
      </Card>
    </main>
  );
}
