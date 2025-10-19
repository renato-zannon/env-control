# Repository Guidelines

## Project Structure & Module Organization
- `Cargo.toml` defines the CLI metadata and dependencies.
- `src/main.rs` drives argument parsing, environment mutation, and the `exec` flow.
- `src/path_set.rs` provides iterators for colon-delimited values; keep related tests nearby.
- `target/` holds build artifacts; clean it before timing or packaging releases, and lean on `README.md` for user-facing guidance.

## Build, Test, and Development Commands
- `cargo build` for debug work; `cargo build --release` for distribution.
- `cargo run -- --help` smoke-tests clap wiring after changing flags.
- `cargo fmt --all` applies the repository’s `rustfmt.toml`; run it before committing.
- `cargo clippy --all-targets --all-features` should pass before opening a pull request.

## Coding Style & Naming Conventions
- Use Rust 2024 defaults with 4-space indentation and no tabs.
- Keep `snake_case` for values/modules and `UpperCamelCase` for types; make modules small and exports explicit.
- Prefer `?` with the shared `AppResult<T>` alias instead of manual error plumbing.
- Let `rustfmt` dictate layout and add comments only when behaviour is non-obvious.

## Commit & Pull Request Guidelines
- Match existing history: short (≤50 char) imperative subjects such as `Fix positional argument parsing`, with detail in the body when necessary.
- Keep formatting-only edits separate and group changes by behaviour.
- Pull requests must explain motivation, list commands run (`cargo fmt`, `cargo test`, `cargo clippy`), and call out user-visible changes.
- Link issues when applicable and include sample invocations or output diffs for behaviour shifts.

## Release & Distribution Notes
- Install locally with `cargo install --path . --locked --profile release` to lock dependencies.
- Announce feature updates in `README.md` Features/Examples before tagging a release.
- Strip `target/` artefacts and verify the release binary prior to distribution.
