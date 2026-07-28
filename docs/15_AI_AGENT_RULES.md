# 15 - AI Agent Rules

Binding rules for whoever (human or AI) implements each phase.

## The ponytail ladder

See `00_PROJECT_CONSTITUTION.md`. Applied before writing any new code, every time, no exceptions except security/correctness/reliability/accessibility/ type-safety, which are never trimmed for simplicity.

## Dependency documentation check

Before adding or using any dependency, its **current** documentation is checked - package name, import path, current API shape - not recalled from training data. This project has already hit one real example of why: the animation library renamed from `framer-motion` to `motion` with a changed import path (`motion/react`); using stale knowledge here would have shipped a deprecated package. Record the check's outcome in `05_TECHNICAL_DECISIONS.md` if it's a new dependency, or note it inline in the relevant phase's memory entry if it's confirming an existing one.

## File size limit

**No code file (`.rs`, `.ts`, `.tsx`) exceeds 200 lines.** When a file approaches this, split by responsibility:

- **React components**: extract sub-components, custom hooks (`useX.ts`), and pure presentational pieces before a component file grows past the limit. A component that both fetches data and renders a complex tree is two files, not one.
- **Rust**: split a module into submodules (`mod.rs` + `handlers.rs` + `types.rs`, etc.) along natural seams - request handling vs. domain types vs. error types are typically separate files even within one Tauri command group.
- **Utils/helpers**: a `utils.ts` that's becoming a junk drawer is a sign it needs to become `utils/format.ts`, `utils/validation.ts`, etc. - named by what the functions do, not a generic catch-all.

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

A helper used by exactly one feature lives in that feature's `utils/`; a helper used by two or more lives in the top-level `utils/`. Don't pre-emptively generalize a single-use helper into the shared folder. always use kebab-case for file & folder names. don't ever use "—", "…", or any other special characters.

## Type safety

- `never use any` - literally: no `any`, no `as any` escape hatches, no implicit `any` from an unannotated parameter. Oxlint's no-explicit-any rule is an error, and `tsconfig.json` has `strict: true` plus explicit `noImplicitAny: true`.
- When a type is genuinely unknown at a boundary (e.g. a Tauri event payload), use `unknown` and narrow it with a type guard or a validation step - never assert past the boundary with `any`.
- Rust: `#![deny(clippy::all)]`-equivalent via the workspace lint table in `05_TECHNICAL_DECISIONS.md`; avoid `unwrap()`/`expect()` outside tests and truly-infallible cases - propagate `Result` with `thiserror`/`anyhow` instead.

## Linters/formatters - must pass before a phase is considered done

- `bun run check` (Ultracite with Oxlint + Oxfmt, including the React Doctor Oxlint plugin)
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --check`
- `tsc -b --noEmit` (typecheck)

## No placeholder implementations

A crate or module that legitimately has no logic yet, because its phase hasn't started, stays an honest, compiling stub with a doc comment pointing to the spec it will implement - never a fake/mocked version of the feature it will eventually be.

## React Doctor through Oxlint

React Doctor is enforced through Oxc's Oxlint plugin, not through a separate React Doctor CLI installation or command. Ultracite runs Oxlint and Oxfmt, and `oxlint-plugin-react-doctor` supplies the React-specific diagnostics.

The project configuration is `oxlint.config.ts`. It extends:

- `ultracite/oxlint/core`
- `ultracite/oxlint/js-plugins`, filtered to the React Doctor plugin

The required gate is therefore simply:

```bash
bun run check
```

Do not disable a React Doctor rule to make the check pass. Fix the underlying code unless a genuine false positive is documented with its rationale. The rules reference is maintained at <https://www.react.doctor/docs/rules> and the Oxlint integration is documented at <https://www.react.doctor/docs/configuration/eslint-and-oxlint-plugins>.

The current project-relevant requirements include:

- Keep components and hooks pure; do not mutate state during render.
- Keep hook dependency arrays complete and preserve manual memoization safely.
- Avoid unnecessary effects and derived state effects.
- Use stable keys for rendered collections; never use array indexes when a stable domain identifier exists.
- Use semantic HTML and valid WAI-ARIA attributes and roles.
- Give every interactive control an accessible name.
- Do not use `dangerouslySetInnerHTML`, `javascript:` URLs, or unsafe DOM mutation patterns.
- Respect reduced-motion preferences for every non-essential animation.
- Avoid excessive z-index values, render-blocking patterns, and avoidable re-renders.

When Oxlint reports a React Doctor rule, read its rule-specific recommendation before editing. Ultracite's single `bun run check` gate now covers formatting, core linting, and React Doctor diagnostics together.