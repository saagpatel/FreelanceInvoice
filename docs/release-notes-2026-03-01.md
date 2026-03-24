# Release Notes - 2026-03-01

## Highlights

- Added atomic invoice draft save with transactional line-item creation and time-entry linkage.
- Added manual time entry create/edit workflow in desktop UI.
- Added Stripe payment-link creation flow (premium-gated) with invoice-level link persistence.
- Added downloadable PDF invoice export while preserving HTML preview.
- Expanded invoice lifecycle actions to include overdue and cancelled transitions.
- Updated quality gates to include correctness checks before performance checks.

## Reliability And Data Integrity

- Timer pause/resume now preserves original session start and correct accumulated duration.
- Invoiced time entries are linked during invoice creation in the same transaction.
- Nullable client/project fields can now be explicitly cleared in update flows.

## Quality Gates

Verified with:

- `bash .codex/scripts/run_verify_commands.sh`
- `pnpm tauri build --debug --no-bundle`

All checks passed in this implementation pass.

## Operational Notes

- Tauri bundle identifier updated to `com.freelanceinvoice.desktop`.
- Legacy data path fallback remains in place to avoid breaking existing local installs.

## Known Follow-Ups

- Add broader route-state and accessibility evidence capture for every high-traffic flow.
- Finalize signed/notarized release process for production distribution targets.
