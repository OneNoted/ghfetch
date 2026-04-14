# ghfetch

[![CI](https://github.com/OneNoted/ghfetch/actions/workflows/ci.yml/badge.svg)](https://github.com/OneNoted/ghfetch/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/ghfetch.svg)](https://crates.io/crates/ghfetch)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)

GitHub stats in the terminal, neofetch-style.

`ghfetch` renders compact terminal cards for GitHub users, repositories, and organizations, with optional JSON output for scripting.

## Features

- User, repository, and organization views
- Catppuccin-based terminal themes
- Optional JSON output for shell scripts and other tooling
- Language breakdowns with detailed table mode
- Contribution and streak stats when authenticated

## Installation

### From crates.io

```bash
cargo install ghfetch
```

### From the AUR

```bash
paru -S ghfetch-rs-bin
paru -S ghfetch-rs-git
```

### From source

```bash
git clone https://github.com/OneNoted/ghfetch.git
cd ghfetch
cargo install --path .
```

## Authentication

`ghfetch` checks for authentication in this order:

1. `--token`
2. `GITHUB_TOKEN`
3. `GH_TOKEN`
4. `gh auth token`

Unauthenticated mode still works for public data, but GitHub rate limits are much lower and contribution data is unavailable.

## Usage

```bash
ghfetch octocat
ghfetch user octocat --all
ghfetch repo rust-lang/rust
ghfetch repo https://github.com/rust-lang/rust
ghfetch org rust-lang --languages
ghfetch octocat --json
ghfetch repo rust-lang/rust --theme latte
```

### Commands

- `ghfetch [username]`
- `ghfetch user <username>`
- `ghfetch repo <owner/repo|github-url|ssh-remote>`
- `ghfetch org <orgname>`

### Common flags

- `--json` prints structured output instead of a card
- `--no-color` disables ANSI styling
- `--theme <mocha|macchiato|frappe|latte>` selects the card palette
- `--verbose` prints API request diagnostics to stderr

## Notes

- `ghfetch user <username>` shows a compact summary by default. Use `--all` to include every section, or specific section flags like `--repos` or `--languages`.
- `ghfetch org <orgname>` and `ghfetch repo <owner/repo|github-url|ssh-remote>` show language summaries by default.
- Detailed language mode (`--languages`) prints a wider table instead of the card view.

## Development

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## Release Automation

This repository ships two AUR packages:

- `ghfetch-rs-bin` for tagged GitHub release artifacts
- `ghfetch-rs-git` for the live `main` branch

The package names are suffixed with `-rs` because `ghfetch` is already taken in the AUR, but both packages still install the `ghfetch` executable.

### GitHub Actions flow

- Pushing a `vX.Y.Z` tag builds `ghfetch-X.Y.Z-x86_64-unknown-linux-gnu.tar.gz`, publishes a GitHub Release, and updates `ghfetch-rs-bin`
- Pushing to `main` refreshes `ghfetch-rs-git`
- CI also renders both AUR package definitions to catch metadata regressions early

### Required repository secret

Add `AUR_SSH_PRIVATE_KEY` to the GitHub repository secrets. It should be the private key for the AUR account `notes`, with the matching public key registered in that AUR account.

The generated AUR commits use:

- `Jonatan Jonasson`
- `notes@madeingotland.com`

The AUR templates and publish scripts live under [`packaging/aur`](/home/notes/Projects/ghfetch/packaging/aur) and [`scripts`](/home/notes/Projects/ghfetch/scripts).

## License

MIT. See [LICENSE](LICENSE).
