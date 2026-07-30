<!--
  OrthographyStudio — Renders the procedurally generated writing systems.

  The engine has always emitted SVG path data for every glyph; nothing in the
  UI drew it. This tab picks a script type and glyph style, generates the
  mapping for an inventory, and renders each glyph as actual SVG.
-->
<script>
  import { presets } from './engine.js';

  let { engine } = $props();

  const SCRIPT_TYPES = ['alphabet', 'abjad', 'abugida', 'syllabary', 'logography'];
  const GLYPH_STYLES = ['angular', 'curved', 'minimal', 'ornate'];

  const { phonologies } = presets();
  const phonologyNames = Object.keys(phonologies).sort();

  let phonology = $state(
    phonologyNames.includes('oceania_austronesian') ? 'oceania_austronesian' : phonologyNames[0],
  );
  let scriptType = $state('alphabet');
  let style = $state('angular');
  let seed = $state('7');
  let filter = $state('');

  let glyphs = $state([]);
  let error = $state('');

  function generate() {
    error = '';
    try {
      const phono = phonologies[phonology];
      const mapping = engine.generateOrthography({
        vowels: phono.vowels,
        consonants: phono.consonants,
        scriptType,
        style,
        seed: seed === '' ? null : seed,
      });
      glyphs = Object.entries(mapping).map(([key, glyph]) => ({ key, ...glyph }));
    } catch (e) {
      glyphs = [];
      error = e.message;
    }
  }

  const visible = $derived(
    filter.trim() === ''
      ? glyphs
      : glyphs.filter(
          (g) =>
            g.key.includes(filter.trim()) ||
            g.category.includes(filter.trim()),
        ),
  );

  const categories = $derived([...new Set(glyphs.map((g) => g.category))].sort());

  /** Bundle every glyph into one downloadable SVG sheet. */
  function downloadSheet() {
    const columns = 10;
    const cell = 40;
    const rows = Math.ceil(visible.length / columns);
    const body = visible
      .map((glyph, i) => {
        const x = (i % columns) * cell;
        const y = Math.floor(i / columns) * cell;
        return `  <g transform="translate(${x},${y})"><path d="${glyph.svg_path}" fill="none" stroke="black" stroke-width="1.5"/></g>`;
      })
      .join('\n');
    const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${columns * cell} ${rows * cell}">\n${body}\n</svg>`;
    const blob = new Blob([svg], { type: 'image/svg+xml' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `${phonology}-${scriptType}-${style}.svg`;
    link.click();
    URL.revokeObjectURL(url);
  }

  generate();
</script>

<div class="studio">
  <h2>Orthography Studio</h2>

  <div class="controls">
    <label>Phonology:
      <select bind:value={phonology} onchange={generate}>
        {#each phonologyNames as p}<option value={p}>{p}</option>{/each}
      </select>
    </label>
    <label>Script:
      <select bind:value={scriptType} onchange={generate}>
        {#each SCRIPT_TYPES as s}<option value={s}>{s}</option>{/each}
      </select>
    </label>
    <label>Style:
      <select bind:value={style} onchange={generate}>
        {#each GLYPH_STYLES as s}<option value={s}>{s}</option>{/each}
      </select>
    </label>
    <label>Seed:
      <input type="number" bind:value={seed} oninput={generate} placeholder="random" style="width:90px" />
    </label>
    <label>Filter:
      <input type="text" bind:value={filter} placeholder="phoneme or category" style="width:150px" />
    </label>
    <button onclick={generate}>Regenerate</button>
    <button class="secondary" onclick={downloadSheet} disabled={visible.length === 0}>
      Download SVG
    </button>
  </div>

  {#if error}<div class="error">{error}</div>{/if}

  {#if glyphs.length > 0}
    <div class="summary">
      <span>{visible.length} of {glyphs.length} glyphs</span>
      <span>{categories.join(', ')}</span>
    </div>
  {/if}

  <div class="sheet">
    {#each visible as glyph (glyph.key)}
      <figure class="glyph" title={glyph.description}>
        <svg viewBox="0 0 30 30" role="img" aria-label={glyph.description}>
          <path d={glyph.svg_path} fill="none" stroke="currentColor" stroke-width="1.5" />
        </svg>
        <figcaption>{glyph.key}</figcaption>
      </figure>
    {/each}
    {#if visible.length === 0 && !error}
      <div class="empty">No glyphs match.</div>
    {/if}
  </div>
</div>

<style>
  .studio { padding: 1rem; }
  .controls { display: flex; gap: 0.75rem; flex-wrap: wrap; align-items: center; margin-bottom: 1rem; padding: 0.75rem; background: #161b22; border-radius: 6px; }
  .controls label { font-size: 0.85rem; display: flex; align-items: center; gap: 0.25rem; }
  .controls select, .controls input { background: #0d1117; color: #c9d1d9; border: 1px solid #30363d; border-radius: 4px; padding: 0.25rem; }
  .controls button { padding: 0.4rem 1rem; background: #238636; color: #fff; border: none; border-radius: 6px; cursor: pointer; font-weight: 600; }
  .controls button.secondary { background: #21262d; color: #c9d1d9; border: 1px solid #30363d; }
  .controls button:disabled { opacity: 0.5; cursor: default; }
  .summary { display: flex; gap: 1.5rem; flex-wrap: wrap; padding: 0.5rem 0.75rem; margin-bottom: 0.75rem; background: #161b22; border-radius: 6px; font-size: 0.85rem; color: #8b949e; }
  .error { padding: 0.5rem; background: #490202; color: #f85149; border-radius: 4px; margin-bottom: 0.5rem; }
  .sheet { display: flex; flex-wrap: wrap; gap: 0.5rem; }
  .glyph {
    margin: 0;
    width: 74px;
    padding: 0.4rem;
    background: #161b22;
    border: 1px solid #21262d;
    border-radius: 6px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.25rem;
    color: #7ee787;
  }
  .glyph:hover { border-color: #2ea043; }
  .glyph svg { width: 48px; height: 48px; }
  .glyph figcaption {
    font-family: ui-monospace, monospace;
    font-size: 0.75rem;
    color: #8b949e;
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .empty { padding: 2rem; color: #484f58; }
</style>
