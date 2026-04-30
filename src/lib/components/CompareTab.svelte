<script>
  import { app, setOutputDir } from '$lib/state.svelte.js';
  import { runDiff, applyCrop } from '$lib/diff.js';
  import CropModal from './CropModal.svelte';

  let running = $state(false);
  let error = $state(null);
  let cropTarget = $state(null);
  let figmaPreview = $state(null);
  let webPreview = $state(null);

  $effect(() => {
    const src = app.figmaImage;
    const crop = app.figmaCrop;
    if (src && crop) {
      applyCrop(src, crop).then(url => figmaPreview = url);
    } else {
      figmaPreview = null;
    }
  });

  $effect(() => {
    const src = app.webCapture;
    const crop = app.webCrop;
    if (src && crop) {
      applyCrop(src, crop).then(url => webPreview = url);
    } else {
      webPreview = null;
    }
  });

  const canRun = $derived(app.figmaImage && app.webCapture && !running);
  const missingMessage = $derived(
    !app.figmaImage && !app.webCapture ? 'Both Figma frame and web capture are missing'
    : !app.figmaImage ? 'Figma frame is missing — go to the Figma tab'
    : !app.webCapture ? 'Web capture is missing — go to the Web tab'
    : null
  );

  async function handleRun() {
    running = true;
    error = null;
    try {
      app.diffResult = await runDiff(app.figmaImage, app.webCapture, app.threshold, app.figmaCrop, app.webCrop);
      app.activeTab = 3;
    } catch (e) {
      error = e.message || 'Comparison failed';
    }
    running = false;
  }
</script>

