# GitHub Copilot Configuration

These instructions help Copilot generate consistent contributions across the
project:

1. Prefer idiomatic Rust with descriptive naming, explicit types, and
   comprehensive error handling.
2. Keep functions small and focused; extract helpers when logic grows beyond a
   few branches or side effects.
3. Adhere to the guidelines in the repository `AGENTS.md` files for formatting,
   documentation, and verification expectations.
4. Include unit or integration tests alongside new behavior and update existing
   coverage when altering functionality.
5. Ensure `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test`
   pass locally before suggesting completion of a change.
6. Avoid introducing new runtime dependencies without prior discussion; prefer
   standard library solutions when practical.
