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

import type { TotpEnrollmentResponse } from "./use-security-account";

type MfaCardProps = {
  busy: boolean;
  confirmCode: string;
  disablePassword: string;
  enrollment: TotpEnrollmentResponse | null;
  newRecoveryCodes: string[];
  regeneratePassword: string;
  onConfirmCodeChange: (value: string) => void;
  onConfirmMfaEnrollment: () => void;
  onDisableMfa: () => void;
  onDisablePasswordChange: (value: string) => void;
  onRegenerateCodes: () => void;
  onRegeneratePasswordChange: (value: string) => void;
  onStartMfaEnrollment: () => void;
};

export function MfaCard({
  busy,
  confirmCode,
  disablePassword,
  enrollment,
  newRecoveryCodes,
  regeneratePassword,
  onConfirmCodeChange,
  onConfirmMfaEnrollment,
  onDisableMfa,
  onDisablePasswordChange,
  onRegenerateCodes,
  onRegeneratePasswordChange,
  onStartMfaEnrollment,
}: MfaCardProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>MFA (TOTP)</CardTitle>
        <CardDescription>
          Enable authenticator-based MFA and manage recovery codes.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-3">
        <Button
          onClick={onStartMfaEnrollment}
          disabled={busy}
          variant="outline"
          className="w-full"
        >
          Start TOTP enrollment
        </Button>
        {enrollment ? (
          <div className="space-y-2 rounded-md border border-emerald-100 bg-emerald-50/70 p-3 text-sm">
            <p>
              <strong>Secret:</strong> {enrollment.secret_base32}
            </p>
            <p className="break-all">
              <strong>URI:</strong> {enrollment.otpauth_uri}
            </p>
            <p>
              <strong>Recovery Codes:</strong>{" "}
              {enrollment.recovery_codes.join(", ")}
            </p>
          </div>
        ) : null}
        <div className="space-y-2">
          <Label htmlFor="confirm-mfa">Confirm code</Label>
          <Input
            id="confirm-mfa"
            value={confirmCode}
            onChange={(event) => onConfirmCodeChange(event.target.value)}
            placeholder="123456"
          />
        </div>
        <Button
          onClick={onConfirmMfaEnrollment}
          disabled={busy}
          className="w-full"
        >
          Confirm MFA enrollment
        </Button>
        <div className="space-y-2 pt-2">
          <Label htmlFor="disable-password">Disable MFA password</Label>
          <Input
            id="disable-password"
            type="password"
            value={disablePassword}
            onChange={(event) => onDisablePasswordChange(event.target.value)}
          />
        </div>
        <Button
          onClick={onDisableMfa}
          disabled={busy}
          variant="outline"
          className="w-full"
        >
          Disable MFA
        </Button>
        <div className="space-y-2 pt-2">
          <Label htmlFor="regen-password">Regenerate codes password</Label>
          <Input
            id="regen-password"
            type="password"
            value={regeneratePassword}
            onChange={(event) => onRegeneratePasswordChange(event.target.value)}
          />
        </div>
        <Button
          onClick={onRegenerateCodes}
          disabled={busy}
          variant="outline"
          className="w-full"
        >
          Regenerate recovery codes
        </Button>
        {newRecoveryCodes.length > 0 ? (
          <p className="rounded-md border border-amber-200 bg-amber-50 p-3 text-sm text-amber-900">
            New codes: {newRecoveryCodes.join(", ")}
          </p>
        ) : null}
      </CardContent>
    </Card>
  );
}
