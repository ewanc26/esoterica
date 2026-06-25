# Esoterica Conlang Generator

Esoterica is a modular, high-performance Rust framework for generating constructed languages (conlangs). It allows users to create consistent languages by composing independent linguistic components: phonology, morphology, syntax, sound changes, semantic drift, and orthography.

## Features

- **Component-based Generation**: Dynamically mix-and-match phonological, morphological, syntactic, and sound-change profiles.
- **Phonology Engine**: Generates syllables from phonotactic patterns (`C(C)V(C)`) with tone and vowel harmony support.
- **Morphology Engine**: Suffix, prefix, infix insertion and reduplication rules.
- **Sound Change Engine**: Legacy TOML-based rules + formal nom-based parser supporting notation like `p > b / V_V` and `k > h / _#`.
- **Semantic Drift**: Probabilistic simulation of meaning change (broadening, narrowing, amelioration, pejoration, metaphor, metonymy, taboo replacement).
- **Orthography Generator**: Procedural script generation for alphabets, abjads, abugidas, syllabaries, and logographies with configurable glyph styles.
- **Collaborative Editing**: ATProto-based collaborative conlanging with merge conflict detection.
- **CLI Interface**: Full command-line generation with semantic drift and orthography export.
- **Interactive TUI**: Ratatui-based interface with phonology designer (IPA grid), config selector, and real-time generation.
- **ATProto Publication**: Publish lexicons to the ATProto network.
- **WASM Support**: Compile core engine to WebAssembly for web use.
- **Svelte 5 Web Interface**: Browser-based phonology designer, lexicon browser, and sound change editor.

## Quick Start

```bash
# Build
cargo build --release

# Interactive TUI
cargo run --release -- --interactive

# CLI generation
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

## Project Structure

```
esoterica/
├── src/
│   ├── lib.rs              # Library root
│   ├── main.rs             # CLI binary
│   ├── wasm.rs             # WASM bindings
│   ├── archetypes.rs       # Linguistic component types + registry
│   ├── phonology.rs        # Syllable/word generation engine
│   ├── morphology.rs       # Morphological transformation engine
│   ├── syntax.rs           # Word order management (6 orders)
│   ├── sound_change.rs     # Nom-based formal rule parser + legacy engine
│   ├── lexicon.rs          # Dictionary generation
│   ├── lexicon_structs.rs  # Data structures
│   ├── semantic_drift.rs   # Probabilistic meaning change simulation
│   ├── orthography.rs      # Procedural script/glyph generator
│   ├── collaborative.rs    # ATProto collaborative editing session
│   ├── atproto.rs          # ATProto publishing logic
│   └── tui/                # Ratatui interactive interface
│       ├── mod.rs, app.rs, ui.rs, event.rs, components.rs
│       └── phonology_designer.rs  # IPA grid phonology designer
├── data/                   # TOML configuration files
│   ├── phonologies.toml    # 8 language phoneme inventories
│   ├── morphologies.toml   # 4 morphological type definitions
│   ├── syntaxes.toml       # 3 word-order configurations
│   └── sound_changes.toml  # 10 diachronic rule sets
├── web/                    # Svelte 5 web interface
│   └── src/lib/            # PhonologyDesigner, LexiconBrowser, SoundChangeEditor
└── Cargo.toml              # Feature-gated (cli, wasm)
```

## ATProto Publication

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

## License

MIT License. See [LICENSE](LICENSE) file for details.
