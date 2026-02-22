# @qryvanta/ui

Shared React UI primitives used across Qryvanta surfaces.

This package is based on shadcn/ui patterns and adapted for Qryvanta product surfaces.

## Install

```bash
pnpm add @qryvanta/ui
```

## Usage

```tsx
import { Button, Card, CardContent, CardHeader, CardTitle } from "@qryvanta/ui";

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@qryvanta/ui/dropdown-menu";
```

## Local Development

```bash
pnpm --filter @qryvanta/ui check
pnpm --filter @qryvanta/ui build
```

## Release Process

1. Add or update components in `src/`.
2. Add a changeset (`pnpm changeset`) when making package changes.
3. Merge to `main` and let the release workflow publish.

For manual publish:

```bash
pnpm --filter @qryvanta/ui build
pnpm --filter @qryvanta/ui publish --access public
```
