# Contributing to esoterica

Esoterica generates constructed languages from composable components — phonology, morphology, syntax, sound changes, semantic drift and orthography — with a CLI, a ratatui TUI, a WASM build and a Svelte 5 web UI, all around one Rust core. Thank you for wanting to help!

## Getting started

```bash
cargo build --release
cargo run --release -- --interactive   # TUI
cargo test --lib                       # validates every data entry
```

The web UI: `cd web && npm install && npm run dev`. The WASM core: `wasm-pack build --features wasm --no-default-features`.

## Notes for this project

- **The data registries are compiled in** with `include_str!`: a change to `data/*.toml` needs a rebuild to take effect.
- `cargo test --lib` is the gate for data changes: every entry must parse, inventories must be non-empty, syllable templates only `C`/`V` with optional `()` groups, sound-change rules must parse, and every phonology is run against every morphology and sound-change set (`coverage_checklist_is_present` in `src/archetypes.rs`). New families and typological types need a checklist entry.
- Inventories are *stylised design aids*, not authoritative descriptions of the languages they are named after — keep that framing in new data and docs.
- Three surfaces (CLI, TUI, web) share one engine: a change to generation semantics should be reflected in all the surfaces that expose it.
- Sound-change notation (`p > b / V_V`) is a formal mini-language; extend it deliberately and document the grammar of any new rule form.

## Before opening a PR

- The documented build and tests pass, and you ran them yourself.
- Conventional commits; one focused PR per change.
- AI-assisted PRs are welcome; stand behind every line.

## Licence

AGPL-3.0 (see [LICENSE](LICENSE)). Contributions are accepted under the same terms.