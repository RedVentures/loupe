import { figmaColorToCSS, figmaFontWeight } from './property-utils.js';

// --- Helpers ---

function parseRGB(str) {
  if (!str) return null;
  const m = str.match(/rgba?\(\s*(\d+),\s*(\d+),\s*(\d+)(?:,\s*([\d.]+))?\s*\)/);
  if (!m) return null;
  return { r: +m[1], g: +m[2], b: +m[3], a: m[4] != null ? parseFloat(m[4]) : 1 };
}

function colorsClose(a, b, tol = 5) {
  const ca = parseRGB(a);
  const cb = parseRGB(b);
  if (!ca || !cb) return false;
  return Math.abs(ca.r - cb.r) <= tol &&
         Math.abs(ca.g - cb.g) <= tol &&
         Math.abs(ca.b - cb.b) <= tol &&
         Math.abs(ca.a - cb.a) <= 0.1;
}

function pxVal(str) {
  if (str == null) return null;
  const n = parseFloat(String(str));
  return isNaN(n) ? null : n;
}

function pxClose(a, b, tol = 1) {
  const va = pxVal(a);
  const vb = pxVal(b);
  if (va === null || vb === null) return null; // can't compare
  return Math.abs(va - vb) <= tol;
}

function fontFamilyClose(figma, web) {
  if (!figma || !web) return null;
  const f = figma.toLowerCase().replace(/['"]/g, '').trim();
  const w = web.toLowerCase().replace(/['"]/g, '').split(',')[0].trim();
  return f === w || w.includes(f) || f.includes(w);
}

function isTransparent(str) {
  if (!str) return true;
  const c = parseRGB(str);
  if (c && c.a === 0) return true;
  return str === 'rgba(0, 0, 0, 0)' || str === 'transparent';
}

// --- Extract values from raw Figma props ---

function getFigmaColor(raw) {
  if (!raw?.fills?.length) return null;
  const fill = raw.fills.find(f => f.type === 'SOLID');
  if (!fill?.color) return null;
  return figmaColorToCSS(fill.color, fill.opacity ?? 1);
}

function getFigmaTextColor(raw) {
  // For the root node, try textColor first (pulled from child text node)
  if (raw.textColor) return figmaColorToCSS(raw.textColor, raw.textColor.opacity ?? 1);
  // If it's a text node itself, use fills
  if (raw.isText) return getFigmaColor(raw);
  return null;
}

function getFigmaBorderRadius(raw) {
  if (raw.cornerRadius != null && raw.cornerRadius > 0) return raw.cornerRadius;
  if (raw.cornerRadii) {
    const vals = raw.cornerRadii.filter(v => v > 0);
    if (vals.length > 0) return Math.max(...vals);
  }
  return 0;
}

function getFigmaPadding(raw) {
  return {
    top: raw.paddingTop ?? 0,
    right: raw.paddingRight ?? 0,
    bottom: raw.paddingBottom ?? 0,
    left: raw.paddingLeft ?? 0,
  };
}

function getFigmaBorderWidth(raw) {
  return raw.strokeWeight ?? 0;
}

// --- Extract values from raw web computed styles ---

function getWebBorderRadius(raw) {
  const vals = [
    raw['border-top-left-radius'],
    raw['border-top-right-radius'],
    raw['border-bottom-right-radius'],
    raw['border-bottom-left-radius'],
  ].map(pxVal).filter(v => v !== null);
  if (vals.length === 0) return 0;
  return Math.max(...vals);
}

function getWebPadding(raw) {
  return {
    top: pxVal(raw['padding-top']) ?? 0,
    right: pxVal(raw['padding-right']) ?? 0,
    bottom: pxVal(raw['padding-bottom']) ?? 0,
    left: pxVal(raw['padding-left']) ?? 0,
  };
}

function getWebBorderWidth(raw) {
  const vals = [
    raw['border-top-width'],
    raw['border-right-width'],
    raw['border-bottom-width'],
    raw['border-left-width'],
  ].map(pxVal).filter(v => v !== null);
  if (vals.length === 0) return 0;
  return Math.max(...vals);
}

function fmtPx(v) { return `${Math.round(v * 10) / 10}px`; }

function fmtPadding(p) {
  if (p.top === p.right && p.right === p.bottom && p.bottom === p.left) return fmtPx(p.top);
  return `${fmtPx(p.top)} ${fmtPx(p.right)} ${fmtPx(p.bottom)} ${fmtPx(p.left)}`;
}

// --- Run the checklist ---

export function runChecklist(figmaRaw, webRaw) {
  if (!figmaRaw || !webRaw) return null;

  const checks = [];

  // 1. Font family
  const fFont = figmaRaw.fontFamily;
  const wFont = webRaw['font-family'];
  if (fFont && fFont !== 'mixed' && wFont) {
    const match = fontFamilyClose(fFont, wFont);
    checks.push({
      label: 'Font family matches',
      pass: match === true,
      skip: false,
      figmaVal: fFont,
      webVal: wFont.split(',')[0].replace(/['"]/g, '').trim(),
    });
  } else {
    checks.push({ label: 'Font family matches', pass: false, skip: true, note: 'Font not detected on one or both sides' });
  }

  // 2. Font size
  const fSize = figmaRaw.fontSize;
  const wSize = webRaw['font-size'];
  if (fSize != null && fSize !== 'mixed' && wSize) {
    const match = pxClose(fSize, wSize, 1);
    checks.push({
      label: 'Font size matches',
      pass: match === true,
      skip: false,
      figmaVal: fmtPx(fSize),
      webVal: fmtPx(pxVal(wSize)),
    });
  } else {
    checks.push({ label: 'Font size matches', pass: false, skip: true, note: 'Font size not detected on one or both sides' });
  }

  // 3. Font weight
  const fWeight = figmaRaw.fontStyle ? figmaFontWeight(figmaRaw.fontStyle) : null;
  const wWeight = webRaw['font-weight'];
  if (fWeight != null && fWeight !== 'mixed' && wWeight) {
    const match = String(fWeight) === String(wWeight);
    checks.push({
      label: 'Font weight matches',
      pass: match,
      skip: false,
      figmaVal: String(fWeight),
      webVal: String(wWeight),
    });
  } else {
    checks.push({ label: 'Font weight matches', pass: false, skip: true, note: 'Font weight not detected on one or both sides' });
  }

  // 4. Text color
  const fTextColor = getFigmaTextColor(figmaRaw);
  const wTextColor = webRaw['color'];
  if (fTextColor && wTextColor && !isTransparent(fTextColor)) {
    const match = colorsClose(fTextColor, wTextColor, 5);
    checks.push({
      label: 'Text color matches',
      pass: match,
      skip: false,
      figmaVal: fTextColor,
      webVal: wTextColor,
    });
  } else {
    checks.push({ label: 'Text color matches', pass: false, skip: true, note: 'Text color not detected on one or both sides' });
  }

  // 5. Background color
  const fBg = getFigmaColor(figmaRaw);
  const wBg = webRaw['background-color'];
  const bothTransparent = isTransparent(fBg) && isTransparent(wBg);
  if (!bothTransparent && (fBg || wBg)) {
    if (isTransparent(fBg) && !isTransparent(wBg)) {
      checks.push({
        label: 'Background color matches',
        pass: false,
        skip: false,
        figmaVal: 'none',
        webVal: wBg,
        note: 'Figma has no fill but web has a background',
      });
    } else if (!isTransparent(fBg) && isTransparent(wBg)) {
      checks.push({
        label: 'Background color matches',
        pass: false,
        skip: false,
        figmaVal: fBg,
        webVal: 'transparent',
        note: 'Figma has a fill but web background is transparent',
      });
    } else {
      const match = colorsClose(fBg, wBg, 5);
      checks.push({
        label: 'Background color matches',
        pass: match,
        skip: false,
        figmaVal: fBg,
        webVal: wBg,
      });
    }
  } else {
    checks.push({ label: 'Background color matches', pass: true, skip: true });
  }

  // 6. Border radius
  const fRadius = getFigmaBorderRadius(figmaRaw);
  const wRadius = getWebBorderRadius(webRaw);
  if (fRadius > 0 || wRadius > 0) {
    const match = Math.abs(fRadius - wRadius) <= 1;
    checks.push({
      label: 'Border radius matches',
      pass: match,
      skip: false,
      figmaVal: fmtPx(fRadius),
      webVal: fmtPx(wRadius),
    });
  } else {
    checks.push({ label: 'Border radius matches', pass: true, skip: true });
  }

  // 7. Padding
  const fPad = getFigmaPadding(figmaRaw);
  const wPad = getWebPadding(webRaw);
  const anyPad = fPad.top > 0 || fPad.right > 0 || fPad.bottom > 0 || fPad.left > 0 ||
                 wPad.top > 0 || wPad.right > 0 || wPad.bottom > 0 || wPad.left > 0;
  if (anyPad) {
    const match = Math.abs(fPad.top - wPad.top) <= 2 &&
                  Math.abs(fPad.right - wPad.right) <= 2 &&
                  Math.abs(fPad.bottom - wPad.bottom) <= 2 &&
                  Math.abs(fPad.left - wPad.left) <= 2;
    checks.push({
      label: 'Padding matches',
      pass: match,
      skip: false,
      figmaVal: fmtPadding(fPad),
      webVal: fmtPadding(wPad),
    });
  } else {
    checks.push({ label: 'Padding matches', pass: true, skip: true });
  }

  // 8. Border width
  const fBorderW = getFigmaBorderWidth(figmaRaw);
  const wBorderW = getWebBorderWidth(webRaw);
  if (fBorderW > 0 || wBorderW > 0) {
    const match = Math.abs(fBorderW - wBorderW) <= 0.5;
    checks.push({
      label: 'Border width matches',
      pass: match,
      skip: false,
      figmaVal: fmtPx(fBorderW),
      webVal: fmtPx(wBorderW),
    });
  } else {
    checks.push({ label: 'Border width matches', pass: true, skip: true });
  }

  // 9. Opacity
  const fOpacity = figmaRaw.opacity;
  const wOpacity = pxVal(webRaw['opacity']);
  if (fOpacity != null && wOpacity != null && (fOpacity < 1 || wOpacity < 1)) {
    const match = Math.abs(fOpacity - wOpacity) <= 0.05;
    checks.push({
      label: 'Opacity matches',
      pass: match,
      skip: false,
      figmaVal: String(Math.round(fOpacity * 100) / 100),
      webVal: String(Math.round(wOpacity * 100) / 100),
    });
  } else {
    checks.push({ label: 'Opacity matches', pass: true, skip: true });
  }

  return checks;
}
