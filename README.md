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

Reference counts, from Glottolog (as of March 2026): 246 genealogical families and 183
isolates ([Glottolog: About Languoids](https://glottolog.org/glottolog/glottologinformation)).
Glottolog also lists a few hundred languages in each of its largest families, such as
Atlantic–Congo (about 1,410 languages) and Austronesian (about 1,274)
([Brilliant Maps, Glottolog 5.0 table](https://brilliantmaps.com/human-language-families/)).

This registry has 97 stylised inventories. Each covers one family, branch or isolate, so
the largest families by language count each have at least one entry: Atlantic–Congo
(Bantu, Yoruboid, Mande, Atlantic, Gur, Kwa, Ubangian), Austronesian (Polynesian,
Malayo-Polynesian, Philippine, Malagasy), Indo-European, Sino-Tibetan, Afro-Asiatic and
Pama–Nyungan. An inventory represents a family's typical segments, not its internal
variety.

Groups covered include Indo-European (Germanic, Romance, Slavic, Indo-Aryan, Celtic,
Iranian, Hellenic, Baltic, Armenian, Albanian, Italic); Uralic (Finnic, Ugric, Samoyedic);
Turkic; Mongolic; Tungusic; Chukotko-Kamchatkan; Kartvelian; Northwest and Nakh-Dagestanian
Caucasian; Semitic, Berber, Cushitic, Chadic and Omotic (Afro-Asiatic); Bantu, Yoruboid,
Mande, Atlantic, Gur, Kwa and Ubangian (Niger-Congo); Nilotic, Saharan and Songhay
(Nilo-Saharan); Khoe, Tuu, Hadza and Sandawe (Khoisan); Sinitic, Burmese and Tibetic
(Sino-Tibetan); Hmong-Mien; Viet-Muong; Tai (Tai-Kadai); Mon-Khmer and Munda (Austroasiatic);
Japonic; Koreanic; Dravidian; Austronesian (Polynesian, Malayo-Polynesian, Philippine,
Malagasy); Trans-New Guinea, Sepik and Angan (Papuan); Pama–Nyungan, Gunwinyguan and
Nyulnyulan (Australian); Uto-Aztecan, Quechuan, Algic, Salishan, Eskaleut, Mayan, Arawakan,
Athabaskan, Oto-Manguean, Tupian, Siouan, Iroquoian, Chibchan, Pano-Tacanan, Jê, Aymaran,
Tucanoan, Wakashan, Muskogean, Caddoan, Yuman, Mixe-Zoque, Totonacan and Mura (Americas);
the isolates Basque, Ainu, Yeniseian, Nivkh, Burushaski, Kusunda and Mapuche; plus two
creoles (Tok Pisin, Haitian) and two constructed languages (Toki Pona, Esperanto).

**Not covered:**
- **Sign languages.** The model generates spoken segments only.
- **Most of the remaining families.** Of Glottolog's 246 families, most are small, and many
  are in Papua, Australia and the Americas. Only one or two groups are represented in
  each of those regions.
- **Papuan.** Only Trans-New Guinea, Sepik and Angan. Papuan spans many small families
  ([Wikipedia: Papuan languages](https://en.wikipedia.org/wiki/Papuan_languages)).
- **Australian.** Only Pama–Nyungan, Gunwinyguan and Nyulnyulan.
- **Caucasian and Americas.** Many branches have no inventory.
- **Other creoles and constructed languages.**

Word orders cover all six, with ergative and split-ergative variants. Morphology covers
agreement and classifier profiles as well as concatenative, root-and-pattern and
reduplicative ones. The `coverage_checklist_is_present` test names every inventory above,
so removing one fails the build.

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
