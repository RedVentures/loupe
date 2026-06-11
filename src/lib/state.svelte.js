/** @param {string} key @param {string} fallback */
function loadStored(key, fallback) {
  if (typeof localStorage === 'undefined') return fallback;
  return localStorage.getItem(`loupe_${key}`) ?? fallback;
}

/** @param {string} key @param {string} value */
function store(key, value) {
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem(`loupe_${key}`, value);
  }
}

/** @type {{ activeTab: number, figmaImage: string | null, webCapture: string | null, diffResult: string | null, threshold: number, outputDir: string, outputDirInitialized: boolean, outputFilename: string, lastUrl: string, viewMode: string, overlayOpacity: number, browserOpen: boolean, viewport: string, figmaProperties: any, webProperties: any, figmaCrop: any, webCrop: any }} */
export const app = $state({
  activeTab: 0,
  figmaImage: null,
  webCapture: null,
  diffResult: null,
  threshold: 10,
  outputDir: loadStored('outputDir', ''),
  outputDirInitialized: false,
  outputFilename: 'diff-{timestamp}.png',
  lastUrl: loadStored('lastUrl', 'http://localhost:3000'),
  viewMode: 'heatmap',
  overlayOpacity: 0.5,
  browserOpen: false,
  viewport: 'desktop',
  figmaProperties: null,
  webProperties: null,
  figmaCrop: null,
  webCrop: null,
});

export async function initOutputDir() {
  // Only set the default if the user hasn't explicitly chosen one
  if (app.outputDir) {
    app.outputDirInitialized = true;
    return;
  }
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    const defaultDir = await invoke('get_default_output_dir');
    app.outputDir = defaultDir;
    store('outputDir', defaultDir);
    app.outputDirInitialized = true;
  } catch (e) {
    console.error('Failed to get default output dir:', e);
    app.outputDirInitialized = true;
  }
}

/** @param {string} url */
export function setLastUrl(url) {
  app.lastUrl = url;
  store('lastUrl', url);
}

/** @param {string} dir */
export function setOutputDir(dir) {
  app.outputDir = dir;
  store('outputDir', dir);
}

export function clearFigma() {
  app.figmaImage = null;
  app.figmaProperties = null;
  app.figmaCrop = null;
}

export function clearWebCapture() {
  app.webCapture = null;
  app.webProperties = null;
  app.webCrop = null;
}

export function clearDiff() {
  app.diffResult = null;
}

export function clearAll() {
  app.figmaImage = null;
  app.figmaProperties = null;
  app.webCapture = null;
  app.webProperties = null;
  app.diffResult = null;
  app.figmaCrop = null;
  app.webCrop = null;
}

/** @param {number} index */
export function isTabComplete(index) {
  if (index === 0) return app.figmaImage !== null;
  if (index === 1) return app.webCapture !== null;
  if (index === 2) return app.diffResult !== null;
  if (index === 3) return app.diffResult !== null;
  if (index === 4) return app.figmaProperties !== null && app.webProperties !== null;
  return false;
}
