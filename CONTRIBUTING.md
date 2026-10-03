# Contributing

Thank you for your interest in contributing!

## Getting Started

1. Fork the repository and follow [Quick Start prerequisites and installation](README.md#quick-start)
2. Create a feature branch: `git checkout -b feat/your-feature`
3. Make your changes
4. Commit using [Conventional Commits](https://www.conventionalcommits.org/): `feat:`, `fix:`, `chore:`, etc.
5. Push and open a pull request

## Verification

Run commands from the repository root on a feature branch, after `pnpm install --frozen-lockfile`. Use pnpm 10.28.1 to match performance CI. If inspecting a linked worktree and installing hooks would change its shared Git configuration, use a standalone clone; `pnpm install --frozen-lockfile --ignore-scripts` skips all lifecycle scripts and hook setup, so it is not equivalent to the normal installation.

The current Git tree contains both `.github/PULL_REQUEST_TEMPLATE.md` and `.github/pull_request_template.md` with different contents. A case-insensitive macOS checkout can therefore report the uppercase path as modified immediately after cloning. Inspect the diff and do not stage it as part of an unrelated change.

Start with the checks affected by the change:

| Scope                            | Command                                                                         | What it covers                                                                         |
| -------------------------------- | ------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| Small frontend check             | `pnpm exec vitest run src/lib/formatters.test.ts`                               | Existing pure formatting cases in the configured jsdom environment                     |
| Changed frontend component/store | `pnpm exec vitest run src/components/shared/Button.test.tsx`                    | Replace the example with the affected test file; Tauri calls in store tests are mocked |
| Frontend suite                   | `pnpm test:unit`                                                                | All configured Vitest tests                                                            |
| Focused Rust database check      | `cargo test --locked --manifest-path src-tauri/Cargo.toml db::clients::tests`   | Existing client tests using in-memory SQLite                                           |
| Rust suite                       | `pnpm test:integration`                                                         | Cargo tests for the Tauri crate; requires native platform build prerequisites          |
| Types                            | `pnpm typecheck`                                                                | TypeScript without emitting output                                                     |
| Existing formatting scope        | `pnpm lint`                                                                     | Prettier checks the paths listed in `package.json`; this is not a source-code linter   |
| Documentation formatting         | `pnpm exec prettier --check README.md CONTRIBUTING.md docs/operator-runbook.md` | Includes CONTRIBUTING, which the existing lint script does not cover                   |
| Frontend build and budget        | `pnpm build`                                                                    | TypeScript, Vite assets, then `size:check`                                             |

For a broader pre-release pass, use the repository authority:

```bash
bash .codex/scripts/run_verify_commands.sh
```

Read [`.codex/verify.commands`](.codex/verify.commands) for its current command order. It includes staged-file Git guards (which require `gitleaks` locally), frontend/Rust tests, HTTP smoke, and local performance commands. It writes build output and `.perf-results` in the checkout. It does not prove signed packaging, external providers, or rendered UI behavior. A failed check remains a gate; do not treat `build:raw` as a replacement for `build` or raise bundle limits to make verification pass.

### UI and route checks

When UI behavior changes, run the affected frontend tests and inspect the changed screens in a browser. Use `pnpm dev` for frontend layout; persistence and Tauri commands need the native app and disposable records. Keep Claude/Stripe credentials unset for local checks, and verify provider behavior only in a separately authorized test environment.

The automated e2e script checks HTTP responses and the app-shell HTML for six routes; it does not drive a browser or verify native database behavior. It reuses an existing `dist/index.html`, so build fresh assets before running it:

```bash
pnpm build:raw
pnpm test:e2e
```

These smoke checks bind a temporary preview server to `127.0.0.1:45173` and stop it afterward; the port must be free. `pnpm test` runs unit, Rust, then this HTTP smoke, so refresh `dist` first for that command too. `build:raw` is useful for this diagnostic lane while a size gate is failing; `pnpm build` must still pass for build readiness.

### Optional performance and desktop checks

`pnpm perf:bundle`, `pnpm perf:build`, `pnpm perf:assets`, and `pnpm perf:memory` are local lanes used by the verification bundle. Run the build before inspecting its bundle. `perf:build` invokes the gated build itself. The memory smoke is a synthetic Node allocation check, not native-app memory profiling.

Both Lighthouse configs upload to temporary public storage when run with `pnpm perf:lhci` or `pnpm perf:lhci:prod`. To collect and assert locally without that upload, build fresh assets, then run:

```bash
pnpm exec lhci collect --config=lighthouserc.json
pnpm exec lhci assert --config=lighthouserc.json
```

This lane requires a Chrome/Chromium installation and a free `127.0.0.1:45174` preview port. The foundation config uses warning thresholds; [`.lighthouserc.production.json`](.lighthouserc.production.json) contains the production assertions. CI currently runs the performance jobs with Node 20 and pnpm 10.28.1; those jobs are not coverage of the full local unit/native/hook lanes. Its API/DB jobs run only when the corresponding repository variables are configured and target those external systems. Do not supply production URLs or run those external lanes as a local smoke check.

For a native compilation check, see [Operator Runbook](docs/operator-runbook.md#pre-release-checks). A debug build does not prove signing, notarization, installation, or provider behavior.

## Reporting Issues

Open a [GitHub Issue](../../issues) with a clear description and steps to reproduce.

## Code Style

Follow the existing conventions in the codebase.
