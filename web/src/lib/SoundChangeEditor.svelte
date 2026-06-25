<!--
  SoundChangeEditor — Rule editor and word tester using formal sound-change notation.
  Supports preset loads (Grimm's Law, Verner's Law, palatalization, devoicing)
  and real-time application to a test word.
-->
<script>
  // Example rules to get started
  let rules = $state([
    { id: 1, rule: 'p > b / V_V', description: 'Voicing between vowels' },
    { id: 2, rule: 'k > h / _#', description: 'Final spirantization' },
    { id: 3, rule: 's > ∅ / #_', description: 'Initial s-deletion' },
  ]);
  let newRule = $state('');
  let testWord = $state('paka');
  let result = $state('');
  let error = $state('');

  // Validate and add a new rule
  function addRule() {
    if (!newRule.trim()) return;
    error = '';
    if (!newRule.includes('>')) {
      error = 'Rule must contain > (e.g. p > b)';
      return;
    }
    rules = [...rules, { id: Date.now(), rule: newRule.trim(), description: '' }];
    newRule = '';
  }

  // Remove a rule by id
  function removeRule(id) {
    rules = rules.filter(r => r.id !== id);
  }

  // Apply all rules to the test word sequentially
  async function testRules() {
    error = '';
    try {
      // Future: call wasm.apply_sound_changes(testWord, rulesJson)
      let word = testWord;
      for (const r of rules) {
        const [from, to] = r.rule.split('>').map(s => s.trim());
        if (to === '∅') {
          word = word.replace(new RegExp(from, 'g'), '');
        } else {
          word = word.replace(new RegExp(from, 'g'), to);
        }
      }
      result = `${testWord} → ${word}`;
    } catch (e) {
      error = e.message || 'Test failed';
    }
  }

  // Well-known sound change presets
  const presets = {
    grimm: [
      'p > f', 't > θ', 'k > h',
    ],
    verner: [
      'f > v / V_V', 'θ > ð / V_V', 's > z / V_V',
    ],
    palatal: [
      'k > ch / _i', 'g > j / _i',
    ],
    devoice: [
      'b > p / _#', 'd > t / _#', 'g > k / _#',
    ],
  };

  // Load a preset, replacing the current rule list
  function loadPreset(name) {
    rules = presets[name].map((rule, i) => ({ id: Date.now() + i, rule, description: '' }));
  }
</script>

<div class="editor">
  <h2>Sound Change Editor</h2>

  <div class="presets">
    <span>Presets:</span>
    <button onclick={() => loadPreset('grimm')}>Grimm's Law</button>
    <button onclick={() => loadPreset('verner')}>Verner's Law</button>
    <button onclick={() => loadPreset('palatal')}>Palatalization</button>
    <button onclick={() => loadPreset('devoice')}>Final Devoicing</button>
  </div>

  <div class="add-rule">
    <input type="text" bind:value={newRule} placeholder="e.g. p > b / V_V" onkeydown={(e) => e.key === 'Enter' && addRule()} />
    <button onclick={addRule}>Add Rule</button>
  </div>

  {#if error}<div class="error">{error}</div>{/if}

  <div class="rule-list">
    {#each rules as rule (rule.id)}
      <div class="rule">
        <code>{rule.rule}</code>
        {#if rule.description}<span class="desc">{rule.description}</span>{/if}
        <button class="remove" onclick={() => removeRule(rule.id)}>×</button>
      </div>
    {/each}
    {#if rules.length === 0}
      <div class="empty">No rules. Add one above or load a preset.</div>
    {/if}
  </div>

  <div class="tester">
    <h4>Test</h4>
    <input type="text" bind:value={testWord} placeholder="Enter a word..." />
    <button onclick={testRules}>Apply Rules</button>
    {#if result}<div class="result">{result}</div>{/if}
  </div>
</div>

<style>
  .editor { padding: 1rem; }
  .presets { display: flex; gap: 0.5rem; align-items: center; margin-bottom: 1rem; flex-wrap: wrap; }
  .presets button { padding: 0.3rem 0.75rem; background: #21262d; color: #58a6ff; border: 1px solid #30363d; border-radius: 4px; cursor: pointer; font-size: 0.85rem; }
  .presets button:hover { background: #30363d; }
  .add-rule { display: flex; gap: 0.5rem; margin-bottom: 1rem; }
  .add-rule input { flex: 1; padding: 0.4rem; background: #0d1117; color: #c9d1d9; border: 1px solid #30363d; border-radius: 4px; font-family: monospace; }
  .add-rule button { padding: 0.4rem 1rem; background: #1f6feb; color: #fff; border: none; border-radius: 4px; cursor: pointer; }
  .rule-list { margin-bottom: 1rem; }
  .rule { display: flex; align-items: center; gap: 0.75rem; padding: 0.4rem 0.75rem; background: #161b22; border: 1px solid #21262d; border-radius: 4px; margin-bottom: 0.25rem; }
  .rule code { font-family: monospace; color: #d2a8ff; }
  .rule .desc { color: #8b949e; font-size: 0.85rem; }
  .rule .remove { margin-left: auto; background: none; border: none; color: #f85149; cursor: pointer; font-size: 1.2rem; }
  .tester { padding: 1rem; background: #161b22; border-radius: 6px; }
  .tester h4 { margin: 0 0 0.5rem; }
  .tester input { padding: 0.4rem; background: #0d1117; color: #c9d1d9; border: 1px solid #30363d; border-radius: 4px; margin-right: 0.5rem; font-family: monospace; }
  .tester button { padding: 0.4rem 1rem; background: #238636; color: #fff; border: none; border-radius: 4px; cursor: pointer; }
  .result { margin-top: 0.5rem; padding: 0.5rem; background: #0d419d; border-radius: 4px; font-family: monospace; }
  .error { padding: 0.5rem; background: #490202; color: #f85149; border-radius: 4px; margin-bottom: 0.5rem; }
  .empty { padding: 1rem; text-align: center; color: #484f58; }
</style>
