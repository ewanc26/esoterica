# Esoterica Conlang Generator

Generates constructed languages from composable components: phonology, morphology, syntax, sound changes, semantic drift, and orthography. Written in Rust.

## Features

- **Reproducible** — `--seed` makes the whole pipeline deterministic; the same seed and configuration produce a byte-identical lexicon and script
- **Phonology** — Syllable generation from phonotactic patterns, with tone and pair-based vowel harmony (a/ä, o/ö, u/y) that respects neutral vowels and never invents phonemes the language lacks
- **Morphology** — Prefixes, suffixes, character-safe infixes, circumfixes, and full or partial reduplication
- **Sound changes** — TOML rule sets + a formal parser (`p > b / V_V`), with step-by-step derivation traces
- **Semantic drift** — Probabilistic meaning change (broadening, narrowing, amelioration, pejoration, metaphor, metonymy, taboo replacement)
- **Orthography** — Procedural script generation (alphabets, abjads, abugidas, syllabaries, logographies) in four glyph styles
- **IPA** — Longest-match transcription that handles multigraphs, front-rounded vowels, and tone letters
- **CLI + TUI** — Ratatui interface with phonology designer, config selector, real-time generation
- **WASM** — Core engine compiles to WebAssembly, with seeds exposed to JavaScript
- **AT Protocol** — Publish lexicons to the AT Protocol

## Quick start

```bash
cargo build --release

# See every available archetype key
cargo run --release -- --list

# Interactive TUI
cargo run --release -- --interactive

# CLI
cargo run --release -- \
  --phonology uralic_finnic \
  --morphology agglutinative \
  --syntax sov_ergative \
  --sound-change lenition,rhotacism \
  --formal-rule 'k > h / _#' \
  --seed 42 \
  --syllables 3 \
  --lexicon-size 500 \
  --drift-steps 3 \
  --generate-orthography --script-type syllabary \
  --preview 10 \
  --output my_language.json

# WASM
wasm-pack build --features wasm --no-default-features

# Web UI
cd web && npm install && npm run dev
```

## Reproducibility

Without `--seed`, every run draws from system entropy and produces a different
language. With `--seed`, every stage — root generation, noun-class assignment,
semantic drift, glyph shapes — is derived from that one number:

```bash
cargo run --release -- -p uralic_finnic -m agglutinative -x sov --seed 42 -o a.json
cargo run --release -- -p uralic_finnic -m agglutinative -x sov --seed 42 -o b.json
cmp a.json b.json   # identical
```

Lexicons and orthographies serialise in sorted key order, so the JSON is stable
across runs and processes and diffs between two seeds are meaningful.

## CLI options

| Flag | Purpose |
| --- | --- |
| `-p, --phonology` | Phoneme inventory key (required) |
| `-m, --morphology` | Morphological profile key (required) |
| `-x, --syntax` | Syntax preset key (required) |
| `-c, --sound-change` | Comma-separated rule-set keys, applied in order |
| `--formal-rule` | Ad-hoc rule in formal notation; repeatable |
| `-s, --seed` | Seed for reproducible generation |
| `-n, --lexicon-size` | Number of entries (default 100) |
| `-y, --syllables` | Syllables per root (default 2) |
| `--drift-steps` / `--drift-rate` | Semantic drift depth and per-word probability |
| `--generate-orthography` | Also generate a script |
| `--script-type` / `--glyph-style` | Writing system and glyph aesthetic |
| `-l, --list` | Print every available archetype key and exit |
| `--preview [N]` | Print the first N entries (default 10) |
| `--sentences N` | Example sentences built from real lexicon entries |
| `-o, --output` | Lexicon output path |

Unknown archetype keys are an error, not a silent skip: a typo in
`--sound-change` fails with the list of valid keys rather than quietly
generating a language that skipped a whole sound change.

## Project layout

```
esoterica/
├── src/
│   ├── lib.rs              # Library root
│   ├── main.rs             # CLI binary
│   ├── wasm.rs             # WASM bindings
│   ├── rng.rs              # Seedable RNG shared by every engine
│   ├── archetypes.rs       # Component types + registries
│   ├── phonology.rs        # Syllable/word generation, validation, harmony
│   ├── ipa.rs              # Orthography → IPA transcription
│   ├── morphology.rs       # Morphological transformations
│   ├── syntax.rs           # Word order (6 orders) + head direction
│   ├── sound_change.rs     # Rule parser, legacy engine, derivation traces
│   ├── lexicon.rs          # Dictionary generation
│   ├── lexicon_structs.rs  # Data structures
│   ├── semantic_drift.rs   # Probabilistic meaning change
│   ├── orthography.rs      # Script/glyph generator
│   ├── collaborative.rs    # In-memory collaborative change log
│   ├── atproto.rs          # AT Protocol publishing
│   └── tui/                # Ratatui interface
├── data/
│   ├── phonologies.toml    # 16 phoneme inventories
│   ├── morphologies.toml   # 10 morphology definitions
│   ├── syntaxes.toml       # 10 word-order configs, all six orders
│   └── sound_changes.toml  # 19 diachronic rule sets
├── tests/pipeline.rs       # End-to-end and reproducibility tests
├── web/                    # Svelte 5 web interface
└── Cargo.toml
```

## Writing rules

Sound changes come in two notations. TOML rule sets in
`data/sound_changes.toml` are plain substring rewrites with an optional
`word_initial` / `word_final` context:

```toml
final_devoicing = [
    { pattern = "b", replacement = "p", context = "word_final" },
]
```

Formal notation, passed with `--formal-rule` or through the library, supports
segment classes and environments:

```
p > b / V_V     # intervocalic voicing
k > h / _#      # word-final spirantisation
s > ∅ / #_      # word-initial deletion
```

`V` matches any vowel, `C` any consonant, `#` a word boundary, and `∅` marks
deletion. Rules that fail to parse are reported rather than skipped.

## Testing

```bash
cargo test --all-features
cargo clippy --all-targets --all-features
cargo build --no-default-features --features wasm
```

## AT Protocol publication

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

Publication is a live write. If `--publish-title` is given without both
credentials the CLI fails rather than silently skipping the upload.

## Licence

MIT
