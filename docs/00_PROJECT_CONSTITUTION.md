# 00 - Project Constitution

This is the condensed, always-in-force ruleset for building ZenPaws. The
full rationale lives in `MASTER_BUILD_PROMPT.md` at the repo root; this file
is the quick-reference version every phase is checked against.

## What ZenPaws is

A production-grade, cross-platform, LAN-only peer-to-peer desktop messenger
with interactive desktop pets, built on Tauri v2 + Rust + React 19.

## Coding discipline - the ponytail ladder

Before writing any code, stop at the first rung that holds:

1. Does this need to exist at all?
2. Is it already in this codebase?
3. Does the Rust/TS standard library do it?
4. Does the OS or Tauri do it natively?
5. Does an already-installed dependency do it?
6. Can it be done in one line?
7. Only then: the minimum new code that correctly solves the problem.

Never trimmed by this ladder: security, correctness, reliability,
accessibility, type safety.

## Code quality bar (applies from Phase 1 onward)

- **No file over 200 lines** for any code file (`.rs`, `.ts`, `.tsx`). Split
  by responsibility into components/hooks/utils/modules - see
  `15_AI_AGENT_RULES.md` for the exact decomposition rules.
- **No `any`** in TypeScript, anywhere, including generics defaults and
  catch clauses. Biome's `noExplicitAny` is set to `error`, not `warn`.
- Every dependency is checked against its own current documentation before
  use - not assumed from training data - since APIs and package names
  change (e.g. `framer-motion` → `motion`).
- Rust: `cargo clippy -- -D warnings` and `cargo fmt --check` must pass.
  TypeScript: `biome check .` must pass. Both are wired into CI from Phase 1.

## Process rules

- Never generate the entire project in one response; one phase at a time.
- Wait for explicit approval before starting the next phase.
- Never generate placeholder/mocked/TODO-only implementations unless
  explicitly requested - a crate that legitimately has no logic yet (because
  its phase hasn't started) stays an honestly empty, compiling stub with a
  doc comment, not a fake implementation.
- If a requirement is ambiguous or two requirements conflict, stop and ask.
- After every phase: update `16_PROJECT_MEMORY.md`, `17_CHANGELOG.md`,
  `18_BACKLOG.md`, `19_KNOWN_ISSUES.md`, and commit the phase as one atomic
  git commit.
- Architecture quality over implementation speed - but "quality" here means
  correctly minimal per the ladder above, not maximal abstraction.

## Non-goals

No WAN/internet relay, no mobile clients, no multi-workspace topology, no
resistance to a nation-state-level adversary on the LAN. See
`03_PROJECT_SCOPE.md` for the full boundary.
