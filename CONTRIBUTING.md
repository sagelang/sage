# Contributing to Sage

Thanks for your interest in contributing to Sage! This document provides guidelines for contributing.

## Getting Started

1. **Fork and clone the repository**
   ```bash
   git clone https://github.com/YOUR_USERNAME/sage.git
   cd sage
   ```

2. **Build the project**
   ```bash
   cargo build
   ```

3. **Run tests**
   ```bash
   cargo test --workspace
   ```

4. **Run clippy**
   ```bash
   cargo clippy --workspace
   ```

## Project Structure

```
sage/
├── crates/
│   ├── sage-types/        # Shared types (Span, Ident, TypeExpr)
│   ├── sage-lexer/        # Tokenizer (logos-based)
│   ├── sage-parser/       # Parser (chumsky-based)
│   ├── sage-checker/      # Type checker and name resolution
│   ├── sage-interpreter/  # Tree-walking interpreter
│   └── sage-cli/          # CLI entry point
├── examples/              # Example .sg programs
└── docs/                  # Documentation and RFCs
```

## Development Workflow

1. Create a branch for your work:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. Make your changes, ensuring:
   - All tests pass: `cargo test --workspace`
   - No clippy warnings: `cargo clippy --workspace`
   - Code is formatted: `cargo fmt`

3. Commit with a clear message describing what and why

4. Push and open a Pull Request

## Code Style

- Follow Rust idioms and best practices
- Use `#[must_use]` for functions that return values that shouldn't be ignored
- Add doc comments for public APIs
- Prefer explicit error types over `unwrap()` in library code

## Adding Tests

- Unit tests go in the same file as the code they test (in a `#[cfg(test)]` module)
- Integration tests can be added to each crate's test suite
- Example programs in `examples/` serve as end-to-end tests

## Reporting Issues

When reporting bugs, please include:
- Sage version (`sage --version`)
- Operating system
- Minimal reproduction case (a small `.sg` file that demonstrates the issue)
- Expected vs actual behavior

## Feature Requests

For new language features, consider:
- Does it fit Sage's design philosophy (agents as first-class citizens)?
- Is there a simpler alternative?
- Can it be implemented without breaking existing code?

Open an issue to discuss before implementing significant changes.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
