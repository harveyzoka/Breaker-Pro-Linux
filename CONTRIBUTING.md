# Contributing

Thanks for improving Breaker Pro.

## Development Setup

Install Rust, Cargo, and GTK4 development headers before building.

```bash
sudo apt install build-essential rustc cargo libgtk-4-dev
cargo build
```

## Checks

Run these checks before opening a pull request:

```bash
cargo fmt --check
cargo check
```

## Pull Requests

- Keep changes focused and small.
- Update documentation when behavior changes.
- Include screenshots for visible UI changes.
- Avoid committing generated build artifacts.
