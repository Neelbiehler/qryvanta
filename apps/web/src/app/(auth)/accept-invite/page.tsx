"use client";

import { useSearchParams } from "next/navigation";
import { useEffect, useState } from "react";

import { Button, Card, CardContent, CardDescription, CardHeader, CardTitle, Input, Label } from "@qryvanta/ui";
import { API_BASE_URL, type AcceptInviteRequest } from "@/lib/api";

type ErrorResponse = { message?: string };

export default function AcceptInvitePage() {
  const token = useSearchParams().get("token") ?? "";
  const [displayName, setDisplayName] = useState("");
  const [password, setPassword] = useState("");
  const [status, setStatus] = useState("");
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!token || typeof window === "undefined") {
      return;
    }

    const url = new URL(window.location.href);
    url.searchParams.delete("token");
    window.history.replaceState({}, "", url.toString());
  }, [token]);

  async function acceptInvite() {
    if (!token) {
      setStatus("Missing invite token.");
      return;
    }

    setLoading(true);
    setStatus("");
    try {
      const payload: AcceptInviteRequest = {
        token,
        password: password || null,
        display_name: displayName || null,
      };

      const response = await fetch(`${API_BASE_URL}/auth/invite/accept`, {
        method: "POST",
        credentials: "include",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      });

      if (!response.ok) {
        const errorPayload = (await response.json().catch(() => ({}))) as ErrorResponse;
        setStatus(errorPayload.message ?? "Invite acceptance failed.");
        return;
      }

      window.location.href = "/";
    } catch {
      setStatus("Invite request failed.");
    } finally {
      setLoading(false);
    }
  }

  return (
    <main className="grid min-h-screen place-items-center bg-app px-6 py-12">
      <Card className="w-full max-w-lg">
        <CardHeader>
          <CardTitle className="font-serif text-3xl">Accept Invitation</CardTitle>
          <CardDescription>
            Set a display name and password if this is your first time joining.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="display-name">Display name</Label>
            <Input
              id="display-name"
              value={displayName}
              onChange={(event) => setDisplayName(event.target.value)}
              placeholder="Alex Rivera"
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="password">Password (new users)</Label>
            <Input
              id="password"
              type="password"
              value={password}
              onChange={(event) => setPassword(event.target.value)}
              placeholder="Set a password if you do not already have one"
            />
          </div>
          <Button onClick={acceptInvite} disabled={loading} className="w-full">
            Accept invite
          </Button>
          {status ? <p className="text-sm text-zinc-600">{status}</p> : null}
        </CardContent>
      </Card>
    </main>
  );
}
