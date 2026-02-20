import { Card, CardContent, CardHeader, CardTitle } from "@qryvanta/ui";

type AccessDeniedCardProps = {
  section: string;
  title: string;
  message: string;
};

export function AccessDeniedCard({
  section,
  title,
  message,
}: AccessDeniedCardProps) {
  return (
    <Card>
      <CardHeader>
        <p className="text-xs uppercase tracking-[0.18em] text-zinc-500">{section}</p>
        <CardTitle className="font-serif text-3xl">{title}</CardTitle>
      </CardHeader>
      <CardContent>
        <p className="text-sm text-zinc-600">{message}</p>
      </CardContent>
    </Card>
  );
}
