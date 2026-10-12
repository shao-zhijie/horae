# Contributing

## Before you start

- [AGENTS.md](AGENTS.md) — the authoritative contributor guide: layering, the mutation seam, migration rules, ID resolution, and the invariants that break things if violated. Read it before your first PR.
- [ARCHITECTURE.md](ARCHITECTURE.md) — module map, data model, scheduling invariants and domain vocabulary.
- `.agents/skills/horae/` — a skill pack for AI assistants (install, CLI/TUI reference, workflows). If you change user-facing behaviour, update it too.

## Local setup

Rust 1.89 or newer (the MSRV job in CI pins the same version). SQLite is bundled, so there is no C dependency to install.

```sh
cargo build --workspace
cargo run -p horae-cli -- capture "buy milk" --tag home   # note the `--` before subcommand args
```

Use `HORAE_CONFIG_DIR=/tmp/horae-test` to point the binary at a throwaway data directory while testing.

## Checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

CI runs exactly these, plus an MSRV job. A local pre-commit hook in `.githooks/` runs the same three commands; install it with `git config core.hooksPath .githooks`.

## Tests

- Unit tests live next to the code they cover — notably `crates/horae-tui/src/tui/tests.rs`, and `parser.rs`, `schedule.rs`, `time.rs`, `repo/backup.rs` in the core.
- End-to-end CLI tests live in `crates/horae-cli/tests/cli.rs` and drive the real binary inside an isolated `HORAE_CONFIG_DIR`. Add one there whenever you change CLI behaviour.
- `tempfile` is available for throwaway databases. JSON state stores can be pointed at a test path with `JsonStateStore::at(path)`; `state::set_test_override()` is global and sticky, so prefer `at()` in tests.

## Conventions

- **Time**: store UTC milliseconds as INTEGER. All arithmetic goes through `time.rs`; all display goes through `format_local`.
- **Migrations**: never edit an existing file in `crates/horae-core/migrations/`. Add a file and a new version block in `db/migrate.rs`.
- **Events**: a new event type means a constant in `model/event.rs` and a matching entry in the `task_events` comment in `0001_init.sql`.
- **Writes**: go through `repo::mutate` so the change and its audit event share one `now`.
- **TUI text**: localized with `crate::tr!` and `Lang`; never hardcode a UI string. Every glyph goes through `App::icon(Icon::…)`, never a literal Nerd Font codepoint.
- **Errors**: domain errors are `crate::error::Error`; command handlers return `anyhow::Result`.
- **Recurrence**: validate with `parser::rrule_valid` at every write path; an unsupported frequency must be an error, not a silent downgrade.

## Documentation

`README.md` is the English manual and `README_CN.md` its Chinese mirror. They are kept structurally identical — when you add or change a user-facing feature, update both in the same commit. Add a `CHANGELOG.md` entry under `Unreleased` for anything a user would notice.

## Pull requests

Keep each PR to one concern, with a commit message that says what changed and why. Pass fmt, clippy with `-D warnings`, and the full test suite before opening it.
