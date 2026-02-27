import { redirect } from "next/navigation";

type MakerAppSitemapPageProps = {
  params: Promise<{
    appLogicalName: string;
  }>;
};

export default async function MakerAppSitemapPage({ params }: MakerAppSitemapPageProps) {
  const { appLogicalName } = await params;
  redirect(`/maker/studio/${encodeURIComponent(appLogicalName)}`);
}
