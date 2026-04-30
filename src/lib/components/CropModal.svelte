<script>
  import { onMount } from 'svelte';

  let { image, onApply, onCancel } = $props();

  let canvasEl = $state(null);
  let imgLoaded = $state(false);
  let imgNatural = $state({ w: 0, h: 0 });
  let selection = $state(null);
  let dragging = $state(false);
  let startPt = $state(null);

  let scale = 1;
  let offsetX = 0;
  let offsetY = 0;

  onMount(() => {
    const img = new Image();
    img.onload = () => {
      imgNatural = { w: img.naturalWidth, h: img.naturalHeight };
      imgLoaded = true;
      requestAnimationFrame(() => drawCanvas(img));
    };
    img.src = image;
  });

  function getCanvasImg() {
    const img = new Image();
    img.src = image;
    return img;
  }

  function fitImage(canvas, img) {
    const cw = canvas.width;
    const ch = canvas.height;
    scale = Math.min(cw / img.naturalWidth, ch / img.naturalHeight, 1);
    const dw = img.naturalWidth * scale;
    const dh = img.naturalHeight * scale;
    offsetX = (cw - dw) / 2;
    offsetY = (ch - dh) / 2;
    return { dw, dh };
  }

  function drawCanvas(img) {
    if (!canvasEl) return;
    const ctx = canvasEl.getContext('2d');
    const rect = canvasEl.parentElement.getBoundingClientRect();
    canvasEl.width = rect.width;
    canvasEl.height = rect.height;

    ctx.clearRect(0, 0, canvasEl.width, canvasEl.height);

    const { dw, dh } = fitImage(canvasEl, img);
    ctx.drawImage(img, offsetX, offsetY, dw, dh);

    if (selection) {
      const sx = selection.x * scale + offsetX;
      const sy = selection.y * scale + offsetY;
      const sw = selection.width * scale;
      const sh = selection.height * scale;

      ctx.fillStyle = 'rgba(0,0,0,0.5)';
      ctx.fillRect(0, 0, canvasEl.width, canvasEl.height);

      ctx.clearRect(sx, sy, sw, sh);
      ctx.drawImage(img, offsetX, offsetY, dw, dh);
      ctx.save();
      ctx.beginPath();
      ctx.rect(sx, sy, sw, sh);
      ctx.clip();
      ctx.drawImage(img, offsetX, offsetY, dw, dh);
      ctx.restore();

      ctx.strokeStyle = '#6366f1';
      ctx.lineWidth = 2;
      ctx.setLineDash([6, 3]);
      ctx.strokeRect(sx, sy, sw, sh);
      ctx.setLineDash([]);
    }
  }

  function canvasToImage(cx, cy) {
    const ix = (cx - offsetX) / scale;
    const iy = (cy - offsetY) / scale;
    return {
      x: Math.max(0, Math.min(ix, imgNatural.w)),
      y: Math.max(0, Math.min(iy, imgNatural.h)),
    };
  }

  function handlePointerDown(e) {
    if (!canvasEl) return;
    const rect = canvasEl.getBoundingClientRect();
    const cx = e.clientX - rect.left;
    const cy = e.clientY - rect.top;
    startPt = canvasToImage(cx, cy);
    dragging = true;
    selection = null;
    canvasEl.setPointerCapture(e.pointerId);
  }

  function handlePointerMove(e) {
    if (!dragging || !startPt || !canvasEl) return;
    const rect = canvasEl.getBoundingClientRect();
    const cx = e.clientX - rect.left;
    const cy = e.clientY - rect.top;
    const end = canvasToImage(cx, cy);

    const x = Math.min(startPt.x, end.x);
    const y = Math.min(startPt.y, end.y);
    const width = Math.abs(end.x - startPt.x);
    const height = Math.abs(end.y - startPt.y);

    if (width > 2 && height > 2) {
      selection = {
        x: Math.round(x),
        y: Math.round(y),
        width: Math.round(width),
        height: Math.round(height),
      };
    }

    drawCanvas(getCanvasImg());
  }

  function handlePointerUp() {
    dragging = false;
    startPt = null;
  }

  function handleApply() {
    onApply(selection);
  }

  function handleReset() {
    onApply(null);
  }

  function handleBackdropClick(e) {
    if (e.target === e.currentTarget) onCancel();
  }

  function handleKeydown(e) {
    if (e.key === 'Escape') onCancel();
  }

  $effect(() => {
    if (imgLoaded && canvasEl) {
      drawCanvas(getCanvasImg());
    }
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="modal-backdrop" onclick={handleBackdropClick}>
  <div class="modal">
    <div class="modal-header">
      <h2>Crop Image</h2>
      <span class="crop-hint">Click and drag to select a crop region</span>
      <button class="btn-close" onclick={onCancel} title="Close">&times;</button>
    </div>
    <div class="cropper-container">
      <canvas
        bind:this={canvasEl}
        onpointerdown={handlePointerDown}
        onpointermove={handlePointerMove}
        onpointerup={handlePointerUp}
        class="crop-canvas"
      ></canvas>
    </div>
    {#if selection}
      <div class="crop-info">
        {selection.width} &times; {selection.height}px (from {selection.x}, {selection.y})
      </div>
    {/if}
    <div class="modal-footer">
      <button class="btn btn-secondary" onclick={handleReset}>Reset (no crop)</button>
      <div class="footer-right">
        <button class="btn btn-secondary" onclick={onCancel}>Cancel</button>
        <button class="btn btn-primary" onclick={handleApply} disabled={!selection}>Apply Crop</button>
      </div>
    </div>
  </div>
</div>

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal {
    background: #fff;
    border-radius: 12px;
    width: min(90vw, 720px);
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-bottom: 1px solid #e5e7eb;
    gap: 12px;
  }

  .modal-header h2 {
    font-size: 16px;
    font-weight: 600;
    color: #111827;
    margin: 0;
  }

  .crop-hint {
    flex: 1;
    font-size: 13px;
    color: #9ca3af;
  }

  .btn-close {
    background: none;
    border: none;
    font-size: 24px;
    color: #6b7280;
    cursor: pointer;
    padding: 0 4px;
    line-height: 1;
  }

  .btn-close:hover {
    color: #111827;
  }

  .cropper-container {
    position: relative;
    width: 100%;
    height: 420px;
    background: #111827;
  }

  .crop-canvas {
    width: 100%;
    height: 100%;
    cursor: crosshair;
    display: block;
  }

  .crop-info {
    padding: 8px 20px;
    font-size: 12px;
    color: #6b7280;
    border-top: 1px solid #e5e7eb;
    font-variant-numeric: tabular-nums;
  }

  .modal-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 20px;
    border-top: 1px solid #e5e7eb;
    gap: 8px;
  }

  .footer-right {
    display: flex;
    gap: 8px;
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

  @media (prefers-color-scheme: dark) {
    .modal { background: #1f2937; }
    .modal-header { border-color: #374151; }
    .modal-header h2 { color: #f9fafb; }
    .crop-hint { color: #6b7280; }
    .btn-close { color: #9ca3af; }
    .btn-close:hover { color: #f9fafb; }
    .crop-info { color: #9ca3af; border-color: #374151; }
    .modal-footer { border-color: #374151; }
    .btn-secondary { background: #374151; border-color: #4b5563; color: #d1d5db; }
    .btn-secondary:hover { background: #4b5563; }
  }
</style>
