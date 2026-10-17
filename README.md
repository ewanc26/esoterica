# Esoterica Conlang Generator

Generates constructed languages from composable components: phonology, morphology, syntax, sound changes, semantic drift, and orthography. Written in Rust.

## Features

- **Phonology** — Syllable generation from phonotactic templates with optional segments, tone (up to nine levels) and vowel harmony, with 64 stylised inventories
- **Morphology** — Affixation, infixation, reduplication, circumfixes, noun classes and non-concatenative root-and-pattern templates
- **Sound changes** — Named rule sets in formal notation (`p > b / V_V`, `n > ∅ / _#`)
- **Semantic drift** — Probabilistic meaning change (broadening, narrowing, amelioration, pejoration, metaphor, etc.)
- **Orthography** — Procedural script generation (alphabets, abjads, abugidas, syllabaries, logographies), selectable with `--script-type` and `--glyph-style`
- **Collaborative** — AT Protocol-based collaborative editing with merge conflict detection
- **CLI + TUI** — Ratatui interface with phonology designer, config selector, real-time generation
- **WASM** — Core engine compiles to WebAssembly
- **Web UI** — Svelte 5 frontend with phonology designer, lexicon browser, sound change editor
- **AT Protocol** — Publish lexicons to the AT Protocol

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
│   ├── collaborative.rs    # AT Protocol collaborative editing
│   ├── atproto.rs          # AT Protocol publishing
│   └── tui/                # Ratatui interface
├── data/
│   ├── phonologies.toml    # phoneme inventories, keyed region_family_branch
│   ├── morphologies.toml   # typological morphology profiles
│   ├── syntaxes.toml       # word orders, case alignment, adposition placement
│   └── sound_changes.toml  # named diachronic rule sets in formal notation
├── web/                    # Svelte 5 web interface
└── Cargo.toml
```

## Interactive TUI

`cargo run --release -- --interactive` opens the terminal interface.

| Key | Action |
| --- | --- |
| Tab / Shift-Tab | Move between Phonology, Morphology, Syntax and Sound Changes |
| Up / Down | Move within a panel |
| Space | Toggle a sound-change set (several can be on) |
| Enter | Generate a lexicon |
| l | Browse the lexicon (Up/Down, PgUp/PgDn, Home/End; l or Esc closes) |
| s | Save the lexicon to `esoterica_out/` (never overwrites) |
| p | Open the phonology designer |
| h | Help |
| q / Ctrl-C | Quit |

In the designer, Tab switches between consonants, vowels and presets. Enter or Space toggles
a phoneme, p and m apply the full and minimal presets, + and - change the syllable template,
t toggles tone and [ and ] set the tone count, h toggles vowel harmony, and y uses the design.
Selecting **(custom designer)** in the Phonology panel generates from the design.

## Adding data

Registries are compiled in with `include_str!`, so a change to `data/` needs a rebuild.
`cargo test --lib` checks every entry: TOML parses, phoneme inventories are non-empty,
syllable templates use only `C`, `V` and optional `( )` groups, word orders are valid,
and every sound-change rule parses. It also runs every phonology against every
morphology and sound-change set, and checks that a named list of families and
typological types is present (`coverage_checklist_is_present` in `src/archetypes.rs`).

Inventories are stylised design aids, not authoritative descriptions of the languages
they are named after.

### Coverage

Reference: Glottolog (as of March 2026) lists 246 genealogical families and 183 isolates
([Glottolog: About Languoids](https://glottolog.org/glottolog/glottologinformation)).
Sign languages are excluded from this project, because the model generates spoken
segments only.

The catalogue has two parts:

- **Curated inventories** (`data/phonologies.toml`, 97 entries). Hand-written, stylised
  inventories for families and branches, including the largest by language count:
  Atlantic–Congo, Austronesian, Indo-European, Sino-Tibetan and Afro-Asiatic.
- **Generated Glottolog representatives** (`data/phonologies_glottolog.toml`, 248 entries).
  One inventory per Glottolog top-level family or isolate that has at least one language in
  [PHOIBLE](https://phoible.org) (3,020 inventories, CC BY-SA 3.0). The script
  `scripts/build_glottolog_phonologies.py` builds them from PHOIBLE and the Glottolog
  repository. Each family uses the member language whose consonant count is the median of
  the family. Syllable templates are a generic default (`CV(C)`), because PHOIBLE does not
  record phonotactics.

**Not covered:**
- **Sign languages**, by design.
- **178 families and isolates** have no PHOIBLE inventory in their subtree, so no inventory
  is generated for them. They are listed by Glottolog code in the header of
  `data/phonologies_glottolog.toml`. Covering them would need other sources, such as
  grammars, which this build doesn't use.
- **Within-family variety.** One inventory represents each family, not its dialects or
  sub-branches.

Word orders cover all six, with ergative and split-ergative variants. Morphology covers
agreement and classifier profiles as well as concatenative, root-and-pattern and
reduplicative ones. Tests check that every curated entry is valid, that every generated
entry is valid, and that sign languages are absent. `coverage_checklist_is_present` names
the curated inventories, so removing one fails the build.

#### Regenerating the generated catalogue

```bash
curl -L -o phoible.csv https://raw.githubusercontent.com/phoible/dev/master/data/phoible.csv
curl -L -o glottolog.zip https://github.com/glottolog/glottolog/archive/refs/heads/master.zip
unzip glottolog.zip
python3 scripts/build_glottolog_phonologies.py --phoible phoible.csv \
  --glottolog glottolog-master --out data/phonologies_glottolog.toml
```

The WASM bindings expose `generate_from_catalogue`, which resolves catalogue keys through
the same pipeline as the CLI. Its browser-side behaviour has been checked with a single call.

To use your own catalogues without rebuilding, put any of `phonologies.toml`,
`morphologies.toml`, `syntaxes.toml` or `sound_changes.toml` in a directory and pass
`--data-dir <dir>`. Entries are layered over the built-ins (same key replaces the
built-in) and validated before generation. Errors name the entry and file.

```bash
cargo run --release -- --data-dir ./my_catalogues -p my_language -m agglutinative -x sov
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

## Support

If you find this project useful, consider supporting its development:

[![Ko-fi](https://img.shields.io/badge/Ko--fi-F16061?style=for-the-badge&logo=ko-fi&logoColor=white)](https://ko-fi.com/ewancroft)
[![GitHub Sponsors](https://img.shields.io/badge/GitHub%20Sponsors-30363D?style=for-the-badge&logo=github&logoColor=white)](https://github.com/sponsors/ewanc26)

## Licence

MIT
