---
"@qryvanta/ui": patch
---

Fix Next.js App Router server/client boundaries by keeping the `@qryvanta/ui` root export server-safe and moving dropdown menu primitives to the client-only subpath `@qryvanta/ui/dropdown-menu`.
