# FreelanceInvoice

FreelanceInvoice is a Tauri desktop app for freelancers to track work, manage clients/projects, generate invoices, and produce AI-assisted project estimates.

## Features

- Clients, projects, and timer-based time tracking
- Invoice builder, sandboxed HTML invoice preview, and invoice history
- Dashboard analytics for revenue and hours
- AI project estimation (requires your own Claude API key)
- Stripe payment link generation (premium-tier)
- Manual time-entry workflow (create/edit)
- Downloadable PDF invoice export

## Tech Stack

- Tauri (Rust) + Vite + React + TypeScript
- Zustand for state
- Recharts for charts
- Tailwind CSS

## Getting Started

### Prerequisites

- Node.js
- pnpm
- For the desktop app: Rust toolchain + Tauri prerequisites for your OS

### Install

```bash
pnpm install
```

### Run (Web Dev)

```bash
pnpm dev
```

### Run (Web Dev, Lean Disk Mode)

```bash
pnpm dev:lean
```

### Run (Tauri Desktop)

```bash
pnpm tauri dev
```

### Run (Tauri Desktop, Lean Disk Mode)

```bash
pnpm tauri:dev:lean
```

### Build

```bash
pnpm build
```

To build a desktop bundle:

```bash
pnpm tauri build
```

## Normal Dev vs Lean Dev

- `pnpm dev` / `pnpm tauri dev` keep local build caches for faster restarts, but they grow disk usage over time.
- `pnpm dev:lean` / `pnpm tauri:dev:lean` redirect Vite and Cargo build caches to temporary OS folders and remove heavy build artifacts when you stop the app.

Tradeoff:

- Lean mode uses less persistent disk space.
- Lean mode usually has slower startup after each restart because caches are not reused.

## Cleanup Commands

Target heavy build artifacts only:

```bash
pnpm clean:heavy
```

This removes:

- `dist`
- `node_modules/.vite`
- `src-tauri/target`

Full local reproducible cleanup:

```bash
pnpm clean:all-local
```

This removes everything above plus:

- `node_modules`

After `pnpm clean:all-local`, run `pnpm install` before starting dev again.

## Configuration (API Keys)

You can set API keys in the app under **Settings**:

- Claude API key: used for AI project estimation
- Stripe API key: used for payment-link generation (premium-tier)

Security note: do not hardcode keys in code or commit them to the repo. Sensitive keys are stored through OS secure storage when saved in the app.

## Feature Maturity

Production scope and release claims are managed in:

- `docs/production-scope-contract.md`
- `docs/feature-maturity-policy.md`
- `docs/operator-runbook.md`

## Recommended IDE Setup

- VS Code + the Tauri extension + rust-analyzer
