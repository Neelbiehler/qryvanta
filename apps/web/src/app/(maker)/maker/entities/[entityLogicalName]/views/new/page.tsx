import { redirect } from "next/navigation";

type MakerNewEntityViewPageProps = {
  params: Promise<{
    entityLogicalName: string;
  }>;
};

export default async function MakerNewEntityViewPage({
  params: _params,
}: MakerNewEntityViewPageProps) {
  await _params;
  redirect("/maker/studio/_default");
}
