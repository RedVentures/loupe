<script>
  import { app } from '$lib/state.svelte.js';
  import { applyCrop } from '$lib/diff.js';
  import { invoke } from '@tauri-apps/api/core';

  let saving = $state(false);
  let saveMessage = $state('');
  let croppedFigma = $state(null);
  let croppedWeb = $state(null);

  const figmaSrc = $derived(
    app.figmaImage ? (app.figmaImage.startsWith('data:') ? app.figmaImage : `data:image/png;base64,${app.figmaImage}`) : null
  );

  $effect(() => {
    const src = app.figmaImage;
    const crop = app.figmaCrop;
    if (src && crop) {
      applyCrop(src, crop).then(url => croppedFigma = url);
    } else {
      croppedFigma = null;
    }
  });

  $effect(() => {
    const src = app.webCapture;
    const crop = app.webCrop;
    if (src && crop) {
      applyCrop(src, crop).then(url => croppedWeb = url);
    } else {
      croppedWeb = null;
    }
  });

  async function handleDownload() {
    if (!app.diffResult) return;

    const filename = app.outputFilename.replace('{timestamp}', Date.now());
    const dir = app.outputDir || '.';
    const fullPath = `${dir}/${filename}`;

    saving = true;
    saveMessage = '';
    try {
      await invoke('save_image', { path: fullPath, data: app.diffResult.image });
      saveMessage = `Saved to ${fullPath}`;
    } catch (e) {
      saveMessage = `Error: ${e}`;
    }
    saving = false;
  }
</script>

