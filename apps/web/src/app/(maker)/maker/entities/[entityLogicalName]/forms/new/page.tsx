import { redirect } from "next/navigation";

type MakerNewEntityFormPageProps = {
  params: Promise<{
    entityLogicalName: string;
  }>;
};

export default async function MakerNewEntityFormPage({
  params: _params,
}: MakerNewEntityFormPageProps) {
  await _params;
  redirect("/maker/studio/_default");
}
