"use client";

import { useState } from "react";

import {
  apiFetch,
  type GenericMessageResponse,
  type InviteRequest,
  type UserIdentityResponse,
} from "@/lib/api";

type ErrorResponse = { message?: string };

export type TotpEnrollmentResponse = {
  secret_base32: string;
  otpauth_uri: string;
  recovery_codes: string[];
};

type RecoveryCodesResponse = {
  recovery_codes: string[];
};

type AccountActionId =
  | "load-me"
  | "resend-verification"
  | "change-password"
  | "start-mfa"
  | "confirm-mfa"
  | "disable-mfa"
  | "regenerate-codes"
  | "send-invite";

async function readErrorMessage(
  response: Response,
  fallback: string,
): Promise<string> {
  try {
    const payload = (await response.json()) as ErrorResponse;
    return payload.message ?? fallback;
  } catch {
    return fallback;
  }
}

export function useSecurityAccount() {
  const [status, setStatus] = useState("");
  const [activeAction, setActiveAction] = useState<AccountActionId | null>(
    null,
  );
  const [me, setMe] = useState<UserIdentityResponse | null>(null);

  const [currentPassword, setCurrentPassword] = useState("");
  const [newPassword, setNewPassword] = useState("");

  const [enrollment, setEnrollment] = useState<TotpEnrollmentResponse | null>(
    null,
  );
  const [confirmCode, setConfirmCode] = useState("");
  const [disablePassword, setDisablePassword] = useState("");
  const [regeneratePassword, setRegeneratePassword] = useState("");
  const [newRecoveryCodes, setNewRecoveryCodes] = useState<string[]>([]);

  const [inviteEmail, setInviteEmail] = useState("");
  const [inviteTenantName, setInviteTenantName] = useState("");

  async function withAction(action: AccountActionId, run: () => Promise<void>) {
    setActiveAction(action);
    setStatus("");
    try {
      await run();
    } finally {
      setActiveAction(null);
    }
  }

  async function loadMe() {
    await withAction("load-me", async () => {
      const response = await apiFetch("/auth/me");
      if (!response.ok) {
        setStatus(await readErrorMessage(response, "Failed to load account."));
        return;
      }
      setMe((await response.json()) as UserIdentityResponse);
    }).catch(() => {
      setStatus("Failed to load account.");
    });
  }

  async function resendVerification() {
    await withAction("resend-verification", async () => {
      const response = await apiFetch("/auth/resend-verification", {
        method: "POST",
      });
      if (!response.ok) {
        setStatus(
          await readErrorMessage(
            response,
            "Failed to send verification email.",
          ),
        );
        return;
      }
      const body = (await response.json()) as GenericMessageResponse;
      setStatus(body.message);
    }).catch(() => {
      setStatus("Failed to send verification email.");
    });
  }

  async function changePassword() {
    if (!currentPassword || !newPassword) {
      setStatus("Current and new password are required.");
      return;
    }

    await withAction("change-password", async () => {
      const response = await apiFetch("/api/profile/password", {
        method: "PUT",
        body: JSON.stringify({
          current_password: currentPassword,
          new_password: newPassword,
        }),
      });

      if (!response.ok) {
        setStatus(await readErrorMessage(response, "Password update failed."));
        return;
      }

      setStatus("Password updated successfully.");
      setCurrentPassword("");
      setNewPassword("");
    }).catch(() => {
      setStatus("Password update failed.");
    });
  }

  async function startMfaEnrollment() {
    await withAction("start-mfa", async () => {
      const response = await apiFetch("/auth/mfa/totp/enroll", {
        method: "POST",
      });
      if (!response.ok) {
        setStatus(
          await readErrorMessage(response, "MFA enrollment start failed."),
        );
        return;
      }

      setEnrollment((await response.json()) as TotpEnrollmentResponse);
      setStatus("Scan the secret and confirm with a TOTP code.");
    }).catch(() => {
      setStatus("MFA enrollment start failed.");
    });
  }

  async function confirmMfaEnrollment() {
    if (!confirmCode) {
      setStatus("Enter the TOTP code to confirm enrollment.");
      return;
    }

    await withAction("confirm-mfa", async () => {
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
    }).catch(() => {
      setStatus("MFA confirmation failed.");
    });
  }

  async function disableMfa() {
    if (!disablePassword) {
      setStatus("Password is required to disable MFA.");
      return;
    }

    await withAction("disable-mfa", async () => {
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
    }).catch(() => {
      setStatus("Failed to disable MFA.");
    });
  }

  async function regenerateRecoveryCodes() {
    if (!regeneratePassword) {
      setStatus("Password is required to regenerate recovery codes.");
      return;
    }

    await withAction("regenerate-codes", async () => {
      const response = await apiFetch("/auth/mfa/recovery-codes/regenerate", {
        method: "POST",
        body: JSON.stringify({ password: regeneratePassword }),
      });

      if (!response.ok) {
        setStatus(
          await readErrorMessage(
            response,
            "Failed to regenerate recovery codes.",
          ),
        );
        return;
      }

      const body = (await response.json()) as RecoveryCodesResponse;
      setNewRecoveryCodes(body.recovery_codes);
      setStatus("Recovery codes regenerated. Save them now.");
      setRegeneratePassword("");
    }).catch(() => {
      setStatus("Failed to regenerate recovery codes.");
    });
  }

  async function sendInvite() {
    if (!inviteEmail) {
      setStatus("Invite email is required.");
      return;
    }

    await withAction("send-invite", async () => {
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
    }).catch(() => {
      setStatus("Failed to send invite.");
    });
  }

  return {
    activeAction,
    confirmCode,
    currentPassword,
    disablePassword,
    enrollment,
    inviteEmail,
    inviteTenantName,
    me,
    newPassword,
    newRecoveryCodes,
    regeneratePassword,
    status,
    changePassword,
    confirmMfaEnrollment,
    disableMfa,
    loadMe,
    regenerateRecoveryCodes,
    resendVerification,
    sendInvite,
    setConfirmCode,
    setCurrentPassword,
    setDisablePassword,
    setInviteEmail,
    setInviteTenantName,
    setNewPassword,
    setRegeneratePassword,
    startMfaEnrollment,
  };
}
