<!--
  LexiconBrowser — Configure and generate a conlang lexicon via the WASM engine.

  Every dropdown is populated from the engine's embedded registries rather than
  a hardcoded list, so the options can never drift out of sync with data/*.toml.
  Results are shown in a sortable table and can be downloaded as JSON.
-->
<script>
  import { presets } from './engine.js';

  let { engine } = $props();

  const { phonologies, morphologies, syntaxes, soundChanges } = presets();

  const phonologyNames = Object.keys(phonologies).sort();
  const morphologyNames = Object.keys(morphologies).sort();
  const syntaxNames = Object.keys(syntaxes).sort();
  const soundChangeNames = Object.keys(soundChanges).sort();

  let config = $state({
    phonology: phonologyNames.includes('uralic_finnic') ? 'uralic_finnic' : phonologyNames[0],
    morphology: morphologyNames.includes('agglutinative') ? 'agglutinative' : morphologyNames[0],
    syntax: syntaxNames.includes('svo') ? 'svo' : syntaxNames[0],
    soundChanges: 'none',
    syllables: 2,
    size: 50,
    seed: '',
    driftSteps: 0,
    driftRate: 0.15,
  });

  let entries = $state([]);
  let sentence = $state('');
  let typology = $state('');
  let error = $state('');
  let notice = $state('');
  let loading = $state(false);
  let sortKey = $state('headword');

  const sorted = $derived(
    [...entries].sort((a, b) => {
      if (sortKey === 'part_of_speech') {
        return a.part_of_speech.localeCompare(b.part_of_speech) ||
          a.headword.localeCompare(b.headword);
      }
      return a.headword.localeCompare(b.headword);
    }),
  );

  function generate() {
    loading = true;
    error = '';
    notice = '';
    try {
      const phono = phonologies[config.phonology];
      const morph = morphologies[config.morphology];
      const syntax = syntaxes[config.syntax];

      let lexicon = engine.generateLexicon({
        vowels: phono.vowels,
        consonants: phono.consonants,
        syllableStructure: phono.syllable_structure,
        tones: phono.tones ?? null,
        vowelHarmony: phono.vowel_harmony ?? null,
        morphRules: morph.rules,
        nounClasses: morph.noun_classes ?? null,
        soundChanges: soundChanges[config.soundChanges] ?? [],
        size: Number(config.size),
        syllables: Number(config.syllables),
        seed: config.seed,
      });

      if (Number(config.driftSteps) > 0) {
        const drifted = engine.applySemanticDrift({
          lexicon,
          driftRate: Number(config.driftRate),
          timeSteps: Number(config.driftSteps),
          seed: config.seed === '' ? null : Number(config.seed) + 1,
        });
        lexicon = drifted.lexicon;
      }

      entries = Object.values(lexicon);

      // The engine stops early when the inventory runs out of distinct forms,
      // so say so rather than silently returning a short dictionary.
      if (entries.length < Number(config.size)) {
        notice = `Produced ${entries.length} of ${config.size} entries — this phonology cannot supply more distinct headwords.`;
      }

      typology = engine.describeSyntax({
        wordOrder: syntax.word_order,
        cases: syntax.cases ?? [],
      });

      const nouns = entries.filter((e) => e.part_of_speech === 'noun');
      const verbs = entries.filter((e) => e.part_of_speech === 'verb');
      if (nouns.length >= 2 && verbs.length >= 1) {
        sentence = engine.generateSentence({
          words: [nouns[0].headword, verbs[0].headword, nouns[1].headword],
          wordOrder: syntax.word_order,
          cases: syntax.cases ?? [],
        });
      } else {
        sentence = '';
      }
    } catch (e) {
      entries = [];
      sentence = '';
      typology = '';
      error = e.message;
    } finally {
      loading = false;
    }
  }

  /** Offer the generated lexicon as a JSON download. */
  function download() {
    const payload = Object.fromEntries(entries.map((e) => [e.headword, e]));
    const blob = new Blob([JSON.stringify(payload, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `${config.phonology}-${config.morphology}.json`;
    link.click();
    URL.revokeObjectURL(url);
  }
</script>

<div class="lexicon">
  <h2>Lexicon Browser</h2>

  <div class="controls">
    <label>Phonology:
      <select bind:value={config.phonology}>
        {#each phonologyNames as p}<option value={p}>{p}</option>{/each}
      </select>
    </label>
    <label>Morphology:
      <select bind:value={config.morphology}>
        {#each morphologyNames as m}<option value={m}>{m}</option>{/each}
      </select>
    </label>
    <label>Syntax:
      <select bind:value={config.syntax}>
        {#each syntaxNames as s}<option value={s}>{s}</option>{/each}
      </select>
    </label>
    <label>Sound:
      <select bind:value={config.soundChanges}>
        {#each soundChangeNames as sc}<option value={sc}>{sc}</option>{/each}
      </select>
    </label>
    <label>Syllables:
      <input type="number" bind:value={config.syllables} min="1" max="6" style="width:50px" />
    </label>
    <label>Entries:
      <input type="number" bind:value={config.size} min="1" max="2000" style="width:70px" />
    </label>
    <label>Seed:
      <input type="number" bind:value={config.seed} placeholder="random" style="width:90px" />
    </label>
    <label>Drift steps:
      <input type="number" bind:value={config.driftSteps} min="0" max="20" style="width:50px" />
    </label>
    <label>Drift rate:
      <input type="number" bind:value={config.driftRate} min="0" max="1" step="0.05" style="width:60px" />
    </label>
    <button onclick={generate} disabled={loading}>
      {loading ? 'Generating…' : 'Generate'}
    </button>
    <button class="secondary" onclick={download} disabled={entries.length === 0}>
      Download JSON
    </button>
  </div>

  {#if error}<div class="error">{error}</div>{/if}
  {#if notice}<div class="notice">{notice}</div>{/if}

  {#if typology}
    <div class="summary">
      <span>{entries.length} entries</span>
      <span>{typology}</span>
      {#if sentence}<span class="sentence">{sentence}</span>{/if}
    </div>
  {/if}

  <div class="table">
    <div class="header">
      <button class="sort" onclick={() => (sortKey = 'headword')} class:active={sortKey === 'headword'}>Word</button>
      <button class="sort" onclick={() => (sortKey = 'part_of_speech')} class:active={sortKey === 'part_of_speech'}>POS</button>
      <span>IPA</span><span>Definition</span><span>Etymology</span>
    </div>
    {#each sorted as entry (entry.headword)}
      <div class="row">
        <span class="word">{entry.headword}</span>
        <span class="pos">{entry.part_of_speech}</span>
        <span class="ipa">{entry.ipa}</span>
        <span class="def">{entry.senses[0]?.definition ?? ''}</span>
        <span class="etym">{entry.etymology}</span>
      </div>
    {/each}
    {#if entries.length === 0 && !loading}
      <div class="empty">Configure and click Generate to create a lexicon.</div>
    {/if}
  </div>
</div>

<style>
  .lexicon { padding: 1rem; }
  .controls { display: flex; gap: 0.75rem; flex-wrap: wrap; align-items: center; margin-bottom: 1rem; padding: 0.75rem; background: #161b22; border-radius: 6px; }
  .controls label { font-size: 0.85rem; display: flex; align-items: center; gap: 0.25rem; }
  .controls select, .controls input { background: #0d1117; color: #c9d1d9; border: 1px solid #30363d; border-radius: 4px; padding: 0.25rem; }
  .controls button { padding: 0.4rem 1rem; background: #238636; color: #fff; border: none; border-radius: 6px; cursor: pointer; font-weight: 600; }
  .controls button.secondary { background: #21262d; color: #c9d1d9; border: 1px solid #30363d; }
  .controls button:disabled { opacity: 0.5; cursor: default; }
  .error { padding: 0.5rem; background: #490202; color: #f85149; border-radius: 4px; margin-bottom: 0.5rem; }
  .notice { padding: 0.5rem; background: #341a00; color: #d29922; border-radius: 4px; margin-bottom: 0.5rem; }
  .summary { display: flex; gap: 1.5rem; flex-wrap: wrap; padding: 0.5rem 0.75rem; margin-bottom: 0.5rem; background: #161b22; border-radius: 6px; font-size: 0.85rem; color: #8b949e; }
  .summary .sentence { color: #7ee787; font-family: ui-monospace, monospace; }
  .table { border: 1px solid #30363d; border-radius: 6px; overflow: hidden; }
  .header, .row { display: grid; grid-template-columns: 1fr 0.7fr 1fr 2fr 1.5fr; gap: 0.5rem; padding: 0.4rem 0.75rem; font-size: 0.85rem; align-items: center; }
  .header { background: #161b22; font-weight: 600; color: #8b949e; }
  .header button.sort { background: none; border: none; color: #8b949e; font-weight: 600; font-size: 0.85rem; cursor: pointer; text-align: left; padding: 0; }
  .header button.sort.active { color: #58a6ff; }
  .row { border-top: 1px solid #21262d; }
  .row:hover { background: #161b22; }
  .word { font-weight: 600; color: #58a6ff; }
  .pos { color: #7ee787; }
  .ipa { font-family: ui-monospace, monospace; color: #d2a8ff; }
  .empty { padding: 2rem; text-align: center; color: #484f58; }
</style>
