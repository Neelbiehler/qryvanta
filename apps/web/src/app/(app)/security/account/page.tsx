"use client";

import { useState } from "react";

import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Input,
  Label,
} from "@qryvanta/ui";
import { apiFetch, type GenericMessageResponse, type InviteRequest, type UserIdentityResponse } from "@/lib/api";

type ErrorResponse = { message?: string };

type TotpEnrollmentResponse = {
  secret_base32: string;
  otpauth_uri: string;
  recovery_codes: string[];
};

type RecoveryCodesResponse = {
  recovery_codes: string[];
};

async function readErrorMessage(response: Response, fallback: string): Promise<string> {
  try {
    const payload = (await response.json()) as ErrorResponse;
    return payload.message ?? fallback;
  } catch {
    return fallback;
  }
}

export default function SecurityAccountPage() {
  const [status, setStatus] = useState("");
  const [loading, setLoading] = useState(false);
  const [me, setMe] = useState<UserIdentityResponse | null>(null);

  const [currentPassword, setCurrentPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");

  const [enrollment, setEnrollment] = useState<TotpEnrollmentResponse | null>(null);
  const [confirmCode, setConfirmCode] = useState("");
  const [disablePassword, setDisablePassword] = useState("");
  const [regeneratePassword, setRegeneratePassword] = useState("");
  const [newRecoveryCodes, setNewRecoveryCodes] = useState<string[]>([]);

  const [inviteEmail, setInviteEmail] = useState("");
  const [inviteTenantName, setInviteTenantName] = useState("");

  async function loadMe() {
    setLoading(true);
    setStatus("");
    try {
      const response = await apiFetch("/auth/me");
      if (!response.ok) {
        setStatus(await readErrorMessage(response, "Failed to load account."));
        return;
      }
      setMe((await response.json()) as UserIdentityResponse);
    } catch {
      setStatus("Failed to load account.");
    } finally {
      setLoading(false);
    }
  }

  async function resendVerification() {
    setLoading(true);
    setStatus("");
    try {
      const response = await apiFetch("/auth/resend-verification", { method: "POST" });
      if (!response.ok) {
        setStatus(await readErrorMessage(response, "Failed to send verification email."));
        return;
      }
      const body = (await response.json()) as GenericMessageResponse;
      setStatus(body.message);
    } catch {
      setStatus("Failed to send verification email.");
    } finally {
      setLoading(false);
    }
  }

  async function changePassword() {
    if (!currentPassword || !newPassword) {
      setStatus("Current and new password are required.");
      return;
    }

    setLoading(true);
    setStatus("");
    try {
      const response = await apiFetch("/api/profile/password", {
        method: "PUT",
        body: JSON.stringify({ current_password: currentPassword, new_password: newPassword }),
      });
      if (!response.ok) {
        setStatus(await readErrorMessage(response, "Password update failed."));
        return;
      }
      setStatus("Password updated successfully.");
      setCurrentPassword("");
      setNewPassword("");
    } catch {
      setStatus("Password update failed.");
    } finally {
      setLoading(false);
    }
  }

  async function startMfaEnrollment() {
    setLoading(true);
    setStatus("");
    try {
      const response = await apiFetch("/auth/mfa/totp/enroll", { method: "POST" });
      if (!response.ok) {
        setStatus(await readErrorMessage(response, "MFA enrollment start failed."));
        return;
      }
      setEnrollment((await response.json()) as TotpEnrollmentResponse);
      setStatus("Scan the secret and confirm with a TOTP code.");
    } catch {
      setStatus("MFA enrollment start failed.");
    } finally {
      setLoading(false);
    }
  }

  async function confirmMfaEnrollment() {
    if (!confirmCode) {
      setStatus("Enter the TOTP code to confirm enrollment.");
      return;
    }

    setLoading(true);
    setStatus("");
    try {
      const response = await apiFetch("/auth/mfa/totp/confirm", {
        method: "POST",
        body: JSON.stringify({ code: confirmCode }),
      });
      if (!response.ok) {
        setStatus(await readErrorMessage(response, "MFA confirmation failed."));
        return;
      }
      setStatus("MFA enabled successfully.");
      setConfirmCode("");
    } catch {
      setStatus("MFA confirmation failed.");
    } finally {
      setLoading(false);
    }
  }

  async function disableMfa() {
    if (!disablePassword) {
      setStatus("Password is required to disable MFA.");
      return;
    }

    setLoading(true);
    setStatus("");
    try {
      const response = await apiFetch("/auth/mfa/totp", {
        method: "DELETE",
        body: JSON.stringify({ password: disablePassword }),
      });
      if (!response.ok) {
        setStatus(await readErrorMessage(response, "Failed to disable MFA."));
        return;
      }
      setStatus("MFA disabled.");
      setDisablePassword("");
      setEnrollment(null);
    } catch {
      setStatus("Failed to disable MFA.");
    } finally {
      setLoading(false);
    }
  }

  async function regenerateRecoveryCodes() {
    if (!regeneratePassword) {
      setStatus("Password is required to regenerate recovery codes.");
      return;
    }

    setLoading(true);
    setStatus("");
    try {
      const response = await apiFetch("/auth/mfa/recovery-codes/regenerate", {
        method: "POST",
        body: JSON.stringify({ password: regeneratePassword }),
      });
      if (!response.ok) {
        setStatus(await readErrorMessage(response, "Failed to regenerate recovery codes."));
        return;
      }
      const body = (await response.json()) as RecoveryCodesResponse;
      setNewRecoveryCodes(body.recovery_codes);
      setStatus("Recovery codes regenerated. Save them now.");
      setRegeneratePassword("");
    } catch {
      setStatus("Failed to regenerate recovery codes.");
    } finally {
      setLoading(false);
    }
  }

  async function sendInvite() {
    if (!inviteEmail) {
      setStatus("Invite email is required.");
      return;
    }

    setLoading(true);
    setStatus("");
    try {
      const payload: InviteRequest = {
        email: inviteEmail,
        tenant_name: inviteTenantName || null,
      };
      const response = await apiFetch("/auth/invite", {
        method: "POST",
        body: JSON.stringify(payload),
      });
      if (!response.ok) {
        setStatus(await readErrorMessage(response, "Failed to send invite."));
        return;
      }
      const body = (await response.json()) as GenericMessageResponse;
      setStatus(body.message);
      setInviteEmail("");
      setInviteTenantName("");
    } catch {
      setStatus("Failed to send invite.");
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="font-serif text-3xl text-zinc-900">Security Settings</h1>
        <p className="mt-2 text-sm text-zinc-600">
          Manage your account verification, password, MFA, and tenant invites.
        </p>
      </div>

      <div className="grid gap-6 lg:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>Account Snapshot</CardTitle>
            <CardDescription>Check the authenticated identity currently loaded.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <Button onClick={loadMe} disabled={loading} variant="outline" className="w-full">
              Refresh account details
            </Button>
            {me ? (
              <div className="rounded-md border border-zinc-200 bg-zinc-50 p-3 text-sm text-zinc-700">
                <p><strong>Display:</strong> {me.display_name}</p>
                <p><strong>Email:</strong> {me.email ?? "n/a"}</p>
                <p><strong>Subject:</strong> {me.subject}</p>
                <p><strong>Tenant:</strong> {me.tenant_id}</p>
              </div>
            ) : null}
            <Button onClick={resendVerification} disabled={loading} className="w-full">
              Resend verification email
            </Button>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Change Password</CardTitle>
            <CardDescription>Rotate your password without ending your current session.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="space-y-2">
              <Label htmlFor="current-password">Current password</Label>
              <Input id="current-password" type="password" value={currentPassword} onChange={(event) => setCurrentPassword(event.target.value)} />
            </div>
            <div className="space-y-2">
              <Label htmlFor="new-password">New password</Label>
              <Input id="new-password" type="password" value={newPassword} onChange={(event) => setNewPassword(event.target.value)} />
            </div>
            <Button onClick={changePassword} disabled={loading} className="w-full">
              Update password
            </Button>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>MFA (TOTP)</CardTitle>
            <CardDescription>Enable authenticator-based MFA and manage recovery codes.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <Button onClick={startMfaEnrollment} disabled={loading} variant="outline" className="w-full">
              Start TOTP enrollment
            </Button>
            {enrollment ? (
              <div className="space-y-2 rounded-md border border-emerald-100 bg-emerald-50/70 p-3 text-sm">
                <p><strong>Secret:</strong> {enrollment.secret_base32}</p>
                <p className="break-all"><strong>URI:</strong> {enrollment.otpauth_uri}</p>
                <p><strong>Recovery Codes:</strong> {enrollment.recovery_codes.join(", ")}</p>
              </div>
            ) : null}
            <div className="space-y-2">
              <Label htmlFor="confirm-mfa">Confirm code</Label>
              <Input id="confirm-mfa" value={confirmCode} onChange={(event) => setConfirmCode(event.target.value)} placeholder="123456" />
            </div>
            <Button onClick={confirmMfaEnrollment} disabled={loading} className="w-full">
              Confirm MFA enrollment
            </Button>
            <div className="space-y-2 pt-2">
              <Label htmlFor="disable-password">Disable MFA password</Label>
              <Input id="disable-password" type="password" value={disablePassword} onChange={(event) => setDisablePassword(event.target.value)} />
            </div>
            <Button onClick={disableMfa} disabled={loading} variant="outline" className="w-full">
              Disable MFA
            </Button>
            <div className="space-y-2 pt-2">
              <Label htmlFor="regen-password">Regenerate codes password</Label>
              <Input id="regen-password" type="password" value={regeneratePassword} onChange={(event) => setRegeneratePassword(event.target.value)} />
            </div>
            <Button onClick={regenerateRecoveryCodes} disabled={loading} variant="outline" className="w-full">
              Regenerate recovery codes
            </Button>
            {newRecoveryCodes.length > 0 ? (
              <p className="rounded-md border border-amber-200 bg-amber-50 p-3 text-sm text-amber-900">
                New codes: {newRecoveryCodes.join(", ")}
              </p>
            ) : null}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Invite Teammate</CardTitle>
            <CardDescription>Send a workspace invite link to a teammate.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="space-y-2">
              <Label htmlFor="invite-email">Email</Label>
              <Input id="invite-email" type="email" value={inviteEmail} onChange={(event) => setInviteEmail(event.target.value)} placeholder="teammate@company.com" />
            </div>
            <div className="space-y-2">
              <Label htmlFor="invite-tenant">Workspace name (optional)</Label>
              <Input id="invite-tenant" value={inviteTenantName} onChange={(event) => setInviteTenantName(event.target.value)} placeholder="Acme Operations" />
            </div>
            <Button onClick={sendInvite} disabled={loading} className="w-full">
              Send invite
            </Button>
          </CardContent>
        </Card>
      </div>

      {status ? (
        <p aria-live="polite" className="rounded-md border border-zinc-200 bg-zinc-50 p-3 text-sm text-zinc-700">
          {status}
        </p>
      ) : null}
    </div>
  );
}
