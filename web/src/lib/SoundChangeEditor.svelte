<!--
  SoundChangeEditor — Rule editor and word tester using formal sound-change
  notation, backed by the engine's nom parser.

  Rules are validated as they are added, and applying them shows the full
  derivation rather than only the final form — the same rule-by-rule table a
  historical linguist would write out.
-->
<script>
  let { engine } = $props();

  let rules = $state([
    { id: 1, rule: 'p > b / V_V', description: 'Voicing between vowels' },
    { id: 2, rule: 'k > h / _#', description: 'Final spirantization' },
    { id: 3, rule: 's > ∅ / #_', description: 'Initial s-deletion' },
  ]);
  let newRule = $state('');
  let testWord = $state('sapak');
  let steps = $state([]);
  let error = $state('');
  let addError = $state('');

  /**
   * Validate against the real parser before accepting a rule.
   *
   * The parser rejects trailing junk too, so `p > b / V_V nonsense` fails here
   * instead of silently becoming `p > b / V_V`.
   */
  function addRule() {
    addError = '';
    const candidate = newRule.trim();
    if (!candidate) return;
    try {
      engine.parseSoundRule(candidate);
    } catch (e) {
      addError = e.message;
      return;
    }
    rules = [...rules, { id: Date.now(), rule: candidate, description: '' }];
    newRule = '';
    testRules();
  }

  function removeRule(id) {
    rules = rules.filter((r) => r.id !== id);
    testRules();
  }

  function moveRule(index, delta) {
    const target = index + delta;
    if (target < 0 || target >= rules.length) return;
    const next = [...rules];
    [next[index], next[target]] = [next[target], next[index]];
    rules = next;
    // Sound changes are order-dependent, so reordering changes the outcome.
    testRules();
  }

  /** Apply every rule in order and keep the intermediate forms. */
  function testRules() {
    error = '';
    steps = [];
    if (!testWord.trim()) return;
    try {
      steps = engine.traceSoundChanges({
        word: testWord.trim(),
        rules: rules.map((r) => r.rule),
      });
    } catch (e) {
      error = e.message;
    }
  }

  const finalForm = $derived(steps.length > 0 ? steps[steps.length - 1].after : testWord);

  // Well-known sound change presets
  const presets = {
    grimm: ['p > f', 't > θ', 'k > h'],
    verner: ['f > v / V_V', 'θ > ð / V_V', 's > z / V_V'],
    palatal: ['k > ch / _i', 'g > j / _i'],
    devoice: ['b > p / _#', 'd > t / _#', 'g > k / _#'],
    lenition: ['p > b / V_V', 't > d / V_V', 'k > g / V_V'],
    apocope: ['a > ∅ / _#', 'e > ∅ / _#', 'o > ∅ / _#'],
  };

  function loadPreset(name) {
    rules = presets[name].map((rule, i) => ({ id: Date.now() + i, rule, description: '' }));
    testRules();
  }

  testRules();
</script>

