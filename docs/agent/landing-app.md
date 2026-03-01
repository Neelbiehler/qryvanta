# Landing App Notes

Qryvanta includes a public landing app in `apps/landing`.

This app exists to keep project messaging separate from authenticated workspace surfaces in `apps/web`.

## Local Run

```bash
pnpm --filter @qryvanta/landing dev
```

Default URL: `http://localhost:3003`

## Current Stack

- Next.js App Router
- React
- Tailwind CSS
- Shared primitives from `@qryvanta/ui`

Keep OSS messaging (`qryvanta.org`) and cloud messaging (`qryvanta.com`) clearly separated.
