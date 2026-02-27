import { redirect } from "next/navigation";

type MakerEntityViewsPageProps = {
  params: Promise<{
    entityLogicalName: string;
  }>;
};

export default async function MakerEntityViewsPage({
  params: _params,
}: MakerEntityViewsPageProps) {
  await _params;
  redirect("/maker/studio/_default");
}
