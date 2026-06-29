# Esoterica Conlang Generator

Generates constructed languages from composable components: phonology, morphology, syntax, sound changes, semantic drift, and orthography. Written in Rust.

## Features

- **Phonology** — Syllable generation from phonotactic patterns with tone and vowel harmony
- **Morphology** — Affix insertion and reduplication rules
- **Sound changes** — TOML-based rules + formal parser (`p > b / V_V`)
- **Semantic drift** — Probabilistic meaning change (broadening, narrowing, amelioration, pejoration, metaphor, etc.)
- **Orthography** — Procedural script generation (alphabets, abjads, abugidas, syllabaries, logographies)
- **Collaborative** — ATProto-based collaborative editing with merge conflict detection
- **CLI + TUI** — Ratatui interface with phonology designer, config selector, real-time generation
- **WASM** — Core engine compiles to WebAssembly
- **Web UI** — Svelte 5 frontend with phonology designer, lexicon browser, sound change editor
- **ATProto** — Publish lexicons to the AT Protocol

## Quick start

```bash
cargo build --release

# Interactive TUI
cargo run --release -- --interactive

# CLI
cargo run --release -- \
  --phonology uralic_finnic \
  --morphology agglutinative \
  --syntax sov \
  --sound-change lenition,rhotacism \
  --syllables 3 \
  --drift-steps 3 \
  --generate-orthography \
  --output my_language.json

# WASM
wasm-pack build --features wasm --no-default-features

# Web UI
cd web && npm install && npm run dev
```

## Project layout

```
esoterica/
├── src/
│   ├── lib.rs              # Library root
│   ├── main.rs             # CLI binary
│   ├── wasm.rs             # WASM bindings
│   ├── archetypes.rs       # Component types + registry
│   ├── phonology.rs        # Syllable/word generation
│   ├── morphology.rs       # Morphological transformations
│   ├── syntax.rs           # Word order (6 orders)
│   ├── sound_change.rs     # Rule parser + legacy engine
│   ├── lexicon.rs          # Dictionary generation
│   ├── lexicon_structs.rs  # Data structures
│   ├── semantic_drift.rs   # Probabilistic meaning change
│   ├── orthography.rs      # Script/glyph generator
│   ├── collaborative.rs    # ATProto collaborative editing
│   ├── atproto.rs          # ATProto publishing
│   └── tui/                # Ratatui interface
├── data/
│   ├── phonologies.toml    # 8 phoneme inventories
│   ├── morphologies.toml   # 4 morphology definitions
│   ├── syntaxes.toml       # 3 word-order configs
│   └── sound_changes.toml  # 10 diachronic rule sets
├── web/                    # Svelte 5 web interface
└── Cargo.toml
```

## ATProto publication

```bash
export ATPROTO_HANDLE="your-handle.bsky.social"
export ATPROTO_PASSWORD="your-app-password"

cargo run --release -- \
  --phonology uralic_finnic \
  --morphology agglutinative \
  --syntax svo \
  --publish-title "My Conlang" \
  --publication-uri "at://did:plc:.../site.standard.publication/..."
```

## Licence

MIT
