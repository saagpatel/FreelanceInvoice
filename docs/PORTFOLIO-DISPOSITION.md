# FreeLanceInvoice — Portfolio Disposition

**Status:** Release Frozen — unsigned desktop pipeline complete (PR #3
merged to `origin/main` 2026-03-24), awaiting operator-only Apple
signing + notarization. Fifth member of the signing-frozen cluster.

> **Audience:** anyone resuming FreeLanceInvoice work, scoping a
> signing batch, or wondering why the prior version of this file
> claimed code was stranded.

---

## Correction notice

A previous version of this disposition doc (in PR #21) claimed that
substantive Stripe + PDF + dashboard code was stranded on the
`codex/feat/release-candidate-closeout` branch and not on `main`.
**That claim was wrong.** It was based on reading the wrong remote:
the analysis looked at `legacy-origin/main` (the frozen `saagar210`
GitHub account) and conflated it with `origin/main`
(`saagpatel/FreelanceInvoice`).

The reality:

- PR #3 merged into `origin/main` 2026-03-24 (merge commit
  `381b73b`).
- `src-tauri/src/services/stripe.rs`, `secure_store.rs`,
  `commands/pdf.rs`, and `commands/invoices.rs` payment-link
  functions all exist on `origin/main` and have since the merge.
- The 700 lines were never stranded.

This file replaces the prior disposition. **No code recovery PR is
needed.**

---

## Why the earlier mistake happened (worth remembering)

The local clone of this repo had `main` tracking `legacy-origin/main`
(the saagar210 account's `main`), which had been frozen with a
different history that ended before the closeout merge. When the
analysis ran `git log origin/main`, the local resolver returned
`legacy-origin/main` because of the tracking config. Without the
trailing `origin/` always-explicit prefix in commands, the wrong
branch was inspected.

**Lesson for any future disposition work in this repo or any other
repo with a `legacy-origin` remote**: always use the literal
`origin/<branch>` form in `git log`, `git merge-base`, and
`git diff` commands. Never trust local tracking config when the
disposition decision hinges on which remote is being read.

This is the same `legacy-origin` migration risk noted in the
PersonalKBDrafter, Relay, and DeepTank disposition docs — but
FreeLanceInvoice is the first case where the misread actually
produced a wrong claim. Worth a one-time sweep of `legacy-origin`
repos to check whether any other dispositions written in this
session have the same bug.

---

## Current state in one paragraph

FreeLanceInvoice is a Tauri 2 desktop app for freelancers: client/
project management, timer-based time tracking, invoice builder with
sandboxed HTML preview and history, premium-tier Stripe payment
links, PDF invoice download, secure invoice settings, and
AI-assisted project estimates. Operator runbook, go/no-go doc, and
production scope contract are captured under `docs/`. Quality gates
defined in `.codex/verify.commands`. The product surface for v0 is
shipped on `main` and tested.

The only gates between "tag a release" and "publish a signed installer"
are Apple Developer ID credentials and the canonical signing /
notarization workflow.

---

## Portfolio operating system instructions

| Aspect               | Posture                                                                                                                                                                         |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Portfolio status     | `Release Frozen`                                                                                                                                                                |
| Closeout packet      | **Resolved** — PR #3 merged 2026-03-24                                                                                                                                          |
| Review cadence       | Suspend overdue counting                                                                                                                                                        |
| Resurface conditions | (a) Apple signing credentials wired in CI, (b) operator opens a v0.2 scope packet, or (c) operator triggers a dependency-refresh sprint for the 5 open dependabot PRs (#16–#20) |
| Co-batch with        | Signing cluster: DesktopPEt, ContentEngine, AIGCCore, Relay, FreeLanceInvoice. **Now 5 repos** — even more reason to batch through signing in one session.                      |

---

## Unblock trigger (operator)

When ready to ship:

1. Add Apple Developer ID + notarization credentials per the
   standard signing-cluster procedure.
2. Bump version across `package.json`, `src-tauri/tauri.conf.json`,
   `src-tauri/Cargo.toml`. Tag `v0.x.y`.
3. Verify the auto-generated draft release contains signed installers
   and a valid `SHA256SUMS.txt`.
4. Publish.
5. Triage open dependabot PRs (#16–#20) opportunistically — none are
   blocking.

Estimated operator time once credentials are in hand: ~2 hours
including a fresh notarization round-trip on macOS.

---

## Reactivation procedure (for the next code session)

When portfolio operating system flips this row to `Active`:

1. **Fix local clone tracking first.** Check `git branch -vv` — if
   `main` tracks `legacy-origin/main`, retarget to `origin/main`
   with `git branch --set-upstream-to=origin/main main`. This was
   the underlying cause of the wrong disposition claim.
2. Delete stale `codex/*` branches (most are merged-history
   artifacts).
3. Re-run `pnpm install && pnpm verify` to confirm the toolchain
   still works after the freeze.
4. Re-run the operator runbook smoke walkthrough before adding any
   new scope.
5. Only then proceed to signing.

---

## Last known reference

| Field                                   | Value                                                                                                                                                                                |
| --------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Last meaningful commit on `origin/main` | `4d9a1f0` docs: portfolio disposition + pnpm workspace fix (#21) — note this file replaces what that PR shipped                                                                      |
| Closeout merge                          | `381b73b` Merge pull request #3 (present on `origin/main`)                                                                                                                           |
| Stripe/PDF/dashboard code               | On `origin/main` since 2026-03-24                                                                                                                                                    |
| Build verification status               | green                                                                                                                                                                                |
| Open dependabot PRs                     | #16 – #20, oldest from 2026-03-29                                                                                                                                                    |
| Blocker                                 | Apple signing + notarization (operator-only)                                                                                                                                         |
| Migration note                          | `legacy-origin` points at frozen `saagar210/FreelanceInvoice`; **do not push there**. Local clones may track legacy-origin/main by accident — verify before relying on branch state. |
