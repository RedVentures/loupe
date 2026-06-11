<script>
  import { app, initOutputDir, clearAll, isTabComplete } from '$lib/state.svelte.js';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { onMount } from 'svelte';
  import TabNav from '$lib/components/TabNav.svelte';
  import FigmaTab from '$lib/components/FigmaTab.svelte';
  import WebTab from '$lib/components/WebTab.svelte';
  import CompareTab from '$lib/components/CompareTab.svelte';
  import ResultTab from '$lib/components/ResultTab.svelte';
  import InspectTab from '$lib/components/InspectTab.svelte';

  onMount(() => {
    initOutputDir();

    const unlistenMenu = listen('menu-open-output-dir', () => {
      invoke('open_output_dir', { path: app.outputDir || '.' });
    });
    const unlistenFigmaProps = listen('figma-properties', (event) => {
      app.figmaProperties = event.payload;
    });
    const unlistenWebProps = listen('web-properties', (event) => {
      app.webProperties = event.payload;
    });

    // Rec #7: Clear in-memory image/design data on window close to minimize persistence
    const handleUnload = () => clearAll();
    window.addEventListener('beforeunload', handleUnload);

    return () => {
      window.removeEventListener('beforeunload', handleUnload);
      clearAll();
      unlistenMenu.then(fn => fn());
      unlistenFigmaProps.then(fn => fn());
      unlistenWebProps.then(fn => fn());
    };
  });
</script>

<div class="app">
  <header class="app-header">
    <h1 class="app-title">Loupe</h1>
  </header>

  <TabNav />

  <main class="tab-content">
    {#if app.activeTab === 0}
      <FigmaTab />
    {:else if app.activeTab === 1}
      <WebTab />
    {:else if app.activeTab === 2}
      <CompareTab />
    {:else if app.activeTab === 3}
      <ResultTab />
    {:else if app.activeTab === 4}
      <InspectTab />
    {/if}
  </main>

  <div class="nav-buttons">
    {#if app.activeTab > 0}
      <button class="nav-btn nav-back" onclick={() => app.activeTab--}>
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><path d="M9 2L4 7l5 5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>
        Back
      </button>
    {:else}
      <div></div>
    {/if}
    {#if app.activeTab < 4}
      <button
        class="nav-btn nav-continue"
        disabled={app.activeTab <= 2 && !isTabComplete(app.activeTab)}
        onclick={() => app.activeTab++}
      >
        Continue
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none"><path d="M5 2l5 5-5 5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>
      </button>
    {:else}
      <div></div>
    {/if}
  </div>
</div>

<style>
  :global(*, *::before, *::after) {
    box-sizing: border-box;
  }

  :global(body) {
    margin: 0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
    font-size: 14px;
    line-height: 1.5;
    color: #111827;
    background: #f9fafb;
    -webkit-font-smoothing: antialiased;
  }

  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    padding: 16px 20px;
    overflow: hidden;
  }

  .app-header {
    display: flex;
    align-items: center;
    margin-bottom: 12px;
  }

  .app-title {
    font-size: 20px;
    font-weight: 700;
    margin: 0;
    color: #111827;
    letter-spacing: -0.01em;
  }

  .tab-content {
    flex: 1;
    overflow: auto;
  }

  .nav-buttons {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 0 0;
    flex-shrink: 0;
  }

  .nav-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 8px 20px;
    font-size: 14px;
    font-weight: 500;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .nav-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .nav-back {
    background: #f3f4f6;
    color: #374151;
    border: 1px solid #d1d5db;
  }

  .nav-back:hover:not(:disabled) {
    background: #e5e7eb;
  }

  .nav-continue {
    background: #6366f1;
    color: #fff;
  }

  .nav-continue:hover:not(:disabled) {
    background: #4f46e5;
  }

  @media (prefers-color-scheme: dark) {
    :global(body) {
      color: #f9fafb;
      background: #111827;
    }
    .app-title {
      color: #f9fafb;
    }
    .nav-back {
      background: #374151;
      border-color: #4b5563;
      color: #d1d5db;
    }
    .nav-back:hover:not(:disabled) {
      background: #4b5563;
    }
  }
</style>
