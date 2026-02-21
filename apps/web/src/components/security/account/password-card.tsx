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

type PasswordCardProps = {
  busy: boolean;
  currentPassword: string;
  newPassword: string;
  onChangePassword: () => void;
  onCurrentPasswordChange: (value: string) => void;
  onNewPasswordChange: (value: string) => void;
};

export function PasswordCard({
  busy,
  currentPassword,
  newPassword,
  onChangePassword,
  onCurrentPasswordChange,
  onNewPasswordChange,
}: PasswordCardProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Change Password</CardTitle>
        <CardDescription>Rotate your password without ending your current session.</CardDescription>
      </CardHeader>
      <CardContent className="space-y-3">
        <div className="space-y-2">
          <Label htmlFor="current-password">Current password</Label>
          <Input
            id="current-password"
            type="password"
            value={currentPassword}
            onChange={(event) => onCurrentPasswordChange(event.target.value)}
          />
        </div>
        <div className="space-y-2">
          <Label htmlFor="new-password">New password</Label>
          <Input
            id="new-password"
            type="password"
            value={newPassword}
            onChange={(event) => onNewPasswordChange(event.target.value)}
          />
        </div>
        <Button onClick={onChangePassword} disabled={busy} className="w-full">
          Update password
        </Button>
      </CardContent>
    </Card>
  );
}
