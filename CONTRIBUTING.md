# Contributing to Pixli

Thank you for your interest in contributing to **Pixli**. This document covers how to get started, coding standards, and the pull request process.

## Getting started

1. **Fork the repository** on GitHub.
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/your-username/pixli.git
   cd pixli
   ```
3. **Build and test**:
   ```bash
   cargo build
   cargo test
   cargo run --example shooter
   ```
4. **Create a branch** for your change:
   ```bash
   git checkout -b feature/your-feature
   ```

## Coding standards

- **Rustfmt** — Format code with `cargo fmt`.
- **Clippy** — Run `cargo clippy` and fix reported warnings.
- **No unwraps on user-facing paths** — Use `Result` and `Option`; prefer `?` or explicit handling. Internal code may use `if let Some(...)` or early returns instead of panics.
- **Documentation** — Public APIs should have `///` doc comments. Generate docs with `cargo doc --open`.

## Testing

- Run the full test suite:
  ```bash
  cargo test
  ```
- Run the shooter example to verify rendering and input:
  ```bash
  cargo run --example shooter
  ```

## Pull request process

1. Ensure tests pass and the example runs.
2. Update **CHANGELOG.md** under `[Unreleased]` with your changes.
3. Open a Pull Request against the `main` branch.
4. Describe the change clearly and reference any related issues.

## Reporting issues

Please use the GitHub issue tracker. Include:

- Pixli version (or commit hash).
- Operating system and GPU/driver (for rendering or surface issues).
- Minimal steps or code to reproduce.

## License

By contributing, you agree that your contributions will be licensed under the MIT License, same as the project.
