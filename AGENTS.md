# AGENTS.md - tomo

Pomodoro timer TUI app. Workspace: `tomo` (main binary) + `crates/ratatui-toaster` (local toast library).

## Quick Commands

```bash
./dev.sh all        # format + lint + test (run before committing)
./dev.sh format     # cargo +nightly fmt --all
./dev.sh lint       # clippy --fix --allow-dirty
./dev.sh test       # cargo test --all-features
./dev.sh docs       # Compile Mermaid diagrams in docs/diagrams/
```

## CI Order (must pass in sequence)

1. `cargo +nightly fmt --all -- --check`
2. `cargo build --all-features --all-targets`
3. `cargo clippy --all-features --all-targets --no-deps -- -D warnings`
4. `cargo test --all-features`

## Architecture

Three layers with strict ownership boundaries:

| Layer | Owns | Never holds |
|-------|------|-------------|
| **Core** (`AppCore`) | Pomodoro, Config, Router, sessions, `overlay`, `config_snapshot` | View-local state |
| **Views** (`TuiTimerView`, `TuiSettingsView`) | Scroll, selected item, text prompt, keybind visibility | Copies of core state |
| **Runner** (`TuiRunner`) | Terminal, tick timer, input polling | Business state |

Views render from **borrowed** core state (`&Pomodoro`, `&Config`). Use `dsp!()` macro on `TuiRunner` (`dsp!(self, pomo/timer/setting/router/core, msg)`) to dispatch messages.

## Domain vs Data Boundary

```
Core/Views/Runner        ← Domain types only (Mode, Session, Task, Config)
TuiEffectHandler         ← Bridges Cmd enums to repo calls
repo traits              ← Return domain models, accept Mode
repo sqlite + model      ← Map Row → domain before returning; From impls live here
```

- Domain models: `src/model/` (Task, Session, Project, Tag, Mode, Pomodoro)
- Database rows (`SessionRow`, `TaskRow`, `PomodoroState`): private to `src/repo/`
- Core must never import `crate::repo::model::*`
- `From<Row>` impls live in `src/repo/model.rs`

## Testing

- Inline tests in source files (`#[cfg(test)]` blocks)
- Test function names use **no `test_` prefix** (e.g. `fn navigate()`, not `fn test_navigate()`)
- Integration tests: `tests/db.rs` (raw Diesel queries, tests schema relationships)
- Run: `cargo test --all-features`

## Conventions

- Edition 2024. Nightly required for formatting (rustfmt uses unstable features).
- ConfigMsg / SettingsMsg / PomodoroMsg enums: **do not add docstrings** to variants; their names are self-explanatory.
- Structs, traits, and public functions at architecture boundaries: **do add docstrings** per RFC 1574 (short summary, no headers).
- `Cmd` is emitted by Core for side-effects. `Msg` is emitted to Core when outside world needs to tell it something.
- Do not manually edit `Cargo.toml` version (CI handles it).

## Project Structure

```
src/
├── main.rs              # Entry: CLI → Config → TuiRunner::run()
├── lib.rs               # APP_NAME / APP_VERSION
├── cli.rs               # Clap CLI definition
├── config.rs            # Serde YAML config (Percentage, Timers, Hooks, Alarms)
├── model/               # Domain models (Pomodoro, Mode, Task, Session, Project, Tag)
├── repo/                # Diesel SQLite layer with embedded migrations
│   ├── traits.rs        # Repo traits return domain models
│   ├── model.rs         # DB Row types + From impls
│   ├── sqlite.rs        # SqliteRepo impls (Row → domain mapping)
│   └── schema.rs        # Diesel auto-generated schema
├── service/             # Alarm (rodio), notify, cmd_runner (hooks)
└── ui/
    ├── core.rs          # AppCore (Pomodoro, Config, Router, overlay state)
    ├── router.rs        # Page {Timer, Settings} state machine
    ├── traits.rs        # Runner, EffectHandler, Updateable<Msg, Cmd>
    ├── prelude.rs       # Re-exports
    ├── update/          # Msg/Cmd types
    │   ├── pomo.rs      # PomodoroMsg/Cmd, TimerMsg/Cmd
    │   └── config.rs    # ConfigMsg, SettingsMsg/Cmd, SettingsItem/SettingsSection
    └── tui/
        ├── backend.rs   # Crossterm terminal init/restore
        ├── runner.rs    # TuiRunner: event loop, dsp! macro dispatches
        ├── effect.rs    # TuiEffectHandler: executes Cmd (sound, DB, hooks, toasts)
        └── view/
            ├── timer.rs
            ├── settings/     # Split into mod.rs + widgets.rs + parse.rs
            └── warnings.rs   # DuplicateWarning, UnsavedWarning, ResetWarning
```

Binaries: `tomo` (default), `sandbox` (`src/bin/sandbox.rs`).

## Rust Toolchain

- Edition 2024, nightly fmt
- Key deps: clap, ratatui 0.30, crossterm, diesel, rodio, strum, serde

## Workspace: ratatui-toaster

Local crate at `crates/ratatui-toaster/`. Uses `ratatui` 0.30 with `unstable-widget-ref`. Optional `tokio` feature for auto-hide timers.

## System Dependencies (Linux)

`pkg-config libfontconfig1-dev libsqlite3-dev libasound2-dev`

## Changelog & Release

- Do not edit `Cargo.toml` version. CI updates it in `release` branch.
- `[pub]` or `[public]` tags in commit → included in changelog.
- `chore!(major)` → major, `chore!(minor)` → minor, default → patch.

## Further Docs

- `docs/dev/ui-architecture.md` — full state ownership rules, domain/data boundary, message flow
- `docs/dev/ui-tui-layout.md` — TUI layout definitions
- `docs/dev/commits.md` — commit conventions and changelog generation
