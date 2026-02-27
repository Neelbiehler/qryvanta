import { redirect } from "next/navigation";

type MakerEntityViewDesignerPageProps = {
  params: Promise<{
    entityLogicalName: string;
    viewLogicalName: string;
  }>;
};

export default async function MakerEntityViewDesignerPage({
  params: _params,
}: MakerEntityViewDesignerPageProps) {
  await _params;
  redirect("/maker/studio/_default");
}
