import pixelmatch from 'pixelmatch';

function loadImage(src) {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => resolve(img);
    img.onerror = reject;
    img.src = src.startsWith('data:') ? src : `data:image/png;base64,${src}`;
  });
}

function trimWhitespace(img) {
  const canvas = document.createElement('canvas');
  canvas.width = img.width;
  canvas.height = img.height;
  const ctx = canvas.getContext('2d');
  ctx.drawImage(img, 0, 0);

  const data = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
  const w = canvas.width;
  const h = canvas.height;

  // Find bounds of non-white/non-transparent content
  let top = h, left = w, bottom = 0, right = 0;

  for (let y = 0; y < h; y++) {
    for (let x = 0; x < w; x++) {
      const i = (y * w + x) * 4;
      const r = data[i], g = data[i + 1], b = data[i + 2], a = data[i + 3];
      // Skip fully transparent or near-white pixels
      if (a < 10) continue;
      if (r > 250 && g > 250 && b > 250) continue;
      if (y < top) top = y;
      if (y > bottom) bottom = y;
      if (x < left) left = x;
      if (x > right) right = x;
    }
  }

  // If image is entirely white/transparent, return as-is
  if (bottom < top || right < left) return img;

  const trimmed = document.createElement('canvas');
  trimmed.width = right - left + 1;
  trimmed.height = bottom - top + 1;
  trimmed.getContext('2d').drawImage(
    canvas,
    left, top, trimmed.width, trimmed.height,
    0, 0, trimmed.width, trimmed.height
  );
  return trimmed;
}

function cropImage(img, crop) {
  if (!crop) return img;
  const { x, y, width, height } = crop;
  if (width <= 0 || height <= 0) return img;

  const canvas = document.createElement('canvas');
  canvas.width = width;
  canvas.height = height;
  canvas.getContext('2d').drawImage(
    img,
    x, y, width, height,
    0, 0, width, height
  );
  return canvas;
}

export async function applyCrop(src, crop) {
  const img = await loadImage(src);
  const cropped = cropImage(img, crop);
  if (cropped instanceof HTMLCanvasElement) return cropped.toDataURL('image/png');
  const c = document.createElement('canvas');
  c.width = cropped.width;
  c.height = cropped.height;
  c.getContext('2d').drawImage(cropped, 0, 0);
  return c.toDataURL('image/png');
}

export async function runDiff(figmaBase64, webBase64, thresholdPercent, figmaCrop, webCrop) {
  const figmaRaw = await loadImage(figmaBase64);
  const figmaImg = figmaCrop ? cropImage(figmaRaw, figmaCrop) : trimWhitespace(figmaRaw);
  const webRaw = await loadImage(webBase64);
  const webImg = webCrop ? cropImage(webRaw, webCrop) : trimWhitespace(webRaw);

  // Use the Figma frame as the target size (source of truth)
  const targetWidth = figmaImg.width;
  const targetHeight = figmaImg.height;

  // Draw Figma image at its native size
  const figmaCanvas = document.createElement('canvas');
  figmaCanvas.width = targetWidth;
  figmaCanvas.height = targetHeight;
  figmaCanvas.getContext('2d').drawImage(figmaImg, 0, 0);

  // Scale web capture to match Figma dimensions
  const webCanvas = document.createElement('canvas');
  webCanvas.width = targetWidth;
  webCanvas.height = targetHeight;
  webCanvas.getContext('2d').drawImage(webImg, 0, 0, targetWidth, targetHeight);

  const figmaData = figmaCanvas.getContext('2d').getImageData(0, 0, targetWidth, targetHeight);
  const webData = webCanvas.getContext('2d').getImageData(0, 0, targetWidth, targetHeight);

  const diffCanvas = document.createElement('canvas');
  diffCanvas.width = targetWidth;
  diffCanvas.height = targetHeight;
  const diffCtx = diffCanvas.getContext('2d');
  const diffData = diffCtx.createImageData(targetWidth, targetHeight);

  const numDiffPixels = pixelmatch(
    webData.data,
    figmaData.data,
    diffData.data,
    targetWidth,
    targetHeight,
    { threshold: thresholdPercent / 100 }
  );

  diffCtx.putImageData(diffData, 0, 0);

  const totalPixels = targetWidth * targetHeight;
  const similarity = ((totalPixels - numDiffPixels) / totalPixels * 100).toFixed(2);

  return {
    image: diffCanvas.toDataURL('image/png'),
    similarity: parseFloat(similarity),
    diffPixels: numDiffPixels,
    totalPixels,
  };
}
