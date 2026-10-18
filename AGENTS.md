<!-- BEGIN:turborepo-agent-rules -->

# This is NOT the Turborepo you know

Turborepo configuration, task behavior, and CLI commands can vary between installed versions and may differ from your training data. Resolve the `turbo` package from this file's directory or relevant workspace; in monorepos, it may not be visible from the repository root. For example, run `node -p "require.resolve('turbo/package.json')"` from a workspace that depends on `turbo`.

Read `docs/README.md` inside that installed package first, then read the relevant pages from its `docs/` directory before changing Turborepo configuration or commands. Heed deprecation notices. These bundled docs match the installed package version and are available without network access.

This block is written and re-added by `turbo` before repository-scoped commands when an AI agent is detected. In the Turborepo source repository, its template is defined in `crates/turborepo-cli/src/cli/agent_guidance.rs`. Removing the managed block while updates are enabled means a later qualifying invocation will add it again. Set `"agentGuidance": false` in the root `turbo.json` or `turbo.jsonc` to opt out; this does not remove an existing block. Keep the block committed with your work to avoid an uncommitted change on the next agent invocation.
<!-- END:turborepo-agent-rules -->

# Components

- Keep `apps/web/src/app` for routes: pages, layouts, and their server actions and data access. Put React components elsewhere.
- Put components specific to one page in `apps/web/src/components/{page}`, e.g. `apps/web/src/components/admin` for `/admin`. Import them with the `@/components/...` alias.
- Before writing a component, check whether part of it is a general UI building block (e.g. a modal, a JSON viewer, a data table). If it can be extracted in a simple, clean, reusable way, put it in `packages/ui` and compose it from the page component.
  - Shared components take data and callbacks through props. They do not import API types, server actions, or anything else app-specific.
  - Do not generalise speculatively. Extract only when the result is simpler than the inline version, and keep page-specific wording and behaviour in the page component.
  - Each file in `packages/ui/src` is its own entry point: `src/modal.tsx` is imported as `@repo/ui/modal`. The package compiles to `dist`, so run `pnpm --filter @repo/ui build` (or `dev`) before the app sees changes. Mark components that use hooks or event handlers with `"use client"`.
