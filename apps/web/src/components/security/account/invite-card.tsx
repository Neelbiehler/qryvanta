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

type InviteCardProps = {
  busy: boolean;
  inviteEmail: string;
  inviteTenantName: string;
  onInviteEmailChange: (value: string) => void;
  onInviteTenantNameChange: (value: string) => void;
  onSendInvite: () => void;
};

export function InviteCard({
  busy,
  inviteEmail,
  inviteTenantName,
  onInviteEmailChange,
  onInviteTenantNameChange,
  onSendInvite,
}: InviteCardProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Invite Teammate</CardTitle>
        <CardDescription>
          Send a workspace invite link to a teammate.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-3">
        <div className="space-y-2">
          <Label htmlFor="invite-email">Email</Label>
          <Input
            id="invite-email"
            type="email"
            value={inviteEmail}
            onChange={(event) => onInviteEmailChange(event.target.value)}
            placeholder="teammate@company.com"
          />
        </div>
        <div className="space-y-2">
          <Label htmlFor="invite-tenant">Workspace name (optional)</Label>
          <Input
            id="invite-tenant"
            value={inviteTenantName}
            onChange={(event) => onInviteTenantNameChange(event.target.value)}
            placeholder="Acme Operations"
          />
        </div>
        <Button onClick={onSendInvite} disabled={busy} className="w-full">
          Send invite
        </Button>
      </CardContent>
    </Card>
  );
}
