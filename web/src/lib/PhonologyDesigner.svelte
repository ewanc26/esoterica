<script>
  // IPA consonant chart data
  const IPA_CONSONANTS = [
    { label: 'Plosive', sounds: ['p','b','t','d','ʈ','ɖ','c','ɟ','k','g','q','ɢ','ʔ'] },
    { label: 'Nasal', sounds: ['m','ɱ','n','ɳ','ɲ','ŋ','ɴ'] },
    { label: 'Fricative', sounds: ['ɸ','β','f','v','θ','ð','s','z','ʃ','ʒ','x','ɣ','h','ɦ'] },
    { label: 'Affricate', sounds: ['ts','dz','tʃ','dʒ'] },
    { label: 'Approximant', sounds: ['w','ʋ','l','j','ɹ','ɻ'] },
    { label: 'Trill/Tap', sounds: ['ɾ','r','ʀ'] },
  ];
  const IPA_VOWELS = [
    { label: 'Close', sounds: ['i','y','ɨ','ʉ','ɯ','u'] },
    { label: 'Near-close', sounds: ['ɪ','ʏ','','','ʊ'] },
    { label: 'Close-mid', sounds: ['e','ø','ɘ','ɵ','ɤ','o'] },
    { label: 'Mid', sounds: ['','','ə','','',''] },
    { label: 'Open-mid', sounds: ['ɛ','œ','ɜ','ɞ','ʌ','ɔ'] },
    { label: 'Open', sounds: ['æ','','ɐ','','ɑ','ɒ'] },
  ];

  let vowels = $state(['a','e','i','o','u']);
  let consonants = $state(['p','t','k','m','n','s','l','r','h']);
  let syllableStructure = $state('CV');
  let tones = $state(false);
  let toneCount = $state(4);
  let vowelHarmony = $state(false);
  let preview = $state('');

  function toggleConsonant(sound) {
    if (!sound) return;
    if (consonants.includes(sound)) {
      consonants = consonants.filter(s => s !== sound);
    } else {
      consonants = [...consonants, sound];
    }
    generatePreview();
  }

  function toggleVowel(sound) {
    if (!sound) return;
    if (vowels.includes(sound)) {
      vowels = vowels.filter(s => s !== sound);
    } else {
      vowels = [...vowels, sound];
    }
    generatePreview();
  }

  async function generatePreview() {
    try {
      const config = JSON.stringify({
        vowels, consonants,
        syllable_structure: syllableStructure,
        tones: tones ? toneCount : null,
        vowel_harmony: vowelHarmony,
        num_syllables: 3,
      });
      // In production: call wasm.generate_word(...)
      preview = `V:${vowels.length} C:${consonants.length} | ${syllableStructure} | Sample syllable would appear here`;
    } catch (e) {
      preview = 'Preview unavailable (WASM not loaded)';
    }
  }
</script>

<div class="designer">
  <h2>Phonology Designer</h2>

  <div class="status">
    <span>C: {consonants.length}</span>
    <span>V: {vowels.length}</span>
    <span>Structure:
      <input type="text" bind:value={syllableStructure} oninput={generatePreview} style="width:80px" />
    </span>
    <label><input type="checkbox" bind:checked={tones} onchange={generatePreview} /> Tones ({toneCount})</label>
    <label><input type="checkbox" bind:checked={vowelHarmony} onchange={generatePreview} /> Harmony</label>
    <button onclick={() => { vowels = ['a','i','u']; consonants = ['p','t','k','m','n','s']; generatePreview(); }}>Minimal</button>
    <button onclick={() => { vowels = ['a','e','i','o','u']; consonants = ['p','b','t','d','k','g','m','n','f','v','s','z','h','l','r','j','w']; generatePreview(); }}>Full</button>
  </div>

  <div class="preview">{preview || 'Click phonemes to build...'}</div>

  <div class="grids">
    <div class="grid">
      <h3>Consonants</h3>
      {#each IPA_CONSONANTS as row}
        <div class="ipa-row">
          <span class="label">{row.label}</span>
          {#each row.sounds as sound}
            {#if sound}
              <button
                class="phoneme"
                class:selected={consonants.includes(sound)}
                onclick={() => toggleConsonant(sound)}
              >{sound}</button>
            {:else}
              <span class="empty-slot"></span>
            {/if}
          {/each}
        </div>
      {/each}
    </div>

    <div class="grid">
      <h3>Vowels</h3>
      {#each IPA_VOWELS as row}
        <div class="ipa-row">
          <span class="label">{row.label}</span>
          {#each row.sounds as sound}
            {#if sound}
              <button
                class="phoneme"
                class:selected={vowels.includes(sound)}
                onclick={() => toggleVowel(sound)}
              >{sound}</button>
            {:else}
              <span class="empty-slot"></span>
            {/if}
          {/each}
        </div>
      {/each}
    </div>
  </div>
</div>

<style>
  .designer { padding: 1rem; }
  .status { display: flex; gap: 1rem; align-items: center; flex-wrap: wrap; margin-bottom: 1rem; padding: 0.5rem; background: #161b22; border-radius: 6px; }
  .status span, .status label { font-size: 0.9rem; }
  .status button { padding: 0.25rem 0.75rem; background: #21262d; color: #c9d1d9; border: 1px solid #30363d; border-radius: 4px; cursor: pointer; }
  .preview { padding: 0.5rem; background: #0d419d; border-radius: 4px; margin-bottom: 1rem; font-family: monospace; }
  .grids { display: flex; gap: 2rem; flex-wrap: wrap; }
  .grid { flex: 1; min-width: 300px; }
  .ipa-row { display: flex; align-items: center; gap: 0.15rem; margin-bottom: 0.25rem; }
  .label { font-size: 0.75rem; color: #8b949e; min-width: 90px; }
  button.phoneme {
    width: 36px; height: 32px; font-size: 0.85rem;
    background: #21262d; color: #484f58;
    border: 1px solid #30363d; border-radius: 4px; cursor: pointer;
    transition: all 0.1s;
  }
  button.phoneme:hover { background: #30363d; }
  button.phoneme.selected { background: #238636; color: #fff; border-color: #2ea043; }
  .empty-slot { width: 36px; }
</style>
