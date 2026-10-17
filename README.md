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

"Most families" is defined concretely: `major_families_are_represented` in
`src/archetypes.rs` requires at least one inventory for each of 39 major families, isolates
and creoles. `coverage_checklist_is_present` names the wider set of 82 inventories. Those inventories cover major and well-documented families and branches:
Germanic, Romance, Slavic, Indo-Aryan, Celtic, Iranian, Hellenic, Baltic, Armenian and
Albanian (Indo-European); Finnic, Ugric and Samoyedic (Uralic); Turkic; Mongolic; Tungusic;
Chukotko-Kamchatkan; Basque, Ainu, Yeniseian, Nivkh and Burushaski (isolates); Kartvelian;
Northwest Caucasian and Nakh-Dagestanian; Semitic, Berber, Cushitic, Chadic and Omotic
(Afro-Asiatic); Bantu, Yoruboid, Mande, Atlantic, Gur, Kwa and Ubangian (Niger-Congo);
Nilotic, Saharan and Songhay (Nilo-Saharan); Khoe and Tuu (Khoisan); Sinitic, Burmese and
Tibetic (Sino-Tibetan); Hmong-Mien; Viet-Muong; Tai (Tai-Kadai); Mon-Khmer (Austroasiatic);
Japonic; Koreanic; Dravidian; Austronesian (Polynesian, Malayo-Polynesian, Philippine,
Malagasy); Trans-New Guinea and Sepik (Papuan); Pama-Nyungan and Gunwinyguan (Australian);
Uto-Aztecan, Quechuan, Algic, Salishan, Eskaleut, Mayan, Arawakan, Athabaskan,
Oto-Manguean, Tupian, Siouan, Iroquoian, Chibchan, Pano-Tacanan, Jê, Aymaran and
Tucanoan (Americas); Creole: Tok Pisin and Haitian; Constructed: Toki Pona and Esperanto.

**Not covered:** sign languages (the model generates spoken segments only); most Papuan
families beyond Trans-New Guinea and Sepik; most Australian families beyond Pama-Nyungan
and Gunwinyguan; most small Americas and Caucasus families; other Hmong-Mien and Chadic
branches; and most other creoles. Word orders cover all six, with ergative and split-ergative
variants. Morphology covers agreement and classifier profiles as well as concatenative,
root-and-pattern and reduplicative ones.

The WASM bindings expose `generate_from_catalogue`, which resolves catalogue keys through
the same pipeline as the CLI. Its browser-side behaviour has not been run in a browser.

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
