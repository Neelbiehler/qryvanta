import Link from 'next/link';

export default function HomePage() {
  return (
    <div className="mx-auto flex w-full max-w-3xl flex-1 flex-col justify-center gap-5 px-6 text-center">
      <p className="text-sm uppercase tracking-[0.2em] text-fd-muted-foreground">
        Open Source + Self Hostable
      </p>
      <h1 className="text-4xl font-semibold tracking-tight">Qryvanta Documentation</h1>
      <p className="text-lg text-fd-muted-foreground">
        Build a metadata-driven business platform with transparent architecture, durable operations,
        and no vendor lock-in.
      </p>
      <div className="flex items-center justify-center gap-4">
        <Link
          href="/docs"
          className="rounded-md bg-fd-primary px-4 py-2 text-sm font-medium text-fd-primary-foreground"
        >
          Open Docs
        </Link>
        <Link href="/docs/quickstart" className="text-sm font-medium underline">
          Quickstart
        </Link>
      </div>
    </div>
  );
}