<div class="editor">
  <h2>Sound Change Editor</h2>

  <div class="presets">
    <span>Presets:</span>
    <button onclick={() => loadPreset('grimm')}>Grimm's Law</button>
    <button onclick={() => loadPreset('verner')}>Verner's Law</button>
    <button onclick={() => loadPreset('palatal')}>Palatalization</button>
    <button onclick={() => loadPreset('devoice')}>Final Devoicing</button>
    <button onclick={() => loadPreset('lenition')}>Lenition</button>
    <button onclick={() => loadPreset('apocope')}>Apocope</button>
  </div>

  <p class="hint">
    <code>V</code> matches any vowel, <code>C</code> any consonant,
    <code>#</code> a word boundary, and <code>∅</code> marks deletion.
    Example: <code>p &gt; b / V_V</code>
  </p>

  <div class="add-rule">
    <input
      type="text"
      bind:value={newRule}
      placeholder="e.g. p > b / V_V"
      onkeydown={(e) => e.key === 'Enter' && addRule()}
    />
    <button onclick={addRule}>Add Rule</button>
  </div>

  {#if addError}<div class="error">{addError}</div>{/if}

  <div class="rule-list">
    {#each rules as rule, i (rule.id)}
      <div class="rule">
        <span class="order">{i + 1}</span>
        <code>{rule.rule}</code>
        {#if rule.description}<span class="desc">{rule.description}</span>{/if}
        <span class="spacer"></span>
        <button class="move" onclick={() => moveRule(i, -1)} disabled={i === 0} title="Apply earlier">↑</button>
        <button class="move" onclick={() => moveRule(i, 1)} disabled={i === rules.length - 1} title="Apply later">↓</button>
        <button class="remove" onclick={() => removeRule(rule.id)} title="Remove">×</button>
      </div>
    {/each}
    {#if rules.length === 0}
      <div class="empty">No rules. Add one above or load a preset.</div>
    {/if}
  </div>

  <div class="tester">
    <h4>Derivation</h4>
    <div class="test-input">
      <input type="text" bind:value={testWord} oninput={testRules} placeholder="Enter a word…" />
      <button onclick={testRules}>Apply Rules</button>
    </div>

    {#if error}<div class="error">{error}</div>{/if}

    {#if steps.length > 0}
      <div class="result">{testWord} → {finalForm}</div>
      <div class="steps">
        {#each steps as step, i}
          <div class="step" class:inert={!step.changed}>
            <span class="idx">{i + 1}</span>
            <code class="rule-text">{step.rule}</code>
            <span class="form">{step.before}</span>
            <span class="arrow">→</span>
            <span class="form after">{step.after}</span>
            {#if !step.changed}<span class="no-op">no change</span>{/if}
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .editor { padding: 1rem; }
  .presets { display: flex; gap: 0.5rem; align-items: center; margin-bottom: 0.75rem; flex-wrap: wrap; }
  .presets button { padding: 0.3rem 0.75rem; background: #21262d; color: #58a6ff; border: 1px solid #30363d; border-radius: 4px; cursor: pointer; font-size: 0.85rem; }
  .presets button:hover { background: #30363d; }
  .hint { font-size: 0.85rem; color: #8b949e; margin: 0 0 1rem; }
  .hint code { background: #161b22; padding: 0.1rem 0.3rem; border-radius: 3px; color: #d2a8ff; }
  .add-rule { display: flex; gap: 0.5rem; margin-bottom: 1rem; }
  .add-rule input { flex: 1; padding: 0.4rem; background: #0d1117; color: #c9d1d9; border: 1px solid #30363d; border-radius: 4px; font-family: ui-monospace, monospace; }
  .add-rule button { padding: 0.4rem 1rem; background: #1f6feb; color: #fff; border: none; border-radius: 4px; cursor: pointer; }
  .rule-list { margin-bottom: 1rem; }
  .rule { display: flex; align-items: center; gap: 0.75rem; padding: 0.4rem 0.75rem; background: #161b22; border: 1px solid #21262d; border-radius: 4px; margin-bottom: 0.25rem; }
  .rule .order { color: #484f58; font-size: 0.8rem; min-width: 1.2rem; }
  .rule code { font-family: ui-monospace, monospace; color: #d2a8ff; }
  .rule .desc { color: #8b949e; font-size: 0.85rem; }
  .rule .spacer { flex: 1; }
  .rule .move { background: none; border: none; color: #8b949e; cursor: pointer; font-size: 0.9rem; padding: 0 0.2rem; }
  .rule .move:disabled { opacity: 0.25; cursor: default; }
  .rule .remove { background: none; border: none; color: #f85149; cursor: pointer; font-size: 1.2rem; }
  .tester { padding: 1rem; background: #161b22; border-radius: 6px; }
  .tester h4 { margin: 0 0 0.5rem; }
  .test-input { display: flex; gap: 0.5rem; }
  .tester input { flex: 1; padding: 0.4rem; background: #0d1117; color: #c9d1d9; border: 1px solid #30363d; border-radius: 4px; font-family: ui-monospace, monospace; }
  .tester button { padding: 0.4rem 1rem; background: #238636; color: #fff; border: none; border-radius: 4px; cursor: pointer; }
  .result { margin-top: 0.75rem; padding: 0.5rem; background: #0d419d; border-radius: 4px; font-family: ui-monospace, monospace; font-size: 1.05rem; }
  .steps { margin-top: 0.5rem; }
  .step { display: flex; align-items: center; gap: 0.6rem; padding: 0.3rem 0.5rem; border-top: 1px solid #21262d; font-size: 0.85rem; }
  .step.inert { opacity: 0.45; }
  .step .idx { color: #484f58; min-width: 1.2rem; }
  .step .rule-text { color: #d2a8ff; min-width: 10rem; }
  .step .form { font-family: ui-monospace, monospace; }
  .step .form.after { color: #7ee787; }
  .step .arrow { color: #484f58; }
  .step .no-op { color: #484f58; font-style: italic; }
  .error { padding: 0.5rem; background: #490202; color: #f85149; border-radius: 4px; margin-bottom: 0.5rem; }
  .empty { padding: 1rem; text-align: center; color: #484f58; }
</style>
