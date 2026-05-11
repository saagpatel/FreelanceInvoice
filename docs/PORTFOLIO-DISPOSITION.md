# FreeLanceInvoice — Portfolio Disposition

**Status:** Release Frozen **with unmerged code work**. Substantive
post-closeout commits (Stripe payment links, PDF invoice workflows,
dashboard hardening, time-tracking tightening) live on
`codex/feat/release-candidate-closeout` and have not landed on
`main`. Resolving that gap is a prerequisite for any signing /
shipping work.

> **Audience:** anyone resuming FreeLanceInvoice work or wondering
> why the `main` branch is missing features the release PR claimed
> to deliver.

---

## Why this file exists

Two reasons:

1. The portfolio operating system has been surfacing FreeLanceInvoice
   as overdue review. The repo deserves a Release Frozen disposition
   (same family as DesktopPEt / ContentEngine / AIGCCore / Relay).
2. **But there's a real problem on top of that posture**: substantive
   code work on `codex/feat/release-candidate-closeout` is not on
   `main`, despite PR #3 ("feat(app): finalize release candidate
   closeout") being marked merged on 2026-03-24.

The disposition is not just "wait for signing" — it's "first
reconcile the missing code, then wait for signing."

---

## The unmerged work

PR #3 was reported merged 2026-03-24 with merge commit `381b73b`.
That merge commit does not exist on `origin/main` today. The most
likely explanation is that `main` was reset or force-pushed after
the merge (possibly during the `saagar210` → `saagpatel` GitHub
account migration), orphaning the merge commit and stranding the
feature work on the codex branch.

Commits on the branch but **not on `main`**:

| Commit                    | Subject                                              | Why it matters                                                                                                                                                               |
| ------------------------- | ---------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `33d6298`                 | feat(app): tighten dashboard and time-tracking flows | Hardens dashboard, time-entry editing, AI estimate behavior (~8 files)                                                                                                       |
| `ee9209d`                 | feat(invoice): add payment-link and pdf workflows    | Stripe payment links + PDF invoice download + secure invoice settings (~15 files, 700+ lines of code, includes new `src-tauri/src/services/stripe.rs` and `secure_store.rs`) |
| `974af35`                 | chore(repo): add release candidate quality gates     | Codifies verify contract for unit/integration/e2e/perf                                                                                                                       |
| 5 codex bootstrap commits | various                                              | Lower priority; many already on `main` via separate paths                                                                                                                    |

The Stripe integration is the highest-value missing piece. It is
not test-coverage polish — it's a feature the release-candidate
narrative claims to have shipped, but main does not actually have.

---

## Recommended sequence

Do **not** treat this row as simple Release Frozen. The right
sequence:

1. **Recover the code first**: open a fresh PR from
   `codex/feat/release-candidate-closeout` (or a cleaner cherry-pick
   of the three substantive commits above) to `main`. Verify the
   tests pass in CI. Merge.
2. Run `pnpm verify` / `cargo test` / the smoke walkthrough
   against the post-merge `main` to confirm the resurrected work
   actually still builds against current toolchain.
3. **Only then** treat this row as Release Frozen, with the same
   Apple signing unblock as the rest of the cluster.

Doing signing first while the code on `main` is missing features
would ship a release that's a regression against what was nominally
finished six weeks ago.

---

## Current state in one paragraph

FreeLanceInvoice is a Tauri 2 desktop app for freelancers to track
work, manage clients and projects, generate invoices, and produce
AI-assisted project estimates. Operator runbook, go/no-go doc, and
production scope contract are captured under `docs/`. The
release-candidate work is **mostly merged** to main, but the Stripe
payment-link integration and dashboard hardening pass have been
stranded on `codex/feat/release-candidate-closeout` since the
account migration.

---

## Portfolio operating system instructions

| Aspect                        | Posture                                                                                                                                                                             |
| ----------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Portfolio status              | **`Active`** until unmerged code is reconciled — not `Release Frozen` yet                                                                                                           |
| Critical follow-up            | Recover the Stripe + PDF + dashboard commits from `codex/feat/release-candidate-closeout` to `main`                                                                                 |
| Review cadence                | Resume normal cadence — this row needs decision-time, not waiting-time                                                                                                              |
| After code recovery           | Transition to `Release Frozen` + join the signing cluster                                                                                                                           |
| Co-batch with signing cluster | **No, not yet.** Recovering the missing code is a prerequisite. Once that PR merges and the branch is reconciled, then co-batch with DesktopPEt / ContentEngine / AIGCCore / Relay. |

---

## Why "Active" instead of "Release Frozen" right now

The signing-cluster repos all have one common property: **what's on
`main` is what would ship if signing were available**. That is not
true here. Shipping what `main` currently has would produce a
release missing the Stripe integration that was nominally the
flagship feature of the closeout. That's a regression, not a
release.

After the code is recovered onto `main`, this row joins the cluster
and becomes Release Frozen on the same axis.

---

## Unblock procedure (operator + Claude Code)

Step 1 — Code recovery (Claude Code can do this in a future session):

1. Branch from current `main`.
2. Cherry-pick `974af35`, `ee9209d`, `33d6298` in order.
3. Resolve any conflicts (likely none against current main since
   main is mostly chore commits since the orphaning).
4. Open a PR titled "feat(app): recover release-candidate work stranded
   from PR #3".
5. Verify CI green, merge.
6. Delete the long-stale `codex/feat/release-candidate-closeout`
   branch.

Step 2 — Then standard Release Frozen unblock (operator):

7. Apple signing + notarization credentials.
8. Re-run the operator runbook smoke walkthrough.
9. Cut a real GitHub release.

---

## Last known reference

| Field                             | Value                                                                                                                                   |
| --------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| Last commit on `main`             | `9a7ff07` refactor(core): prune dead code and harden bundle budgets                                                                     |
| Last commit on stranded branch    | `33d6298` feat(app): tighten dashboard and time-tracking flows                                                                          |
| Stranded commits worth recovering | `974af35`, `ee9209d`, `33d6298`                                                                                                         |
| Stranded commit count             | 8 (3 substantive, 5 chore/bootstrap)                                                                                                    |
| Build verification status         | green on the stranded branch; main has not been verified post-stranding                                                                 |
| Open dependabot PRs               | #16 – #20, oldest from 2026-03-29                                                                                                       |
| Blocker                           | (1) code recovery, (2) Apple signing                                                                                                    |
| Migration note                    | `legacy-origin` points at frozen `saagar210/FreelanceInvoice`; this is likely where the original merge commit lives — do not push there |
