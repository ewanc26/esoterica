# AGENTS.md

Guidance for agents working on Esoterica, a modular Rust constructed-language generator with CLI/TUI, library, WASM, web UI, and AT Protocol publishing surfaces.

## Boundaries

- Core generation and data models must compile without the default CLI feature.
- CLI/TUI dependencies are feature-gated; WASM uses `wasm` with default features disabled.
- `data/` is schema-like generation input; preserve compatibility and deterministic seeded output.
- `web/` consumes the WASM/library surface rather than duplicating generation logic.
- AT Protocol publication must serialize stable, valid records and keep credentials outside generated content.

## Rules

- Keep random number generation injectable/seeded for reproducible tests.
- Validate grammars/data with actionable context; avoid panics on user-authored inputs.
- Do not pull filesystem, Tokio, terminal, or network assumptions into the core library.
- Maintain Unicode correctness throughout phonology, orthography, serialization, and display.

## Validation

Run `cargo fmt --check`, `cargo clippy --all-targets --all-features`, `cargo test --all-features`, `cargo build --release`, and `cargo build --no-default-features --features wasm`. Build the WASM/web surface when touched. Test seeded reproducibility, malformed data, empty inventories, Unicode output, CLI and TUI exits, and publication serialization.
