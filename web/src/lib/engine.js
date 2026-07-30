// engine.js — the single place the web UI talks to the Rust/WASM core.
//
// The WASM package is built with `wasm-pack build --target web`, which means
// the module must be initialised before any export is callable. `load()` does
// that once and hands back a façade with JSON marshalling and error handling
// already applied, so components never touch raw JSON strings.
//
// Build the package first:
//   wasm-pack build --target web --no-default-features --features wasm

let enginePromise = null;
let catalogues = null;

/**
 * Load and initialise the WASM engine. Safe to call from many components at
 * once — the work happens once and every caller shares the same promise.
 *
 * @returns {Promise<object>} the engine façade
 */
export function load() {
  if (!enginePromise) {
    enginePromise = init();
  }
  return enginePromise;
}

/**
 * The bundled archetype catalogues, read once at load time.
 *
 * These are compile-time constants baked into the WASM binary, so components
 * import them directly rather than reading them off a prop — which also keeps
 * them out of Svelte's reactivity graph, where a never-changing value has no
 * business being.
 *
 * @returns {{phonologies: object, morphologies: object, syntaxes: object, soundChanges: object}}
 */
export function presets() {
  if (!catalogues) {
    throw new Error('presets() called before load() resolved');
  }
  return catalogues;
}

async function init() {
  let wasm;
  try {
    wasm = await import('esoterica');
  } catch (cause) {
    throw new Error(
      'WASM package not found. Build it first:\n' +
        '  wasm-pack build --target web --no-default-features --features wasm',
      { cause },
    );
  }

  // `--target web` exports a default init that fetches the .wasm alongside the JS.
  await wasm.default();
  wasm.init_panic_hook?.();

  const engine = facade(wasm);
  catalogues = {
    phonologies: engine.phonologyPresets(),
    morphologies: engine.morphologyPresets(),
    syntaxes: engine.syntaxPresets(),
    soundChanges: engine.soundChangePresets(),
  };
  return engine;
}

/**
 * Rust returns errors as JS exceptions carrying a plain string. Normalise them
 * into `Error` so callers can rely on `.message`.
 */
function callEngine(fn, args) {
  try {
    return fn(...args);
  } catch (raw) {
    throw raw instanceof Error ? raw : new Error(String(raw));
  }
}

/** Seeds cross the boundary as u64, which wasm-bindgen maps to BigInt. */
function toSeed(seed) {
  if (seed === null || seed === undefined || seed === '') return undefined;
  const n = typeof seed === 'bigint' ? seed : BigInt(Math.trunc(Number(seed)));
  return n < 0n ? -n : n;
}

function facade(wasm) {
  const json = (fn, args) => JSON.parse(callEngine(fn, args));

  return {
    /** Preset catalogues, straight from the embedded TOML registries. */
    phonologyPresets: () => json(wasm.get_phonology_presets, []),
    morphologyPresets: () => json(wasm.get_morphology_presets, []),
    syntaxPresets: () => json(wasm.get_syntax_presets, []),
    soundChangePresets: () => json(wasm.get_sound_change_presets, []),

    /**
     * Generate one word from an inventory.
     * @returns {{word: string, ipa: string}}
     */
    generateWord({ vowels, consonants, syllableStructure, tones, vowelHarmony, syllables, seed }) {
      return json(wasm.generate_word, [
        JSON.stringify(vowels),
        JSON.stringify(consonants),
        syllableStructure,
        tones ?? undefined,
        vowelHarmony ?? undefined,
        syllables,
        toSeed(seed),
      ]);
    },

    /**
     * Generate a batch of distinct words. May return fewer than `count` when
     * the inventory cannot supply that many forms.
     * @returns {Array<{word: string, ipa: string}>}
     */
    generateWords({ vowels, consonants, syllableStructure, tones, vowelHarmony, syllables, count, seed }) {
      return json(wasm.generate_words, [
        JSON.stringify(vowels),
        JSON.stringify(consonants),
        syllableStructure,
        tones ?? undefined,
        vowelHarmony ?? undefined,
        syllables,
        count,
        toSeed(seed),
      ]);
    },

    /**
     * Generate a full lexicon, keyed by headword.
     * @returns {Object<string, object>}
     */
    generateLexicon(config) {
      return json(wasm.generate_lexicon, [
        JSON.stringify({
          vowels: config.vowels,
          consonants: config.consonants,
          syllable_structure: config.syllableStructure,
          tones: config.tones ?? null,
          vowel_harmony: config.vowelHarmony ?? null,
          morph_rules: config.morphRules ?? [],
          noun_classes: config.nounClasses ?? null,
          sound_changes: config.soundChanges ?? [],
          size: config.size,
          syllables_per_word: config.syllables,
          seed: config.seed === undefined || config.seed === null || config.seed === ''
            ? null
            : Math.trunc(Number(config.seed)),
        }),
      ]);
    },

    /**
     * Age a lexicon's meanings.
     * @returns {{lexicon: object, history: object}}
     */
    applySemanticDrift({ lexicon, driftRate, timeSteps, seed }) {
      return json(wasm.apply_semantic_drift, [
        JSON.stringify(lexicon),
        driftRate,
        timeSteps,
        toSeed(seed),
      ]);
    },

    /** Build a writing system for an inventory. */
    generateOrthography({ vowels, consonants, scriptType, style, seed }) {
      return json(wasm.generate_orthography, [
        JSON.stringify(vowels),
        JSON.stringify(consonants),
        scriptType,
        style,
        toSeed(seed),
      ]);
    },

    /** Arrange words according to a word order and case system. */
    generateSentence({ words, wordOrder, cases }) {
      return callEngine(wasm.generate_sentence, [
        JSON.stringify(words),
        wordOrder,
        JSON.stringify(cases ?? []),
      ]);
    },

    /** One-line typological summary of a syntactic profile. */
    describeSyntax({ wordOrder, cases }) {
      return callEngine(wasm.describe_syntax, [wordOrder, JSON.stringify(cases ?? [])]);
    },

    /** Apply formal sound-change rules, returning only the final form. */
    applySoundChanges({ word, rules }) {
      return callEngine(wasm.apply_sound_changes, [word, JSON.stringify(rules)]);
    },

    /**
     * Apply formal rules and return the full derivation.
     * @returns {Array<{rule: string, before: string, after: string, changed: boolean}>}
     */
    traceSoundChanges({ word, rules }) {
      return json(wasm.trace_sound_changes, [word, JSON.stringify(rules)]);
    },

    /** Parse one rule, throwing with the parser's message if it is malformed. */
    parseSoundRule(rule) {
      return json(wasm.parse_sound_rule, [rule]);
    },
  };
}
