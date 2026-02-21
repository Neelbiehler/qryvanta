import { AcceptInviteForm } from "@/components/auth/accept-invite-form";

type AcceptInvitePageProps = {
  searchParams?: Promise<{
    token?: string;
  }>;
};

export default async function AcceptInvitePage({
  searchParams,
}: AcceptInvitePageProps) {
  const resolvedSearchParams = (await searchParams) ?? {};
  return <AcceptInviteForm token={resolvedSearchParams.token ?? ""} />;
}
