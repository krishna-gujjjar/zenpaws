# 11 - UI/UX Guidelines

## Component architecture

React function components only, typed props (no `any`, no untyped
`children: React.ReactNode` left implicit - always declared). Structure per
`15_AI_AGENT_RULES.md`'s decomposition rules: a component file stays under
200 lines by extracting hooks (`useX.ts`), pure helpers (`utils/`), and
sub-components rather than growing one file.

## Styling

Utility-first CSS (Tailwind) is the working assumption for velocity, kept
consistent with Motion for anything that's actually animated rather than
just transitioned via CSS. This is revisited concretely once Phase 1's
frontend shell exists - recorded as a decision to confirm, not yet locked.

## Motion usage

- Use `motion/react`'s declarative `animate`/`variants` API for state-driven
  UI motion (message send/receive, panel open/close, pet reactions bridging
  into UI, e.g. a notification toast).
- Respect `useReducedMotion()` - every non-essential animation must check it
  and fall back to an instant or minimal transition.
- Prefer one orchestrated moment over scattered micro-animations; excessive
  ambient motion both hurts the low-end-hardware CPU budget and reads as
  noisy rather than polished.

## Accessibility

Keyboard focus must be visible everywhere; every interactive element
reachable by keyboard; color contrast meets WCAG AA at minimum; screen
reader labels on icon-only buttons (e.g. reaction picker, context menu
triggers).

## Virtualized list UX

The 100k-message list must not visibly "pop" content in - use skeleton
placeholders or a fade-in tied to `useReducedMotion`-aware Motion, sized to
avoid layout shift as rows mount.

## Empty and error states

Per the interface's own voice, not a person's: explain what happened and
what to do next, plainly. An empty DM list says what a person can do next
(start a conversation with an online peer), not just "No messages yet."

## Detailed visual design (colors, type, layout)

Deferred to when actual screens are built (Phase 5+) rather than speculated
here - a full token system, aesthetic direction, and asset production
happens against real content, not this planning document, following the
`frontend-design` guidance available in the build environment.
