# AGENTS.md

Guidance for agents working on Esoterica, a Rust conlang-generation library with CLI/TUI, optional WASM bindings, a WASM-backed web UI, and AT Protocol publication helpers.

## Actual surfaces

- Core modules are always compiled: archetype registries, seedable RNG, phonology, IPA transcription, morphology, lexicon, sound changes, syntax, semantic drift, orthography, and the in-memory collaborative model.
- Default feature `cli` adds Clap, Tokio, Ratatui, and AT Protocol dependencies plus the binary. `wasm` is separate and intended for `--no-default-features --features wasm`.
- `data/*.toml` is embedded at compile time with `include_str!`. It currently contains sixteen phonologies, ten morphologies, ten syntax presets covering all six word orders, and nineteen sound-change keys including `none`. `Registries::load` returns a `Result`; the older `get_*_registry` helpers still panic on a malformed file and are kept for compatibility.
- CLI mode uses only the first phonology and morphology values, errors on unknown archetype keys (listing the valid ones), requires a syntax preset key, and supports `--seed`, `--list`, `--preview`, `--sentences`, `--formal-rule`, `--script-type`, and `--glyph-style`. Publication requires both credential variables and fails loudly if they are missing.
- The TUI honours `--seed`, `--lexicon-size`, `--syllables`, and `--output` from the CLI arguments; other flags are still ignored.
- `web/` runs the real engine. `web/src/lib/engine.js` is the only module that touches WASM: it initialises the `--target web` build once, caches the archetype catalogues, and exposes a JSON-marshalling facade. Components receive the facade as an `engine` prop from `App.svelte`, which renders them only after loading resolves; catalogues come from the module-level `presets()` rather than a prop, so they stay out of Svelte's reactivity graph. Four tools: phonology designer, lexicon browser, sound-change editor with derivation traces, and an orthography studio that renders the generated SVG glyphs.

## Invariants to preserve

- **Determinism.** Every engine takes a seed (`PhonologyEngine::seeded`, `MorphologyEngine::seeded`, `LexiconGenerator::seeded`, `OrthographyEngine::seeded`, `DriftConfig::seed`) and holds a `SharedRng` from `src/rng.rs`. A seeded run must be byte-reproducible end to end. Two things make that hold and must not be undone: `Lexicon` and the orthography map are `BTreeMap`s so serialisation is key-ordered, and `SemanticDriftEngine::apply_to_lexicon` walks `sorted_headwords()` rather than map order. `tests/pipeline.rs` pins this; do not weaken those tests.
- **No nested RNG borrows.** `SharedRng::with` panics if re-entered. Helper methods take `&mut StdRng` instead of reaching back into `self.rng`.
- **Character boundaries.** Infixing, partial reduplication, and vowel-harmony segmentation all operate on `chars`, never byte offsets. Every phonology with `ä`, `ŋ`, or IPA symbols exercises this.
- **Validation before generation.** `Phonology::validate` rejects inventories that cannot satisfy their own template, empty phoneme strings, and tone counts above five. `try_new`/`try_seeded`/`LexiconGenerator::try_new` go through it; the plain `new` constructors stay permissive and degrade instead of panicking.
- **Headword uniqueness.** `generate_core_lexicon` retries collisions rather than overwriting, bounded by `ATTEMPTS_PER_ENTRY`, so a small inventory stops early instead of looping. Callers must check `Lexicon::len` rather than assuming the requested size.
- **Loud failure over silent skip.** Unknown registry keys, unparseable formal rules, unrecognised word orders (`SyntaxEngine::try_new`), and missing publication credentials are all errors. `SoundChangeEngine::from_formal_rules` and `SyntaxEngine::new` keep the old lenient behaviour; prefer the `try_` variants in new code.

## Current limitations

- Definitions and citations are generated fictional boilerplate from a fixed template bank, not researched linguistic data. Treat them as flavour text.
- Formal sound-change parsing supports a small literal/V/C/# model with a single segment on each side. `is_vowel` is a hardcoded character set, so it does not consult the phonology's actual inventory. There is no feature geometry, no `[+voice]` notation, and no rule ordering beyond declaration order.
- Vowel harmony is pair-based (a/ä, o/ö, u/y and relatives) with everything else neutral. It does not model rounding harmony, height harmony, or disharmonic roots.
- `Syntax.cases` still uses the first two strings as subject/object suffixes rather than a real case paradigm; `adposition` and `adjective_order` are typological metadata used by `describe`, `generate_noun_phrase`, and `generate_adpositional_phrase`, not by clause generation.
- IPA transcription in `src/ipa.rs` is a romanisation convention, matched longest-first. It assumes the orthography used by `data/phonologies.toml` and passes unknown characters through.
- "Collaborative" code is an in-memory change log/merge helper only. It has no AT Protocol transport, signature verification, persistence, stream watcher, or cryptographic signing despite README/module comments.
- AT Protocol publishing creates raw `site.standard.publication`/`document` JSON with server validation unset. Dictionary publication embeds the entire JSON twice, supplies only a publication URI rather than a verified strong reference/CID, does not paginate publication listing past 50, and has no size/blob handling. Validate against current Standard.site lexicons before live writes.
- Never log or commit app passwords/session material, and use a dedicated account for protocol testing.
- `repos.txt` is unrelated repository inventory and includes forks; it is not a runtime input or ownership authority.

## Validation

Run `cargo clippy --all-targets --all-features`, `cargo test --all-features`, `cargo build --release`, and `cargo build --no-default-features --features wasm`.

The tree is **not** rustfmt-clean and never has been — several modules use a deliberately dense hand-formatted style. Do not run `cargo fmt` across the repository; it would rewrite thousands of unrelated lines. Match the surrounding style in whatever file you are editing.

For web changes, produce `pkg/` with `wasm-pack build --target web --no-default-features --features wasm` (or `npm run wasm` from `web/`), then `npm install && npm run build` under `web/`. The build must stay warning-free; `state_referenced_locally` in particular means a component is snapshotting a prop and should read from `presets()` instead.

`--target web` matters: the bundler target would need a Vite WASM plugin, whereas the web target initialises through the default export that `engine.js` already awaits. `Cargo.toml` sets `wasm-opt = false` because wasm-pack fetches a binaryen release at build time, which fails offline; run `wasm-opt -Oz` separately when shipping.

There is no automated web test suite. Verify UI changes in a browser — the repo image ships Chromium at `/opt/pw-browsers/chromium` with `PLAYWRIGHT_BROWSERS_PATH` already set, so a short Playwright script against `vite preview` is the fastest check.

`tests/pipeline.rs` holds the end-to-end tests: seeded reproducibility, every phonology × morphology pairing, every sound-change set, every syntax preset, and JSON round-trips. Unit tests live beside each module. When adding a feature, add coverage for empty/Unicode inventories, boundary conditions, invalid input, and — if it touches generation — a same-seed reproducibility assertion.
