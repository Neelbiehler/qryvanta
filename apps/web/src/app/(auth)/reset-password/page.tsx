"use client";

import { useSearchParams } from "next/navigation";
import { useState } from "react";

import { Button, Card, CardContent, CardHeader, CardTitle, Input, Label } from "@qryvanta/ui";
import { API_BASE_URL, type GenericMessageResponse } from "@/lib/api";

type ErrorResponse = { message?: string };

export default function ResetPasswordPage() {
  const searchParams = useSearchParams();
  const token = searchParams.get("token") ?? "";
  const [password, setPassword] = useState("");
  const [status, setStatus] = useState("");
  const [loading, setLoading] = useState(false);

  async function submit() {
    if (!token || !password) {
      setStatus("Token and new password are required.");
      return;
    }

    setLoading(true);
    setStatus("");
    try {
      const response = await fetch(`${API_BASE_URL}/auth/reset-password`, {
        method: "POST",
        credentials: "include",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ token, new_password: password }),
      });

      if (!response.ok) {
        const payload = (await response.json().catch(() => ({}))) as ErrorResponse;
        setStatus(payload.message ?? "Reset failed.");
        return;
      }

      const body = (await response.json()) as GenericMessageResponse;
      setStatus(body.message);
    } catch {
      setStatus("Reset request failed.");
    } finally {
      setLoading(false);
    }
  }

  return (
    <main className="grid min-h-screen place-items-center bg-app px-6 py-12">
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle className="font-serif text-3xl">Reset Password</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="new-password">New Password</Label>
            <Input
              id="new-password"
              type="password"
              value={password}
              onChange={(event) => setPassword(event.target.value)}
              placeholder="Enter your new password"
            />
          </div>
          <Button onClick={submit} disabled={loading} className="w-full">
            Update password
          </Button>
          {status ? <p className="text-sm text-zinc-600">{status}</p> : null}
        </CardContent>
      </Card>
    </main>
  );
}
