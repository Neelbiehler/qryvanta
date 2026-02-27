import { redirect } from "next/navigation";

type MakerEntityFormDesignerPageProps = {
  params: Promise<{
    entityLogicalName: string;
    formLogicalName: string;
  }>;
};

export default async function MakerEntityFormDesignerPage({
  params: _params,
}: MakerEntityFormDesignerPageProps) {
  await _params;
  redirect("/maker/studio/_default");
}
