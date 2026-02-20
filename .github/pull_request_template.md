## Summary

- Describe the motivation and the behavior change.
- Link relevant issues.

## Changes

- List key implementation changes by area (API, domain, web, docs).

## Validation

- [ ] `cargo fmt --all`
- [ ] `cargo xcheck`
- [ ] `cargo xclippy`
- [ ] `cargo xtest`
- [ ] `pnpm check`
- [ ] `pnpm lint`
- [ ] `pnpm test`

## Documentation

- [ ] Updated docs under `apps/docs/content/docs` when behavior or operations changed
- [ ] Updated `README.md` and/or `AGENTS.md` if contributor workflow changed

## API Contracts

- [ ] Not applicable (no DTO changes)
- [ ] Ran `pnpm contracts:generate` and `pnpm contracts:check`

## Checklist

- [ ] Follows architecture boundaries (`domain` -> `application` -> `infrastructure` -> `apps/api`)
- [ ] No secrets committed
- [ ] Includes tests for behavior changes
