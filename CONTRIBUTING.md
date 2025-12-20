# Contributing to Steward

Thanks for your interest in contributing to Steward! This document outlines how to get started.

## Getting Started

1. Fork the repository
2. Clone your fork locally
3. Install Rust (1.75+ recommended): https://rustup.rs
4. Build the project: `cargo build`
5. Run tests: `cargo test`

## Development Workflow

```bash
# Build all crates
cargo build

# Run tests
cargo test

# Run with clippy lints
cargo clippy --all-targets -- -D warnings

# Format code
cargo fmt

# Run the CLI
cargo run -p steward-cli -- evaluate --contract contracts/general.yaml --output output.txt
```

## How to Contribute

### Reporting Bugs

- Check existing issues first to avoid duplicates
- Use a clear, descriptive title
- Include steps to reproduce the issue
- Describe expected vs actual behavior
- Include relevant contract and output samples if applicable

### Suggesting Features

- Open an issue describing the feature
- Explain the use case and why it would be valuable
- Reference the blueprint (`docs/steward-blueprint-specs.md`) if relevant
- Be open to discussion about implementation approaches

### Pull Requests

1. Create a branch from `main` for your changes
2. Make your changes with clear, focused commits
3. Ensure all tests pass (`cargo test`)
4. Ensure code is formatted (`cargo fmt`)
5. Ensure clippy passes (`cargo clippy`)
6. Open a PR with a clear description of changes
7. Link any related issues

## Code Style

- Run `cargo fmt` before committing
- Follow existing patterns in the codebase
- Use meaningful variable and function names
- Add doc comments to public APIs
- Include unit tests for new functionality

## Architecture Guidelines

### Critical Boundaries

- **steward-core must never make LLM calls** — this is deterministic evaluation only
- **Lenses must stay independent** — no lens may access another lens's findings
- **Evidence is required** — every BLOCKED state must cite rule_id and evidence

### Testing Requirements

- **Golden tests**: Exact JSON output matching for contract + output pairs
- **Property tests**: Invariants (determinism, BLOCKED dominance, confidence bounds)
- **No substring matching**: Tests assert exact structure, not "contains"

## Areas for Contribution

- **Lenses**: Implement remaining lenses (Dignity, Restraint, Transparency, Accountability)
- **Contract domains**: Add domain-specific contracts (healthcare, finance, legal)
- **Bindings**: Python (PyO3) and TypeScript (napi-rs) bindings
- **Documentation**: Examples, tutorials, contract authoring guides
- **Testing**: Golden tests, property tests, edge cases

## Questions?

Open an issue or reach out to the maintainers. We're happy to help!
