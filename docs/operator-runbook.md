# Operator Runbook

This runbook covers day-to-day operation, verification, and local recovery for FreeLanceInvoice.

## Pre-Release Checks

Run from repo root:

```bash
bash .codex/scripts/run_verify_commands.sh
pnpm tauri build --debug --no-bundle
```

Expected result:

- all verify commands pass
- desktop debug build succeeds

## Local Data Location

The desktop app stores local SQLite data in the user local data directory:

- current app path: `com.freelanceinvoice.desktop`
- legacy fallback path: `com.freelanceinvoice.app` (used automatically when legacy data exists)

Database filename:

- `freelanceinvoice.db`

## Backup Procedure

1. Close the app.
2. Copy `freelanceinvoice.db` to a timestamped backup file.
3. Store backups outside the repo working directory.

Example:

```bash
cp "$HOME/Library/Application Support/com.freelanceinvoice.desktop/freelanceinvoice.db" \
  "$HOME/Backups/freelanceinvoice-$(date +%Y%m%d-%H%M%S).db"
```

## Restore Procedure

1. Close the app.
2. Replace current `freelanceinvoice.db` with a chosen backup.
3. Re-open app and validate dashboard, invoices, and time entries.

## Incident Triage Basics

- If invoice save fails:
  - confirm client is selected and line items are valid.
  - check time entries are not already invoiced.
- If Stripe link creation fails:
  - confirm plan tier is `premium`.
  - confirm Stripe API key is set in Settings.
  - confirm invoice total is greater than zero.
- If PDF export fails:
  - confirm invoice exists and contains at least one line item.

## Release Notes Checklist

- summarize user-visible capability changes
- summarize migration/compatibility notes
- summarize known limitations and deferred items
- include commands run for verification
