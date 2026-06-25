<script>
  import PhonologyDesigner from './lib/PhonologyDesigner.svelte';
  import LexiconBrowser from './lib/LexiconBrowser.svelte';
  import SoundChangeEditor from './lib/SoundChangeEditor.svelte';

  let activeTab = $state('designer');

  const tabs = [
    { id: 'designer', label: 'Phonology Designer' },
    { id: 'lexicon', label: 'Lexicon Browser' },
    { id: 'sound', label: 'Sound Changes' },
  ];
</script>

<div class="app">
  <header>
    <h1>Esoterica</h1>
    <p>A modular conlang generator</p>
    <nav>
      {#each tabs as tab}
        <button
          class="tab"
          class:active={activeTab === tab.id}
          onclick={() => activeTab = tab.id}
        >
          {tab.label}
        </button>
      {/each}
    </nav>
  </header>

  <main>
    {#if activeTab === 'designer'}
      <PhonologyDesigner />
    {:else if activeTab === 'lexicon'}
      <LexiconBrowser />
    {:else if activeTab === 'sound'}
      <SoundChangeEditor />
    {/if}
  </main>

  <footer>
    <p>
      Built with <a href="https://svelte.dev">Svelte 5</a> —
      <a href="https://github.com/ewanc26/esoterica">ewanc26/esoterica</a>
    </p>
  </footer>
</div>

<style>
  :global(body) {
    font-family: system-ui, -apple-system, sans-serif;
    margin: 0;
    padding: 0;
    background: #0d1117;
    color: #c9d1d9;
  }

  .app {
    max-width: 1200px;
    margin: 0 auto;
    padding: 1rem;
    min-height: 100vh;
    display: flex;
    flex-direction: column;
  }

  header {
    text-align: center;
    padding: 1rem 0;
    border-bottom: 1px solid #30363d;
    margin-bottom: 1rem;
  }

  header h1 { margin: 0; color: #58a6ff; font-size: 2rem; }
  header p { margin: 0.25rem 0 1rem; color: #8b949e; }

  nav { display: flex; gap: 0.5rem; justify-content: center; }

  button.tab {
    background: #21262d;
    color: #c9d1d9;
    border: 1px solid #30363d;
    padding: 0.5rem 1.25rem;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.9rem;
    transition: all 0.15s;
  }
  button.tab:hover { background: #30363d; }
  button.tab.active { background: #1f6feb; border-color: #1f6feb; color: #fff; }

  main { flex: 1; padding: 1rem 0; }

  footer {
    text-align: center;
    padding: 1rem;
    color: #484f58;
    font-size: 0.85rem;
    border-top: 1px solid #30363d;
  }
  footer a { color: #58a6ff; text-decoration: none; }
</style>
