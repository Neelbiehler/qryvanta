"use client";

import { AccountSnapshotCard } from "@/components/security/account/account-snapshot-card";
import { InviteCard } from "@/components/security/account/invite-card";
import { MfaCard } from "@/components/security/account/mfa-card";
import { PasswordCard } from "@/components/security/account/password-card";
import { useSecurityAccount } from "@/components/security/account/use-security-account";

export default function AdminSecurityAccountPage() {
  const {
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
  } = useSecurityAccount();

  const busy = activeAction !== null;

  return (
    <div className="space-y-6">
      <div>
        <p className="text-xs uppercase tracking-[0.18em] text-zinc-500">
          Admin Center
        </p>
        <h1 className="font-serif text-3xl text-zinc-900">
          Security Settings
        </h1>
        <p className="mt-2 text-sm text-zinc-600">
          Manage your account verification, password, MFA, and tenant invites.
        </p>
      </div>

      <div className="grid gap-6 lg:grid-cols-2">
        <AccountSnapshotCard
          busy={busy}
          me={me}
          onLoadMe={loadMe}
          onResendVerification={resendVerification}
        />
        <PasswordCard
          busy={busy}
          currentPassword={currentPassword}
          newPassword={newPassword}
          onChangePassword={changePassword}
          onCurrentPasswordChange={setCurrentPassword}
          onNewPasswordChange={setNewPassword}
        />
        <MfaCard
          busy={busy}
          confirmCode={confirmCode}
          disablePassword={disablePassword}
          enrollment={enrollment}
          newRecoveryCodes={newRecoveryCodes}
          regeneratePassword={regeneratePassword}
          onConfirmCodeChange={setConfirmCode}
          onConfirmMfaEnrollment={confirmMfaEnrollment}
          onDisableMfa={disableMfa}
          onDisablePasswordChange={setDisablePassword}
          onRegenerateCodes={regenerateRecoveryCodes}
          onRegeneratePasswordChange={setRegeneratePassword}
          onStartMfaEnrollment={startMfaEnrollment}
        />
        <InviteCard
          busy={busy}
          inviteEmail={inviteEmail}
          inviteTenantName={inviteTenantName}
          onInviteEmailChange={setInviteEmail}
          onInviteTenantNameChange={setInviteTenantName}
          onSendInvite={sendInvite}
        />
      </div>

      {status ? (
        <p
          aria-live="polite"
          className="rounded-md border border-zinc-200 bg-zinc-50 p-3 text-sm text-zinc-700"
        >
          {status}
        </p>
      ) : null}
    </div>
  );
}
