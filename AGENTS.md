# AGENTS.md

Guidance for agents working on Esoterica, a Rust conlang-generation library with CLI/TUI, optional WASM bindings, a prototype web UI, and AT Protocol publication helpers.

## Actual surfaces

- Core modules are always compiled: archetype registries, phonology, morphology, lexicon, sound changes, syntax, semantic drift, orthography, and the in-memory collaborative model.
- Default feature `cli` adds Clap, Tokio, Ratatui, and AT Protocol dependencies plus the binary. `wasm` is separate and intended for `--no-default-features --features wasm`.
- `data/*.toml` is embedded at compile time with `include_str!`; registry parse errors panic at startup/build use. It currently contains eight phonologies, four morphologies, three syntax presets, and eleven sound-change keys including `none`.
- CLI mode uses only the first phonology and morphology values, silently ignores unknown sound-change keys, requires a syntax preset key, generates random data, optionally writes JSON/orthography, and publishes only when title and both credential variables are present.
- The TUI generates a 100-entry lexicon and saves to fixed `lexicon_output.json`; it does not use most incoming CLI arguments.
- `web/` declares a file dependency on wasm-pack output at `pkg/`, but its three Svelte tools are currently JavaScript mock/stub implementations and never import the WASM package. The lexicon tab generates random placeholder entries; the rule editor ignores formal contexts; the phonology preview is text only.

## Current limitations and correctness rules

- Generation is not deterministic. Engines construct `thread_rng()` internally, and `DriftConfig.seed` is never read. Do not claim seeded reproducibility until RNG state is injected end-to-end and fixtures cover map iteration/order.
- Validate non-empty inventories and phonotactic templates before generation. Phonology uses `choose(...).unwrap()`; empty vowel/consonant sets can panic. Morphological infix insertion uses a byte midpoint and can panic on non-ASCII roots.
- Lexicon entries are stored by final headword in a `HashMap`; collisions overwrite earlier entries, so requested size is not guaranteed. Definitions/citations are generated fictional boilerplate, not researched linguistic data.
- Formal sound-change parsing supports a small literal/V/C/# model. `from_formal_rules` silently drops invalid rules, while the web mock applies regex replacements without escaping input and ignores `/ context` semantics.
- Syntax supports all six orders in code but only SVO/SOV/VSO presets in tracked TOML. Unknown `word_order` values silently fall back to SVO. Case configuration uses the first two strings as subject/object suffixes rather than linguistic case semantics.
- “Collaborative” code is an in-memory change log/merge helper only. It has no AT Protocol transport, signature verification, persistence, stream watcher, or cryptographic signing despite README/module comments.
- AT Protocol publishing creates raw `site.standard.publication`/`document` JSON with server validation unset. Dictionary publication embeds the entire JSON twice, supplies only a publication URI rather than a verified strong reference/CID, does not paginate publication listing past 50, and has no size/blob handling. Validate against current Standard.site lexicons before live writes.
- If publication flags are supplied without both environment credentials, the CLI silently skips publication. Never log or commit app passwords/session material, and use a dedicated account for protocol testing.
- `repos.txt` is unrelated repository inventory and includes forks; it is not a runtime input or ownership authority.

## Validation

Run `cargo fmt --check`, `cargo clippy --all-targets --all-features`, `cargo test --all-features`, `cargo build --release`, and `cargo build --no-default-features --features wasm`. For web changes, produce `pkg/` with `wasm-pack build --no-default-features --features wasm`, then install/build under `web/`; there is no tracked web lockfile or test suite. Add tests for empty/Unicode inventories, infix boundaries, headword collisions, invalid formal rules, drift-rate bounds and real seeds, CLI unknown keys, and serialization compatibility. Treat AT Protocol publication as a live write and the web UI as a prototype until it actually calls the WASM exports.
