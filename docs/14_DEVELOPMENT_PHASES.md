# 14 - Development Phases

| Phase | Name | Status |
|---|---|---|
| 0 | Documentation and Architecture | ✅ Complete |
| 1 | Project Foundation | Complete |
| 2 | Networking Foundation | Complete |
| 3 | Database Foundation | Complete |
| 4 | Pet Engine | Complete |
| 5 | Chat System | In progress |
| 6 | Image System | Not started |
| 7 | File Transfer System | Not started |
| 8 | Notification System | Not started |
| 9 | Performance Optimization | Not started |
| 10 | Testing and Hardening | Not started |

Never skip phases. Status is changed only with the corresponding updates to
`16_PROJECT_MEMORY.md`, `17_CHANGELOG.md`, `18_BACKLOG.md`, and
`19_KNOWN_ISSUES.md`.

## Definition of Done (applies to every phase)

1. The code and docs cover that phase only; later-phase concerns remain
   deferred.
2. Project memory, changelog, backlog, and known issues are updated.
3. The applicable Bun and Cargo quality gates have passed and their results
   are recorded.
4. The phase is one atomic git commit (`phase-N: <summary>`).
5. A ponytail-ladder self-review states what was not built, what was reused,
   and what genuinely required new code.

After this checklist, record Definition of Done, Open Questions, Risks, and
Next Phase Plan, then wait for approval before a subsequent phase.
