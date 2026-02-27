import { redirect } from "next/navigation";

type MakerEntityFormsPageProps = {
  params: Promise<{
    entityLogicalName: string;
  }>;
};

export default async function MakerEntityFormsPage({
  params: _params,
}: MakerEntityFormsPageProps) {
  await _params;
  redirect("/maker/studio/_default");
}
