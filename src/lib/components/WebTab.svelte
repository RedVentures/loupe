<script>
  import { app, setLastUrl, clearWebCapture } from '$lib/state.svelte.js';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { onMount } from 'svelte';

  const PRESETS = {
    mobile:  { width: 375,  height: 812  },
    tablet:  { width: 768,  height: 1024 },
    desktop: { width: 1200, height: 800  },
  };

  let customW = $state(1440);
  let customH = $state(900);

  const activeViewport = $derived(
    app.viewport === 'custom'
      ? { width: customW, height: customH }
      : PRESETS[app.viewport] || PRESETS.desktop
  );

  let urlInput = $state(app.lastUrl);
  let loading = $state(false);
  let capturing = $state(false);

  onMount(() => {
    const unlisten = listen('web-capture', (event) => {
      app.webCapture = event.payload;
    });
    return () => unlisten.then(fn => fn());
  });

  async function handleOpen() {
    if (!urlInput.trim()) return;
    loading = true;
    capturing = false;
    try {
      setLastUrl(urlInput.trim());
      const vp = activeViewport;
      await invoke('open_browser', { url: urlInput.trim(), width: vp.width, height: vp.height });
      app.browserOpen = true;
    } catch (e) {
      console.error('Failed to open browser:', e);
    }
    loading = false;
  }

  async function handleClose() {
    try {
      await invoke('close_browser');
      app.browserOpen = false;
      capturing = false;
    } catch (e) {
      console.error('Failed to close browser:', e);
    }
  }

  async function toggleCapture() {
    try {
      if (capturing) {
        await invoke('stop_capture');
        capturing = false;
      } else {
        await invoke('start_capture');
        capturing = true;
      }
    } catch (e) {
      console.error('Capture toggle error:', e);
    }
  }

  function handleClear() {
    clearWebCapture();
  }

  function handleKeydown(e) {
    if (e.key === 'Enter') handleOpen();
  }
</script>

