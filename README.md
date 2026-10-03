# FreelanceInvoice

[![TypeScript](https://img.shields.io/badge/TypeScript-%233178c6?style=flat-square&logo=typescript)](#) [![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](#) [![Platform](https://img.shields.io/badge/platform-macOS-lightgrey?style=flat-square)](#)

> Stop losing money to unbilled hours and forgotten projects — a freelancer's complete billing stack, offline and on your machine.

FreelanceInvoice is a Tauri desktop app that handles the full client billing workflow: track time against projects, manage your client roster, build invoices from tracked sessions, and export polished PDFs — all without a subscription or a third-party SaaS account. Optional Claude AI integration generates project estimates so you can scope work more accurately before committing.

## Features

- **Timer-Based Time Tracking** — Start/stop timers per project; create manual time entries when you worked without the app open
- **Invoice Builder** — Assemble invoices from time entries, preview with sandboxed HTML renderer, and export to PDF
- **Client & Project Management** — Full CRUD for clients and projects with project-level rate configuration
- **Revenue Dashboard** — Charts for billed hours, revenue by client, and invoice status at a glance via Recharts
- **AI Project Estimation** — Feed a project brief to Claude and get a structured estimate (requires your own Claude API key)
- **Stripe Payment Links** — Generate Stripe payment links directly from invoices (premium tier)

## Quick Start

### Prerequisites

- Node.js 22.22.1+ on the 22.x line, or 24+ (the current lockfile includes Vite 8, jsdom 29, and lint-staged 17)
- pnpm 10.28.1, matching the performance CI workflows
- For Rust tests and desktop commands: stable Rust plus the [Tauri v2 platform prerequisites](https://v2.tauri.app/start/prerequisites/); on macOS, install Xcode Command Line Tools
- For the Git verification guards: `gitleaks` on `PATH`

### Installation

```bash
git clone https://github.com/saagpatel/FreelanceInvoice.git
cd FreelanceInvoice
pnpm install --frozen-lockfile
```

Run all commands from the repository root. `.env` is optional for the host/cache overrides in `.env.example`; local unit tests do not need Claude or Stripe credentials. Installation runs the `prepare` script to install Husky hooks in this checkout.

### Run (development)

```bash
pnpm dev        # Vite frontend only
pnpm tauri dev  # Native desktop app and frontend
```

The native app opens its local billing database. Use disposable test data or a separate OS test account for scenarios that write records or settings.

### Build

```bash
pnpm build        # TypeScript, frontend assets, and the strict bundle-size gate
pnpm tauri build  # Native desktop package; also runs pnpm build first
```

Desktop packaging needs the platform prerequisites above; signed distribution has additional release requirements. For focused tests, broader verification, and UI smoke checks, see [Contributing](CONTRIBUTING.md#verification).

## Tech Stack

| Layer         | Technology                |
| ------------- | ------------------------- |
| Desktop shell | Tauri 2 + Rust            |
| Frontend      | React + TypeScript + Vite |
| State         | Zustand                   |
| Charts        | Recharts                  |
| Styling       | Tailwind CSS              |
| AI estimation | Anthropic Claude API      |
| Payments      | Stripe                    |

## Architecture

FreelanceInvoice is a Tauri 2 monorepo with a Rust backend owning all persistent state (SQLite), business logic, and PDF generation. The React frontend communicates via Tauri's typed command interface. Invoice preview uses a sandboxed WebView to render HTML templates safely before PDF export. AI estimation and Stripe integration are optional capabilities gated behind environment variables.

## License

MIT
