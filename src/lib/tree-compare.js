import { normalizeFigmaProps, normalizeWebProps, compareProperties, CATEGORIES } from './property-utils.js';

// --- Bounding-box IoU (intersection over union) ---

function bboxIoU(a, b) {
  const x1 = Math.max(a.x, b.x);
  const y1 = Math.max(a.y, b.y);
  const x2 = Math.min(a.x + a.width, b.x + b.width);
  const y2 = Math.min(a.y + a.height, b.y + b.height);
  const inter = Math.max(0, x2 - x1) * Math.max(0, y2 - y1);
  if (inter === 0) return 0;
  const areaA = a.width * a.height;
  const areaB = b.width * b.height;
  return inter / (areaA + areaB - inter);
}

// --- Classify which properties are meaningful for a node ---
// Skip dimensions (almost always differ due to viewport/scaling),
// and skip typography props on non-text nodes, etc.

const SKIP_PROPS = new Set(['width', 'height']);

function meaningfulProps(figmaNorm, webNorm) {
  const fOut = {};
  const wOut = {};
  for (const cat of CATEGORIES) {
    for (const prop of cat.props) {
      if (SKIP_PROPS.has(prop)) continue;
      const fVal = figmaNorm?.[prop] ?? null;
      const wVal = webNorm?.[prop] ?? null;
      // Only include if at least one side has a meaningful value
      if (fVal !== null || wVal !== null) {
        if (fVal !== null) fOut[prop] = fVal;
        if (wVal !== null) wOut[prop] = wVal;
      }
    }
  }
  return { figma: fOut, web: wOut };
}

// --- Collect "visual" nodes from each tree ---
// Figma: TEXT nodes and frames/shapes with visible fills, strokes, or effects
// Web: elements with text content, visible backgrounds, or borders

function collectFigmaNodes(node, arr = []) {
  if (!node) return arr;
  if (!node.bbox || node.bbox.width <= 0 || node.bbox.height <= 0) return arr;

  const isText = node.type === 'TEXT';
  const hasFill = node.props?.fills?.length > 0;
  const hasStroke = node.props?.strokes?.length > 0;
  const hasEffect = node.props?.effects?.length > 0;
  const hasBorderRadius = (node.props?.cornerRadius > 0) || (node.props?.cornerRadii?.some(v => v > 0));

  if (isText || hasFill || hasStroke || hasEffect || hasBorderRadius) {
    arr.push(node);
  }

  if (node.children) {
    for (const child of node.children) {
      collectFigmaNodes(child, arr);
    }
  }
  return arr;
}

function collectWebNodes(node, arr = []) {
  if (!node) return arr;
  if (!node.bbox || node.bbox.width <= 0 || node.bbox.height <= 0) return arr;

  const p = node.props;
  const hasText = !!node.textContent;
  const hasBg = p?.['background-color'] && p['background-color'] !== 'rgba(0, 0, 0, 0)' && p['background-color'] !== 'transparent';
  const hasBorder = p?.['border-top-width'] && p['border-top-width'] !== '0px';
  const hasRadius = p?.['border-top-left-radius'] && p['border-top-left-radius'] !== '0px';
  const hasShadow = p?.['box-shadow'] && p['box-shadow'] !== 'none';

  if (hasText || hasBg || hasBorder || hasRadius || hasShadow) {
    arr.push(node);
  }

  if (node.children) {
    for (const child of node.children) {
      collectWebNodes(child, arr);
    }
  }
  return arr;
}

// --- Match nodes using normalized bounding-box overlap ---

