# Repository Agent Instructions

## Scope
These instructions apply to the entire repository unless a more specific
`AGENTS.md` overrides them within a subdirectory.

## Coding Guidelines
- Follow idiomatic Rust style and favor clarity over micro-optimizations.
- Keep functions short and cohesive; refactor shared logic into helper
  functions or modules.
- Prefer explicit types and exhaustive error handling when interfaces cross
  module boundaries.
- Write descriptive commit messages and document noteworthy design decisions in
  code comments where appropriate.

## Testing
- Add or update automated tests for new or modified behavior.
- Run `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test`
  before submitting changes.

## Documentation
- Update README or inline documentation when behavior or usage changes.
- Include references to any relevant standards or resources when helpful.

## Collaboration
- Prefer small, reviewable pull requests.
- Link related issues in PR descriptions and templates when applicable.
