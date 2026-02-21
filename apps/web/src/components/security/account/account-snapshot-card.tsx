import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@qryvanta/ui";

import type { UserIdentityResponse } from "@/lib/api";

type AccountSnapshotCardProps = {
  busy: boolean;
  me: UserIdentityResponse | null;
  onLoadMe: () => void;
  onResendVerification: () => void;
};

export function AccountSnapshotCard({
  busy,
  me,
  onLoadMe,
  onResendVerification,
}: AccountSnapshotCardProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Account Snapshot</CardTitle>
        <CardDescription>
          Check the authenticated identity currently loaded.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-3">
        <Button
          onClick={onLoadMe}
          disabled={busy}
          variant="outline"
          className="w-full"
        >
          Refresh account details
        </Button>
        {me ? (
          <div className="rounded-md border border-zinc-200 bg-zinc-50 p-3 text-sm text-zinc-700">
            <p>
              <strong>Display:</strong> {me.display_name}
            </p>
            <p>
              <strong>Email:</strong> {me.email ?? "n/a"}
            </p>
            <p>
              <strong>Subject:</strong> {me.subject}
            </p>
            <p>
              <strong>Tenant:</strong> {me.tenant_id}
            </p>
          </div>
        ) : null}
        <Button
          onClick={onResendVerification}
          disabled={busy}
          className="w-full"
        >
          Resend verification email
        </Button>
      </CardContent>
    </Card>
  );
}
