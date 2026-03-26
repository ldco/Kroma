# Releases

This directory is the canonical release log for framework users upgrading existing Puppet Master projects.

## Versioning Policy

Puppet Master follows Semantic Versioning:

- `MAJOR` (`X.0.0`): breaking changes that require project-level migration.
- `MINOR` (`1.X.0`): backwards-compatible features and platform improvements.
- `PATCH` (`1.X.Y`): backwards-compatible fixes.

## How to Use Release Notes

For each upgrade:

1. Read the target release file in this directory.
2. Apply all required migration steps in the "Upgrade Actions" section.
3. Run verification commands in your project:
   - `npm run lint`
   - `npm run build`
   - `npm run test:run`
4. Deploy only after verification passes.

## Available Releases

| Version | Date | Summary |
|---------|------|---------|
| [1.3.1](./1.3.1.md) | 2026-03-10 | **Latest** — Documentation correction patch for 1.3.0: empty-state envelope consistency, command/composable reference fixes, security wording alignment |
| [1.3.0](./1.3.0.md) | 2026-03-07 | API contract hardening + security improvements: strict response envelope, centralized RBAC, encrypted secrets storage, S3 visibility contract |
| [1.2.1](./1.2.1.md) | 2026-02-21 | Patch follow-up for 1.2.0: width-key migration clarity, onepager/admin contract hardening, release migration guidance |
| [1.2.0](./1.2.0.md) | 2026-02-20 | Scrollytelling baseline + global timeline contract, style/security hardening, docs entrypoint triad, header/footer width-mode migration |
| [1.1.0](./1.1.0.md) | 2026-02-20 | Stability, API hardening, CSS consistency, test runner reliability |

## Release Feedback Packages

| Target Release | Source Project | Date | File |
|---|---|---|---|
| `1.2.0` | EugeniaLatuWeb | 2026-02-21 | [docs/releases/feedback/1.2.0-eugenialatuweb.md](./feedback/1.2.0-eugenialatuweb.md) |
