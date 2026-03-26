# Puppet Master Entrypoints

This is the first documentation page to open when you start work in Puppet Master.

## Choose Your Path

Pick exactly one entrypoint based on your goal:

| If your goal is... | Start here |
|---|---|
| Build or change the PM framework itself | [Framework Development](./framework-development.md) |
| Launch a new product on PM from scratch | [Greenfield Build](./greenfield-build.md) |
| Migrate or upgrade an existing product | [Brownfield Migration](./brownfield-migration.md) |

## Fast Decision Rule

1. New codebase on PM -> Greenfield.
2. Existing codebase moving to PM (or PM version upgrade) -> Brownfield.
3. Core framework behavior, contracts, architecture, or release policy changes -> Framework Development.

## First 15 Minutes

1. Open your chosen entrypoint file above.
2. Run setup (`/pm-init` in Claude/Qwen, or `npm run init`).
3. Run quality gates:
   - `npm run lint`
   - `npm run test:run`
   - `npm run build`
4. Read release contracts before touching framework behavior:
   - `docs/releases/README.md`
   - latest release in `docs/releases/`

## Related Documents

- Main project onboarding: `README.md`
- Full docs index: `docs/README.md`
- Contribution and framework rules: `CONTRIBUTING.md`
