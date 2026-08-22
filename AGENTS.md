# Repository Guidelines

## Project Structure & Module Organization

This Rust 2021 crate implements abstract domains for 64-bit integers. Public modules live in `src/`: `tnum.rs`, `znum.rs`, and `rnum.rs` contain the core numeric domains; `wrapped.rs`, `range_set.rs`, and `bit_range.rs` provide interval and reduced-product experiments. `src/lib.rs` declares and re-exports the public API. Integration and property tests are under `tests/`, generally named after the module they exercise. `examples/compare.rs` demonstrates precision differences between domains.

## Build, Test, and Development Commands

- `cargo build` compiles the library in debug mode.
- `cargo test` runs all unit, integration, and Proptest-based property tests.
- `cargo run --example compare` runs the comparison example.
- `cargo fmt --all -- --check` verifies standard Rust formatting; use `cargo fmt --all` to apply it.
- `cargo clippy --all-targets --all-features -- -D warnings` checks idioms and treats warnings as failures.
- `cargo doc --no-deps` builds local API documentation.

Run formatting, Clippy, and the full test suite before submitting changes.

## Coding Style & Naming Conventions

Follow `rustfmt` defaults (four-space indentation) and conventional Rust naming: `snake_case` for functions and modules, `CamelCase` for types, and `SCREAMING_SNAKE_CASE` for constants. Prefer explicit wrapping operations for machine-integer semantics. Preserve each domain's canonical representation and document public methods with `///`, especially when behavior differs from ordinary integer arithmetic. Keep module re-exports in `src/lib.rs` synchronized with public additions.

For specific implementations of operations or types, link to references that informed their structure in comments.

## Testing Guidelines

Place public-behavior tests in the matching `tests/<module>.rs` file. Name tests after the property being asserted, such as `shifts_use_modulo_64_counts`. Use Proptest for algebraic laws and containment/soundness over broad inputs; retain generated regression seeds such as `tests/tnum.proptest-regressions`. Add focused examples for boundary values including `0`, `u64::MAX`, wrapping arithmetic, empty intersections, and signed-bound transitions. No coverage threshold is configured, so prioritize semantic edge cases.

## Commit & Pull Request Guidelines

Recent commits use short, lowercase, imperative summaries such as `add checked_div` and `add range/tnum intersect` and `fix clippy`. Keep each commit narrowly scoped and explain non-obvious design choices in the body. Pull requests should describe the affected domain, its soundness or precision impact, and the commands run. Link relevant issues and include before/after example output when observable behavior changes; screenshots are generally unnecessary for this library.