<div class="compare-tab">
  <p class="tab-desc">Configure and run a pixel-level comparison between the Figma frame and web capture.</p>
  <div class="thumbnails">
    <div class="thumb-card">
      <span class="thumb-label">Figma</span>
      {#if app.figmaImage}
        <div class="thumb-img">
          <img src={figmaPreview || (app.figmaImage.startsWith('data:') ? app.figmaImage : `data:image/png;base64,${app.figmaImage}`)} alt="Figma" />
        </div>
        <button class="btn-crop" onclick={() => cropTarget = 'figma'} title="Crop the Figma image to remove box-shadow whitespace">
          {app.figmaCrop ? 'Re-crop' : 'Crop'}
        </button>
      {:else}
        <div class="thumb-empty">No image</div>
      {/if}
    </div>
    <div class="thumb-card">
      <span class="thumb-label">Web</span>
      {#if app.webCapture}
        <div class="thumb-img">
          <img src={webPreview || app.webCapture} alt="Web" />
        </div>
        <button class="btn-crop" onclick={() => cropTarget = 'web'} title="Crop the web capture">
          {app.webCrop ? 'Re-crop' : 'Crop'}
        </button>
      {:else}
        <div class="thumb-empty">No image</div>
      {/if}
    </div>
  </div>

  <div class="settings">
    <div class="setting-row">
      <label for="threshold">Threshold</label>
      <div class="slider-group">
        <input id="threshold" type="range" min="0" max="100" bind:value={app.threshold} title="How sensitive the diff is — lower values catch smaller color differences" />
        <span class="slider-value">{app.threshold}%</span>
      </div>
    </div>
    <div class="setting-row">
      <label for="output-dir">Output directory</label>
      <input id="output-dir" type="text" bind:value={app.outputDir} oninput={() => setOutputDir(app.outputDir)} placeholder="(optional) path to save diff" class="text-input" title="Folder where diff images are saved" />
    </div>
    <div class="setting-row">
      <label for="output-name">Filename</label>
      <input id="output-name" type="text" bind:value={app.outputFilename} class="text-input" title="Filename pattern for saved diffs — {'{timestamp}'} is replaced with the current time" />
    </div>
  </div>

  {#if missingMessage}
    <p class="missing">{missingMessage}</p>
  {/if}

  {#if error}
    <p class="error">{error}</p>
  {/if}

  {#if app.diffResult && !running}
    <div class="result-preview">
      <span class="similarity">{app.diffResult.similarity}% similar</span>
      <span class="diff-count">{app.diffResult.diffPixels.toLocaleString()} different pixels</span>
    </div>
  {/if}

  <button class="btn-run" onclick={handleRun} disabled={!canRun} title="Run pixelmatch comparison between the two captures">
    {running ? 'Comparing...' : 'Run Comparison'}
  </button>
</div>

{#if cropTarget}
  <CropModal
    image={cropTarget === 'figma'
      ? (app.figmaImage.startsWith('data:') ? app.figmaImage : `data:image/png;base64,${app.figmaImage}`)
      : app.webCapture}
    onApply={(pixels) => {
      if (cropTarget === 'figma') app.figmaCrop = pixels;
      else app.webCrop = pixels;
      cropTarget = null;
    }}
    onCancel={() => cropTarget = null}
  />
{/if}

<style>
  .compare-tab {
    display: flex;
    flex-direction: column;
    gap: 20px;
    max-width: 600px;
    margin: 0 auto;
  }

  .thumbnails {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }

  .thumb-card {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .thumb-label {
    font-size: 12px;
    font-weight: 600;
    color: #6b7280;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .thumb-img {
    height: 160px;
    border: 1px solid #e5e7eb;
    border-radius: 8px;
    overflow: hidden;
    background: #f9fafb;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .thumb-img img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
  }

  .thumb-empty {
    height: 160px;
    border: 2px dashed #d1d5db;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #9ca3af;
    font-size: 14px;
  }

  .settings {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .setting-row {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .setting-row label {
    flex: 0 0 120px;
    font-size: 14px;
    font-weight: 500;
    color: #374151;
  }

  .slider-group {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .slider-group input[type="range"] {
    flex: 1;
    accent-color: #6366f1;
  }

  .slider-value {
    flex: 0 0 40px;
    font-size: 14px;
    font-weight: 500;
    color: #374151;
    text-align: right;
  }

  .text-input {
    flex: 1;
    padding: 6px 10px;
    font-size: 14px;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    background: #fff;
    color: #111827;
    outline: none;
  }

  .text-input:focus {
    border-color: #6366f1;
    box-shadow: 0 0 0 2px rgba(99, 102, 241, 0.15);
  }

  .missing {
    font-size: 13px;
    color: #f59e0b;
    margin: 0;
    text-align: center;
  }

  .error {
    font-size: 13px;
    color: #ef4444;
    margin: 0;
    text-align: center;
  }

  .result-preview {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 16px;
    padding: 12px;
    background: #f0fdf4;
    border-radius: 8px;
  }

  .similarity {
    font-size: 20px;
    font-weight: 700;
    color: #16a34a;
  }

  .diff-count {
    font-size: 13px;
    color: #6b7280;
  }

  .btn-run {
    padding: 12px 24px;
    font-size: 15px;
    font-weight: 600;
    border: none;
    border-radius: 8px;
    background: #6366f1;
    color: #fff;
    cursor: pointer;
    transition: background 0.15s;
  }

  .btn-run:hover:not(:disabled) {
    background: #4f46e5;
  }

  .btn-run:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .tab-desc {
    font-size: 13px;
    color: #6b7280;
    margin: 0 0 12px;
  }

  .btn-crop {
    padding: 4px 10px;
    font-size: 12px;
    font-weight: 500;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    background: #fff;
    color: #6b7280;
    cursor: pointer;
    align-self: flex-start;
    margin-top: 4px;
    transition: background 0.15s, border-color 0.15s;
  }

  .btn-crop:hover {
    background: #f3f4f6;
    border-color: #9ca3af;
  }

  @media (prefers-color-scheme: dark) {
    .tab-desc { color: #9ca3af; }
    .thumb-label { color: #9ca3af; }
    .thumb-img { background: #111827; border-color: #374151; }
    .thumb-empty { border-color: #4b5563; color: #6b7280; }
    .setting-row label { color: #d1d5db; }
    .slider-value { color: #d1d5db; }
    .text-input { background: #1f2937; border-color: #4b5563; color: #f9fafb; }
    .result-preview { background: #052e16; }
    .similarity { color: #4ade80; }
    .btn-crop { background: #1f2937; border-color: #4b5563; color: #9ca3af; }
    .btn-crop:hover { background: #374151; }
  }
</style>