<div class="result-tab">
  {#if !app.diffResult}
    <div class="empty-state">
      <div class="icon">
        <svg width="48" height="48" viewBox="0 0 48 48" fill="none">
          <rect x="8" y="8" width="32" height="32" rx="4" stroke="currentColor" stroke-width="2" fill="none"/>
          <path d="M18 24 L22 28 L30 20" stroke="currentColor" stroke-width="2.5" fill="none" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </div>
      <h2>No results yet</h2>
      <p>Run a comparison in the previous tab to see results here.</p>
      <button class="btn-link" onclick={() => app.activeTab = 2} title="Go back to the Compare tab to run a comparison">Go to Compare</button>
    </div>
  {:else}
    <div class="results">
      <p class="tab-desc">Comparison results — view the diff as a heatmap, side by side, or as an overlay.</p>
      <div class="stats-bar">
        <span class="similarity">{app.diffResult.similarity}% similar</span>
        <span class="diff-count">{app.diffResult.diffPixels.toLocaleString()} / {app.diffResult.totalPixels.toLocaleString()} pixels differ</span>
      </div>

      <div class="view-toggle">
        <button class:active={app.viewMode === 'heatmap'} onclick={() => app.viewMode = 'heatmap'} title="Show only the pixels that differ">Heatmap</button>
        <button class:active={app.viewMode === 'sidebyside'} onclick={() => app.viewMode = 'sidebyside'} title="Show web capture and Figma frame next to each other">Side by Side</button>
        <button class:active={app.viewMode === 'overlay'} onclick={() => app.viewMode = 'overlay'} title="Layer the Figma frame over the web capture with adjustable opacity">Overlay</button>
      </div>

      <div class="view-area">
        {#if app.viewMode === 'heatmap'}
          <div class="single-view">
            <img src={app.diffResult.image} alt="Diff heatmap" />
          </div>
        {:else if app.viewMode === 'sidebyside'}
          <div class="side-by-side">
            <div class="side">
              <span class="side-label">Web</span>
              <img src={croppedWeb || app.webCapture} alt="Web capture" />
            </div>
            <div class="side">
              <span class="side-label">Figma</span>
              <img src={croppedFigma || figmaSrc} alt="Figma frame" />
            </div>
          </div>
        {:else if app.viewMode === 'overlay'}
          <div class="overlay-view">
            <div class="overlay-container">
              <img class="overlay-layer" src={croppedWeb || app.webCapture} alt="Web capture" />
              <img
                class="overlay-layer"
                src={croppedFigma || figmaSrc}
                alt="Figma frame"
                style="opacity: {app.overlayOpacity}"
              />
            </div>
            <div class="opacity-control">
              <label for="overlay-opacity">Figma opacity</label>
              <input id="overlay-opacity" type="range" min="0" max="1" step="0.01" bind:value={app.overlayOpacity} title="Adjust the Figma frame transparency" />
              <span>{Math.round(app.overlayOpacity * 100)}%</span>
            </div>
          </div>
        {/if}
      </div>

      <div class="actions">
        <button class="btn-download" onclick={handleDownload} disabled={saving} title="Save the diff heatmap image to disk">
          {saving ? 'Saving...' : 'Download Diff'}
        </button>
        {#if saveMessage}
          <span class="save-msg" class:error={saveMessage.startsWith('Error')}>{saveMessage}</span>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .result-tab {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 400px;
  }

  .empty-state {
    text-align: center;
    color: #6b7280;
    margin: auto;
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
    margin: 0 0 16px;
  }

  .btn-link {
    padding: 8px 16px;
    font-size: 14px;
    font-weight: 500;
    border: 1px solid #6366f1;
    border-radius: 8px;
    background: transparent;
    color: #6366f1;
    cursor: pointer;
  }

  .btn-link:hover {
    background: rgba(99, 102, 241, 0.05);
  }

  .results {
    display: flex;
    flex-direction: column;
    gap: 12px;
    height: 100%;
  }

  .stats-bar {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 10px 16px;
    background: #f0fdf4;
    border-radius: 8px;
  }

  .similarity {
    font-size: 18px;
    font-weight: 700;
    color: #16a34a;
  }

  .diff-count {
    font-size: 13px;
    color: #6b7280;
  }

  .view-toggle {
    display: flex;
    gap: 2px;
    background: #e5e7eb;
    border-radius: 8px;
    padding: 3px;
    align-self: flex-start;
  }

  .view-toggle button {
    padding: 6px 14px;
    font-size: 13px;
    font-weight: 500;
    border: none;
    border-radius: 6px;
    background: transparent;
    color: #6b7280;
    cursor: pointer;
  }

  .view-toggle button.active {
    background: #fff;
    color: #111827;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.08);
  }

  .view-area {
    flex: 1;
    overflow: auto;
    border: 1px solid #e5e7eb;
    border-radius: 8px;
    background: #f9fafb;
  }

  .single-view {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 16px;
    min-height: 200px;
  }

  .single-view img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
  }

  .side-by-side {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1px;
    background: #e5e7eb;
    min-height: 200px;
  }

  .side {
    background: #f9fafb;
    padding: 12px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
  }

  .side-label {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    color: #9ca3af;
    letter-spacing: 0.05em;
  }

  .side img {
    max-width: 100%;
    object-fit: contain;
  }

  .overlay-view {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 16px;
  }

  .overlay-container {
    display: grid;
    justify-items: start;
    align-items: start;
  }

  .overlay-layer {
    grid-area: 1 / 1;
    display: block;
    max-width: 100%;
    height: auto;
  }

  .opacity-control {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 13px;
    color: #6b7280;
  }

  .opacity-control input[type="range"] {
    width: 180px;
    accent-color: #6366f1;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .btn-download {
    padding: 8px 20px;
    font-size: 14px;
    font-weight: 500;
    border: none;
    border-radius: 8px;
    background: #6366f1;
    color: #fff;
    cursor: pointer;
  }

  .btn-download:hover:not(:disabled) {
    background: #4f46e5;
  }

  .btn-download:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .save-msg {
    font-size: 13px;
    color: #16a34a;
  }

  .save-msg.error {
    color: #ef4444;
  }

  .tab-desc {
    font-size: 13px;
    color: #6b7280;
    margin: 0 0 12px;
  }

  @media (prefers-color-scheme: dark) {
    .tab-desc { color: #9ca3af; }
    .empty-state h2 { color: #e5e7eb; }
    .empty-state .icon { color: #4b5563; }
    .btn-link { border-color: #818cf8; color: #818cf8; }
    .stats-bar { background: #052e16; }
    .similarity { color: #4ade80; }
    .view-toggle { background: #374151; }
    .view-toggle button { color: #9ca3af; }
    .view-toggle button.active { background: #1f2937; color: #f9fafb; }
    .view-area { border-color: #374151; background: #111827; }
    .side-by-side { background: #374151; }
    .side { background: #111827; }
  }
</style>