function matchNodes(figmaNodes, webNodes, figmaRoot, webRoot) {
  const fRootW = figmaRoot.bbox.width || 1;
  const fRootH = figmaRoot.bbox.height || 1;
  const wRootW = webRoot.bbox.width || 1;
  const wRootH = webRoot.bbox.height || 1;

  function normF(bbox) {
    return { x: bbox.x / fRootW, y: bbox.y / fRootH, width: bbox.width / fRootW, height: bbox.height / fRootH };
  }
  function normW(bbox) {
    return { x: bbox.x / wRootW, y: bbox.y / wRootH, width: bbox.width / wRootW, height: bbox.height / wRootH };
  }

  // Build a score matrix and use greedy best-first matching
  const candidates = [];
  for (let fi = 0; fi < figmaNodes.length; fi++) {
    const fn = normF(figmaNodes[fi].bbox);
    for (let wi = 0; wi < webNodes.length; wi++) {
      const wn = normW(webNodes[wi].bbox);
      const iou = bboxIoU(fn, wn);
      if (iou >= 0.2) {
        // Bonus for text-to-text match
        const fIsText = figmaNodes[fi].type === 'TEXT';
        const wIsText = !!webNodes[wi].textContent;
        const textBonus = (fIsText && wIsText) ? 0.3 : 0;
        candidates.push({ fi, wi, score: iou + textBonus });
      }
    }
  }

  // Sort by score descending, greedily assign
  candidates.sort((a, b) => b.score - a.score);
  const usedF = new Set();
  const usedW = new Set();
  const pairs = [];

  for (const c of candidates) {
    if (usedF.has(c.fi) || usedW.has(c.wi)) continue;
    usedF.add(c.fi);
    usedW.add(c.wi);
    pairs.push({ figma: figmaNodes[c.fi], web: webNodes[c.wi], iou: Math.min(c.score, 1) });
  }

  // Collect unmatched
  for (let i = 0; i < figmaNodes.length; i++) {
    if (!usedF.has(i)) pairs.push({ figma: figmaNodes[i], web: null, iou: 0 });
  }
  for (let i = 0; i < webNodes.length; i++) {
    if (!usedW.has(i)) pairs.push({ figma: null, web: webNodes[i], iou: 0 });
  }

  return pairs;
}

// --- Compare a matched pair, skipping unhelpful properties ---

function compareNodePair(figmaNode, webNode) {
  const figmaNorm = normalizeFigmaProps(figmaNode.props);
  const webNorm = normalizeWebProps(webNode.props);

  // Filter to meaningful properties only (skip dimensions)
  const { figma: fFiltered, web: wFiltered } = meaningfulProps(figmaNorm, webNorm);
  const comparison = compareProperties(fFiltered, wFiltered);

  // Text content match
  let textMatch = null;
  const fText = figmaNode.props?.textContent?.trim();
  const wText = webNode.textContent?.trim();
  if (fText && wText) {
    textMatch = fText === wText;
  }

  return {
    comparison,
    textMatch,
    figmaLabel: figmaNode.name || figmaNode.type,
    webLabel: webNode.name || webNode.type,
    figmaBbox: figmaNode.bbox,
    webBbox: webNode.bbox,
  };
}

// --- Main: compare two trees ---

export function compareTrees(figmaTree, domTree) {
  if (!figmaTree || !domTree) return null;

  const figmaNodes = collectFigmaNodes(figmaTree);
  const webNodes = collectWebNodes(domTree);

  if (figmaNodes.length === 0 || webNodes.length === 0) return null;

  const pairs = matchNodes(figmaNodes, webNodes, figmaTree, domTree);

  const results = [];
  let totalMatched = 0;
  let totalProps = 0;
  let matchedNodes = 0;
  let unmatchedFigma = 0;
  let unmatchedWeb = 0;

  for (const pair of pairs) {
    if (pair.figma && pair.web) {
      const detail = compareNodePair(pair.figma, pair.web);
      // Only count pairs that actually have comparable properties
      if (detail.comparison.totalProps > 0) {
        results.push({ type: 'matched', iou: pair.iou, ...detail });
        matchedNodes++;
        totalMatched += detail.comparison.totalMatched;
        totalProps += detail.comparison.totalProps;
      }
    } else if (pair.figma) {
      results.push({
        type: 'figma-only',
        figmaLabel: pair.figma.name || pair.figma.type,
        figmaBbox: pair.figma.bbox,
      });
      unmatchedFigma++;
    } else if (pair.web) {
      results.push({
        type: 'web-only',
        webLabel: pair.web.name || pair.web.type,
        webBbox: pair.web.bbox,
      });
      unmatchedWeb++;
    }
  }

  // Sort: matched pairs first (by IoU desc), then unmatched
  results.sort((a, b) => {
    if (a.type === 'matched' && b.type !== 'matched') return -1;
    if (a.type !== 'matched' && b.type === 'matched') return 1;
    return (b.iou || 0) - (a.iou || 0);
  });

  const overallScore = totalProps > 0 ? Math.round(totalMatched / totalProps * 100) : 0;

  return {
    results,
    summary: {
      matchedNodes,
      unmatchedFigma,
      unmatchedWeb,
      totalNodes: figmaNodes.length + webNodes.length,
      totalMatched,
      totalProps,
      overallScore,
    },
  };
}
