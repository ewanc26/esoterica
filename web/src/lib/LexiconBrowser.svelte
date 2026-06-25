<script>
  let entries = $state([]);
  let loading = $state(false);
  let error = $state('');
  let config = $state({
    phonology: 'uralic_finnic',
    morphology: 'agglutinative',
    syntax: 'svo',
    soundChanges: 'none',
    syllables: 2,
    size: 50,
  });

  const phonologies = ['eurasia_ie_germanic','eurasia_ie_romance','africa_afroasiatic_semitic','africa_nigercongo_bantu','asia_sinotibetan_sinitic','americas_utoaztecan','oceania_austronesian','uralic_finnic'];
  const morphologies = ['agglutinative','fusional','analytic','polysynthetic'];
  const syntaxes = ['svo','sov','vso','vos','ovs','osv'];
  const soundChanges = ['none','lenition','spirantization','grimms_law','final_devoicing','nasal_assimilation','intervocalic_voicing','vowel_reduction','palatalization','rhotacism','cluster_simplification'];

  async function generate() {
    loading = true; error = '';
    try {
      // In production: call wasm.generate_lexicon(configJson)
      const mockWords = [];
      const chars = 'ptkmnslrhaeiou';
      for (let i = 0; i < config.size; i++) {
        let w = '';
        for (let s = 0; s < config.syllables; s++) {
          w += chars[Math.floor(Math.random() * chars.length)];
          w += chars[Math.floor(Math.random() * 5) + 8];
        }
        mockWords.push({
          headword: w,
          part_of_speech: ['noun','verb','adjective'][Math.floor(Math.random() * 3)],
          ipa: `/${w}/`,
          etymology: `Proto-root *${w}`,
          senses: [{ definition: `A ${w} in the conlang.` }],
          root: w,
        });
      }
      entries = mockWords;
    } catch (e) {
      error = e.message || 'Generation failed';
    } finally {
      loading = false;
    }
  }
</script>

<div class="lexicon">
  <h2>Lexicon Browser</h2>

  <div class="controls">
    <label>Phonology:
      <select bind:value={config.phonology}>
        {#each phonologies as p}<option value={p}>{p}</option>{/each}
      </select>
    </label>
    <label>Morphology:
      <select bind:value={config.morphology}>
        {#each morphologies as m}<option value={m}>{m}</option>{/each}
      </select>
    </label>
    <label>Syntax:
      <select bind:value={config.syntax}>
        {#each syntaxes as s}<option value={s}>{s}</option>{/each}
      </select>
    </label>
    <label>Sound:
      <select bind:value={config.soundChanges}>
        {#each soundChanges as sc}<option value={sc}>{sc}</option>{/each}
      </select>
    </label>
    <label>Syllables:
      <input type="number" bind:value={config.syllables} min="1" max="6" style="width:50px" />
    </label>
    <label>Entries:
      <input type="number" bind:value={config.size} min="10" max="500" style="width:60px" />
    </label>
    <button onclick={generate} disabled={loading}>
      {loading ? 'Generating...' : 'Generate'}
    </button>
  </div>

  {#if error}<div class="error">{error}</div>{/if}

  <div class="table">
    <div class="header">
      <span>Word</span><span>POS</span><span>IPA</span><span>Definition</span><span>Etymology</span>
    </div>
    {#each entries as entry (entry.headword)}
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
  .controls button:disabled { opacity: 0.5; }
  .error { padding: 0.5rem; background: #490202; color: #f85149; border-radius: 4px; margin-bottom: 0.5rem; }
  .table { border: 1px solid #30363d; border-radius: 6px; overflow: hidden; }
  .header, .row { display: grid; grid-template-columns: 1fr 0.7fr 1fr 2fr 1.5fr; gap: 0.5rem; padding: 0.4rem 0.75rem; font-size: 0.85rem; align-items: center; }
  .header { background: #161b22; font-weight: 600; color: #8b949e; }
  .row { border-top: 1px solid #21262d; }
  .row:hover { background: #161b22; }
  .word { font-weight: 600; color: #58a6ff; }
  .pos { color: #7ee787; }
  .ipa { font-family: monospace; color: #d2a8ff; }
  .empty { padding: 2rem; text-align: center; color: #484f58; }
</style>
