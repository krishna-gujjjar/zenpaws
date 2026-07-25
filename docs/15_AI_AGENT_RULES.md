# 15 - AI Agent Rules

Binding rules for whoever (human or AI) implements each phase.

## The ponytail ladder

See `00_PROJECT_CONSTITUTION.md`. Applied before writing any new code, every
time, no exceptions except security/correctness/reliability/accessibility/
type-safety, which are never trimmed for simplicity.

## Dependency documentation check

Before adding or using any dependency, its **current** documentation is
checked - package name, import path, current API shape - not recalled from
training data. This project has already hit one real example of why: the
animation library renamed from `framer-motion` to `motion` with a changed
import path (`motion/react`); using stale knowledge here would have shipped
a deprecated package. Record the check's outcome in `05_TECHNICAL_DECISIONS.md`
if it's a new dependency, or note it inline in the relevant phase's memory
entry if it's confirming an existing one.

## File size limit

**No code file (`.rs`, `.ts`, `.tsx`) exceeds 200 lines.** When a file
approaches this, split by responsibility:

- **React components**: extract sub-components, custom hooks (`useX.ts`),
  and pure presentational pieces before a component file grows past the
  limit. A component that both fetches data and renders a complex tree is
  two files, not one.
- **Rust**: split a module into submodules (`mod.rs` + `handlers.rs` +
  `types.rs`, etc.) along natural seams - request handling vs. domain
  types vs. error types are typically separate files even within one
  Tauri command group.
- **Utils/helpers**: a `utils.ts` that's becoming a junk drawer is a sign
  it needs to become `utils/format.ts`, `utils/validation.ts`, etc. - named
  by what the functions do, not a generic catch-all.

## Component/utility structure convention

```
src/
  app/            # app shell, providers, routing (if any)
  features/       # one folder per feature area (chat/, pets/, transfers/, settings/)
    chat/
      components/
      hooks/
      utils/
  components/     # cross-feature shared UI primitives
  hooks/          # cross-feature shared hooks
  utils/          # cross-feature pure helpers (no side effects, no React)
  stores/         # Zustand stores
  queries/        # TanStack Query hooks/keys
```

A helper used by exactly one feature lives in that feature's `utils/`; a
helper used by two or more lives in the top-level `utils/`. Don't
pre-emptively generalize a single-use helper into the shared folder.
always use kebab-case for file & folder names. don't ever use "—", "…", or any other special characters.

## Type safety

- `never use any` - literally: no `any`, no `as any` escape hatches, no
  implicit `any` from an unannotated parameter. Biome's
  `linter.rules.suspicious.noExplicitAny` is `error`, and `tsconfig.json`
  has `strict: true` plus explicit `noImplicitAny: true`.
- When a type is genuinely unknown at a boundary (e.g. a Tauri event
  payload), use `unknown` and narrow it with a type guard or a validation
  step - never assert past the boundary with `any`.
- Rust: `#![deny(clippy::all)]`-equivalent via the workspace lint table in
  `05_TECHNICAL_DECISIONS.md`; avoid `unwrap()`/`expect()` outside tests and
  truly-infallible cases - propagate `Result` with `thiserror`/`anyhow`
  instead.

## Linters/formatters - must pass before a phase is considered done

- `biome check .` (lint + format check)
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --check`
- `tsc -b --noEmit` (typecheck)

## No placeholder implementations

A crate or module that legitimately has no logic yet, because its phase
hasn't started, stays an honest, compiling stub with a doc comment pointing
to the spec it will implement - never a fake/mocked version of the feature
it will eventually be.