# Production Scope Contract

This document defines what FreeLanceInvoice can claim as production-ready today, what is still being completed, and what is out of scope for the current release cycle.

## Scope Lock

- This contract is the source of truth for product claims.
- README and in-app copy must not claim a capability as complete unless it is listed as `Implemented` here.
- Any scope change requires updating this file and `docs/feature-maturity-policy.md` in the same change.

## Capability Matrix

| Capability                                | Current Status | Production Claim Allowed | In-Scope Path                                             |
| ----------------------------------------- | -------------- | ------------------------ | --------------------------------------------------------- |
| Clients CRUD                              | Implemented    | Yes                      | Keep covered by unit + integration checks                 |
| Projects CRUD                             | Implemented    | Yes                      | Keep covered by unit + integration checks                 |
| Timer lifecycle (start/pause/resume/stop) | Implemented    | Yes                      | Harden audit semantics and regression tests               |
| Invoice draft/list/status history         | Implemented    | Yes                      | Keep status transitions and tests aligned                 |
| Dashboard analytics                       | Implemented    | Yes                      | Maintain existing query/test coverage                     |
| AI estimation (Claude key)                | Implemented    | Yes                      | Keep key handling and error mapping stable                |
| Atomic invoice save with time-linking     | Implemented    | Yes                      | Maintain transactional save and linkage test coverage     |
| Stripe payment links (premium gated)      | Implemented    | Yes                      | Keep premium gate, secure URLs, and error handling stable |
| Manual time-entry create/edit             | Implemented    | Yes                      | Maintain validation and edit-path regression coverage     |
| Downloadable PDF export                   | Implemented    | Yes                      | Keep HTML preview and PDF export both covered             |
| Release packaging + readiness packet      | In Progress    | No                       | Complete release gates, RC smoke, and closeout docs       |

## Out Of Scope For This Cycle

- Multi-platform simultaneous GA launch.
- Non-essential redesign work not tied to capability completion.
- Experimental Codex features in production-critical paths without stable fallback.

## Default Release Decisions

- Stripe remains in scope and is premium-tier gated.
- Quality gates are strict by default (no implicit waivers).
- Release sequencing is staged, not all-platform-at-once.

## Ownership And Change Control

- Product scope owner: PM lane.
- Execution owner: codex-execution-os lane.
- A scope change is valid only when:
  - capability matrix row is updated,
  - maturity policy impact is documented,
  - acceptance criteria and gates are updated.