<div class="web-tab">
  <div class="url-bar">
    <input
      type="text"
      bind:value={urlInput}
      onkeydown={handleKeydown}
      placeholder="Enter URL (e.g. http://localhost:6006)"
      class="url-input"
      title="URL of the page containing the component to capture"
    />
    {#if app.browserOpen}
      <button class="btn btn-secondary" onclick={handleClose} title="Close the browser window">Close</button>
      <button
        class="btn"
        class:btn-capture-active={capturing}
        class:btn-capture={!capturing}
        onclick={toggleCapture}
        title={capturing ? 'Deactivate the DOM inspector' : 'Activate the DOM inspector to select and capture an element'}
      >
        {capturing ? 'Stop Capture' : 'Start Capture'}
      </button>
    {/if}
    <button class="btn btn-primary" onclick={handleOpen} disabled={loading || !urlInput.trim()} title={app.browserOpen ? 'Reload the browser window with this URL' : 'Open the URL in a browser window'}>
      {loading ? 'Opening...' : app.browserOpen ? 'Reload' : 'Open Browser'}
    </button>
  </div>

  <div class="viewport-bar">
    <span class="viewport-label">Viewport</span>
    <div class="viewport-presets">
      {#each Object.entries(PRESETS) as [key, vp]}
        <button
          class="vp-btn"
          class:vp-active={app.viewport === key}
          onclick={() => app.viewport = key}
          title="{vp.width} × {vp.height}"
        >
          {key.charAt(0).toUpperCase() + key.slice(1)}
        </button>
      {/each}
      <button
        class="vp-btn"
        class:vp-active={app.viewport === 'custom'}
        onclick={() => app.viewport = 'custom'}
        title="Set a custom viewport size"
      >
        Custom
      </button>
    </div>
    {#if app.viewport === 'custom'}
      <div class="viewport-custom">
        <input type="number" class="vp-input" bind:value={customW} min="320" max="3840" title="Width in pixels" />
        <span class="vp-x">&times;</span>
        <input type="number" class="vp-input" bind:value={customH} min="320" max="2160" title="Height in pixels" />
        <span class="vp-unit">px</span>
      </div>
    {:else}
      <span class="viewport-dims">{activeViewport.width} &times; {activeViewport.height}px</span>
    {/if}
  </div>

  <p class="tab-desc">Open a browser window, navigate to your component, then capture an element.</p>

  <div class="content">
    {#if !app.webCapture}
      <div class="empty-state">
        <div class="icon">
          <svg width="48" height="48" viewBox="0 0 48 48" fill="none">
            <rect x="6" y="8" width="36" height="28" rx="3" stroke="currentColor" stroke-width="2" fill="none"/>
            <line x1="6" y1="16" x2="42" y2="16" stroke="currentColor" stroke-width="2"/>
            <circle cx="12" cy="12" r="1.5" fill="currentColor"/>
            <circle cx="17" cy="12" r="1.5" fill="currentColor"/>
            <circle cx="22" cy="12" r="1.5" fill="currentColor"/>
            <rect x="6" y="36" width="36" height="4" rx="1" stroke="currentColor" stroke-width="2" fill="none"/>
          </svg>
        </div>
        {#if !app.browserOpen}
          <h2>Capture a web element</h2>
          <p>Enter a URL above and click <strong>Open Browser</strong> to open a browser window. Browse to the page you want, then click <strong>Start Capture</strong> to enable element selection.</p>
        {:else if !capturing}
          <h2>Browser is open</h2>
          <p>Navigate to the element you want to capture, then click <strong>Start Capture</strong> above. If the element is inside an iframe, click the iframe first to navigate into it.</p>
        {:else}
          <h2>Capture mode active</h2>
          <p>Hover over any element to highlight it, then <strong>click to capture</strong>. If you see an iframe, clicking it will navigate into it so you can capture elements inside.</p>
          <div class="status pulse">
            <span class="dot"></span>
            Waiting for capture...
          </div>
        {/if}
      </div>
    {:else}
      <div class="preview">
        <div class="preview-header">
          <h2>Web Capture</h2>
          <button class="btn-clear" onclick={handleClear} title="Clear the captured image and start over">Clear</button>
        </div>
        <div class="preview-image">
          <img src={app.webCapture} alt="Web element capture" />
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .web-tab {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 400px;
  }

  .url-bar {
    display: flex;
    gap: 8px;
    margin-bottom: 16px;
    flex-wrap: wrap;
  }

  .url-input {
    flex: 1;
    min-width: 200px;
    padding: 8px 12px;
    font-size: 14px;
    border: 1px solid #d1d5db;
    border-radius: 8px;
    background: #fff;
    color: #111827;
    outline: none;
  }

  .url-input:focus {
    border-color: #6366f1;
    box-shadow: 0 0 0 2px rgba(99, 102, 241, 0.15);
  }

  .btn {
    padding: 8px 16px;
    font-size: 14px;
    font-weight: 500;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    white-space: nowrap;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-primary {
    background: #6366f1;
    color: #fff;
  }

  .btn-primary:hover:not(:disabled) {
    background: #4f46e5;
  }

  .btn-secondary {
    background: #f3f4f6;
    color: #374151;
    border: 1px solid #d1d5db;
  }

  .btn-secondary:hover {
    background: #e5e7eb;
  }

  .btn-capture {
    background: #f59e0b;
    color: #fff;
  }

  .btn-capture:hover {
    background: #d97706;
  }

  .btn-capture-active {
    background: #ef4444;
    color: #fff;
    animation: capture-pulse 2s ease-in-out infinite;
  }

  .btn-capture-active:hover {
    background: #dc2626;
  }

  @keyframes capture-pulse {
    0%, 100% { box-shadow: 0 0 0 0 rgba(239, 68, 68, 0.4); }
    50% { box-shadow: 0 0 0 4px rgba(239, 68, 68, 0); }
  }

  .content {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
  }

  .empty-state {
    text-align: center;
    color: #6b7280;
  }

  .empty-state .icon {
    color: #d1d5db;
    margin-bottom: 16px;
  }

  .empty-state h2 {
    font-size: 18px;
    font-weight: 600;
    color: #374151;
    margin: 0 0 8px;
  }

  .empty-state p {
    font-size: 14px;
    margin: 0 0 20px;
    max-width: 420px;
    line-height: 1.5;
  }

  .status {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    padding: 6px 14px;
    background: #f3f4f6;
    border-radius: 20px;
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #6366f1;
  }

  .status.pulse .dot {
    animation: pulse 1.5s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.3; }
  }

  .preview {
    width: 100%;
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .preview-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 12px;
  }

  .preview-header h2 {
    font-size: 16px;
    font-weight: 600;
    color: #374151;
    margin: 0;
  }

  .btn-clear {
    padding: 4px 12px;
    font-size: 13px;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    background: #fff;
    color: #6b7280;
    cursor: pointer;
  }

  .btn-clear:hover {
    background: #f9fafb;
    border-color: #9ca3af;
  }

  .preview-image {
    flex: 1;
    overflow: auto;
    background: #f9fafb;
    border: 1px solid #e5e7eb;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 16px;
  }

  .preview-image img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    border-radius: 4px;
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.08);
  }

  .tab-desc {
    font-size: 13px;
    color: #6b7280;
    margin: 0 0 12px;
  }

  /* Viewport bar */
  .viewport-bar {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 10px;
    flex-wrap: wrap;
  }

  .viewport-label {
    font-size: 12px;
    font-weight: 600;
    color: #6b7280;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .viewport-presets {
    display: flex;
    gap: 4px;
  }

  .vp-btn {
    padding: 4px 10px;
    font-size: 12px;
    font-weight: 500;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    background: #fff;
    color: #6b7280;
    cursor: pointer;
  }

  .vp-btn:hover {
    background: #f3f4f6;
    border-color: #9ca3af;
  }

  .vp-btn.vp-active {
    background: #6366f1;
    color: #fff;
    border-color: #6366f1;
  }

  .viewport-dims {
    font-size: 13px;
    font-family: 'SF Mono', Monaco, Consolas, monospace;
    color: #374151;
    background: #f3f4f6;
    padding: 3px 8px;
    border-radius: 4px;
  }

  .viewport-custom {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .vp-input {
    width: 64px;
    padding: 3px 6px;
    font-size: 13px;
    font-family: 'SF Mono', Monaco, Consolas, monospace;
    border: 1px solid #d1d5db;
    border-radius: 4px;
    background: #fff;
    color: #374151;
    text-align: center;
    outline: none;
    -moz-appearance: textfield;
  }

  .vp-input::-webkit-inner-spin-button,
  .vp-input::-webkit-outer-spin-button {
    -webkit-appearance: none;
    margin: 0;
  }

  .vp-input:focus {
    border-color: #6366f1;
    box-shadow: 0 0 0 2px rgba(99, 102, 241, 0.15);
  }

  .vp-x {
    font-size: 13px;
    color: #9ca3af;
  }

  .vp-unit {
    font-size: 12px;
    color: #9ca3af;
  }

  @media (prefers-color-scheme: dark) {
    .tab-desc { color: #9ca3af; }
    .url-input { background: #1f2937; border-color: #4b5563; color: #f9fafb; }
    .url-input:focus { border-color: #6366f1; }
    .btn-secondary { background: #374151; border-color: #4b5563; color: #d1d5db; }
    .btn-secondary:hover { background: #4b5563; }
    .empty-state h2 { color: #e5e7eb; }
    .empty-state .icon { color: #4b5563; }
    .status { background: #374151; }
    .preview-header h2 { color: #e5e7eb; }
    .btn-clear { background: #1f2937; border-color: #4b5563; color: #9ca3af; }
    .btn-clear:hover { background: #374151; }
    .preview-image { background: #111827; border-color: #374151; }
    .viewport-label { color: #9ca3af; }
    .vp-btn { background: #374151; border-color: #4b5563; color: #9ca3af; }
    .vp-btn:hover { background: #4b5563; }
    .vp-btn.vp-active { background: #6366f1; color: #fff; border-color: #6366f1; }
    .viewport-dims { background: #1f2937; color: #d1d5db; }
    .vp-input { background: #1f2937; border-color: #4b5563; color: #d1d5db; }
    .vp-input:focus { border-color: #6366f1; }
  }
</style>
