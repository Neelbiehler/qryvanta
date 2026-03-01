import Link from "next/link";

const year = new Date().getFullYear();

type FooterStripProps = {
  docsUrl: string;
  githubUrl: string;
};

export function FooterStrip({ docsUrl, githubUrl }: FooterStripProps) {
  return (
    <footer className="mt-10 border-t border-emerald-100/80 pt-5">
      <div className="flex flex-col gap-3 text-xs text-slate-500 sm:flex-row sm:items-center sm:justify-between">
        <div className="flex flex-col gap-1 sm:flex-row sm:items-center sm:gap-4">
          <span>&copy; {year} Qryvanta. Open-source project.</span>
          <span className="hidden sm:inline text-emerald-200">|</span>
          <span>No personal data collected. No cookies set.</span>
        </div>

        <div className="flex items-center gap-4">
          <Link
            href={docsUrl}
            className="transition-colors hover:text-slate-700"
          >
            Docs
          </Link>
          <Link
            href={githubUrl}
            className="transition-colors hover:text-slate-700"
          >
            GitHub
          </Link>
          <span className="text-emerald-200">|</span>
          <Link href="/impressum" className="transition-colors hover:text-slate-700">
            Impressum
          </Link>
          <Link href="/privacy" className="transition-colors hover:text-slate-700">
            Privacy
          </Link>
        </div>
      </div>
    </footer>
  );
}
