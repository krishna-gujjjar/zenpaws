# 22 - Contributing Guide

## Workflow

- One phase = one atomic commit (`phase-N: <summary>`), per
  `00_PROJECT_CONSTITUTION.md`. Within a phase, smaller commits are fine as
  long as the phase's final state is a clean, buildable commit.
- After Phase 10, ordinary work follows Conventional Commits
  (`feat:`, `fix:`, `refactor:`, `docs:`, `test:`) since the phase-gate
  discipline is specific to the initial build.

## Before every commit

```bash
# Frontend
bun run check
bun run typecheck

# Backend (run from src-tauri/)
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo nextest run
```

All four must pass. This isn't optional CI-will-catch-it - run it locally
first.

## Code review checklist

- [ ] No file over 200 lines (see `15_AI_AGENT_RULES.md` for how to split).
- [ ] No `any` in TypeScript, no unexplained `unwrap()`/`expect()` in Rust.
- [ ] New dependency? Its current docs were checked, and the decision (with
      rationale) is recorded in `05_TECHNICAL_DECISIONS.md`.
- [ ] New Tauri command? Its capability-file reachability was updated in
      `12_SECURITY_MODEL.md`'s per-window scoping.
- [ ] Touches the DB schema? A new migration file was added - an existing
      one was never edited.
- [ ] Docs (`16_PROJECT_MEMORY.md`, `17_CHANGELOG.md`, and any spec doc the
      change affects) updated in the same PR, not deferred.

## Ponytail self-check

Before opening a PR, answer in the description: what did this change decide
NOT to build (YAGNI), what did it reuse instead of adding, and what actually
needed new code? This mirrors the phase-level self-review in
`14_DEVELOPMENT_PHASES.md`'s Definition of Done.
