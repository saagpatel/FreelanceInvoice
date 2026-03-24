# Feature Maturity Policy

This policy defines how FreeLanceInvoice classifies feature readiness and when a feature can move into production claims.

## Maturity Levels

### Implemented

Feature is complete for current scope and can be described as available in README and in-app UI.

Required:

- Core behavior implemented.
- Error handling in place for user-facing failures.
- Covered by required tests.
- Included in local verify + CI contracts.
- User-facing docs and copy updated.

### Stabilizing

Feature exists but has known completion work before it can be claimed as fully production-ready.

Required:

- Feature behind explicit wording as in-progress.
- Known gaps tracked in scope contract.
- Regression plan and acceptance criteria defined.
- No misleading claims in README or product copy.

### Planned

Feature direction is approved but implementation has not started or is not yet user-available.

Required:

- Explicitly marked as planned.
- No production claim language.
- Dependencies and ownership documented before implementation starts.

## Promotion Rules

- Planned -> Stabilizing:
  - Scope and acceptance criteria locked.
  - Implementation work started with measurable checkpoints.

- Stabilizing -> Implemented:
  - All high-priority gaps closed.
  - Mandatory tests pass in local verify and CI.
  - Security/reliability checks pass for affected surfaces.
  - README and UI copy describe actual behavior.

## Gate Policy

- Critical path uses stable or beta workflows only.
- Experimental tooling can be used only with clear fallback and no hard dependency for release-critical gates.
- Gate waivers must be explicit and documented; no implicit waivers.

## Reporting Rules

- Default status reporting remains PM-friendly and big-picture.
- Technical details stay in implementation artifacts unless requested.
- Any maturity change must include:
  - impacted capability,
  - evidence used,
  - updated claim language.
