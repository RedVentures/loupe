# Loupe — Internal Documentation

> Pixel-level visual comparison between Figma designs and live web implementations.

Loupe is a desktop app that lets you capture a frame from Figma and an element from a live web page, then run a pixel-level diff between them. It also compares extracted CSS properties side by side. Use it to catch visual regressions and verify that implementations match designs.

---

## Table of Contents

1. [Installation](#installation)
2. [Figma Plugin Setup](#figma-plugin-setup)
3. [Walkthrough: Your First Comparison](#walkthrough-your-first-comparison)
   - [Step 1 — Figma Tab](#step-1--figma-tab)
   - [Step 2 — Web Tab](#step-2--web-tab)
   - [Step 3 — Compare Tab](#step-3--compare-tab)
   - [Step 4 — Result Tab](#step-4--result-tab)
   - [Step 5 — Inspect Tab](#step-5--inspect-tab)
4. [Inspector Panel (In-Browser)](#inspector-panel-in-browser)
   - [DOM Tree](#dom-tree)
   - [Viewport Switcher](#viewport-switcher)
   - [Capturing an Element](#capturing-an-element)
5. [Comparison Settings](#comparison-settings)
6. [Result View Modes](#result-view-modes)
7. [Property Inspection & Export](#property-inspection--export)
8. [Output & Saving](#output--saving)
9. [Troubleshooting](#troubleshooting)
10. [Releasing New Versions](#releasing-new-versions)

---

## Installation

### Download a Pre-Built Release

Go to the **Releases** page on the internal GitHub repo and download the appropriate installer for your platform:

| Platform         | File Type         |
|------------------|-------------------|
| macOS (Apple Silicon) | `.dmg`        |
| Windows          | `.msi` or `.exe`  |
| Linux            | `.deb` or `.AppImage` |

Open the installer and follow the prompts. On macOS, drag the app to your Applications folder.

> **📸 SCREENSHOT NEEDED:** The GitHub Releases page showing available downloads for each platform.

### Build from Source (Optional)

If you need to build locally:

**Prerequisites:**
- Node.js 18+
- Rust (install via [rustup.rs](https://rustup.rs/))

```sh
git clone <repo-url>
cd loupe
npm install
npm run tauri dev      # development mode
npm run tauri build    # production build (output in src-tauri/target/release/bundle/)
```

---

## Figma Plugin Setup

The Figma plugin is included in the repo and must be loaded locally — it is **not** published to the Figma Community.

1. Open **Figma** (the desktop app)
2. Go to **Plugins → Development → Import plugin from manifest...**
3. Navigate to the cloned repo and select `figma-plugin/manifest.json`
4. The plugin now appears under **Plugins → Development → Loupe**

> **📸 SCREENSHOT NEEDED:** Figma's "Import plugin from manifest" dialog with the `figma-plugin/manifest.json` file selected.

> **📸 SCREENSHOT NEEDED:** The Loupe plugin appearing in the Figma Plugins → Development menu.

**Important:** The Loupe desktop app must be running before you use the plugin. The plugin sends data to the app over `localhost:7700`.

---

## Walkthrough: Your First Comparison

Loupe uses a five-tab workflow. Each tab has a numbered step indicator that shows a green checkmark (✓) when that step is complete.

> **📸 SCREENSHOT NEEDED:** The full Loupe app window showing the tab navigation bar with all five tabs (Figma, Web, Compare, Result, Inspect).

### Step 1 — Figma Tab

This tab receives a design frame from Figma via the companion plugin.

1. Make sure **Loupe is running** — the tab shows "Listening on port 7700..." with a pulsing indicator
2. In **Figma**, select the frame you want to compare
3. Run the Loupe plugin: **Plugins → Development → Loupe → Send to Loupe**
4. In the plugin UI, click **Send to Loupe**
5. The frame appears in the Figma tab as a preview

> **📸 SCREENSHOT NEEDED:** The Figma tab in its "waiting" state, showing the listening indicator.

> **📸 SCREENSHOT NEEDED:** The Figma plugin UI inside Figma, with a frame selected and the "Send to Loupe" button visible.

> **📸 SCREENSHOT NEEDED:** The Figma tab after receiving a frame, showing the preview image.

**Tips:**
- The plugin exports at **2× scale** for high-fidelity comparison
- CSS properties (dimensions, colors, typography, borders, spacing, effects) are extracted automatically from the Figma node and sent along with the image
- Click **Clear** to discard the frame and start over

### Step 2 — Web Tab

This tab opens a browser window and lets you capture a specific DOM element.

1. Enter the URL of your web page (e.g. `http://localhost:3000`) in the URL bar
2. Click **Open Browser** — a browser window opens
3. Navigate to the page/component you want to compare
4. Click **Start Capture** — this activates the inspector (see [Inspector Panel](#inspector-panel-in-browser) below)
5. In the inspector, select an element from the DOM tree and click **Capture Selected**
6. The captured element appears back in the Web tab

> **📸 SCREENSHOT NEEDED:** The Web tab with a URL entered and the Open Browser / Start Capture buttons visible.

> **📸 SCREENSHOT NEEDED:** The Web tab after a successful capture, showing the preview of the captured element.

**Tips:**
- The browser window remembers your last URL between sessions
- If the element is inside an **iframe**, click the iframe in the DOM tree to expand into it
- Click **Close** to close the browser window, or **Reload** to reload it
- Click **Clear** to discard the capture and start over
- Box shadows are automatically stripped from captures to prevent 2× scaling artifacts from distorting comparisons

### Step 3 — Compare Tab

This tab configures and runs the pixel-level diff between the Figma frame and web capture.

1. Review the two thumbnail previews (Figma and Web) at the top
2. Adjust settings if needed (see [Comparison Settings](#comparison-settings))
3. Click **Run Comparison**
4. The app automatically advances to the Result tab

> **📸 SCREENSHOT NEEDED:** The Compare tab showing both thumbnails, the threshold slider, output directory, and the Run Comparison button.

If either capture is missing, a warning message tells you which tab to visit.

### Step 4 — Result Tab

This tab displays the comparison results. See [Result View Modes](#result-view-modes) for details.

> **📸 SCREENSHOT NEEDED:** The Result tab showing the heatmap view with the similarity percentage and diff pixel count.

### Step 5 — Inspect Tab

This tab compares extracted CSS properties between the Figma design and web implementation. See [Property Inspection & Export](#property-inspection--export) for details.

> **📸 SCREENSHOT NEEDED:** The Inspect tab showing the property comparison table with match/mismatch indicators.

---

## Inspector Panel (In-Browser)

When you click **Start Capture** in the Web tab, an inspector panel is injected at the bottom of the browser window. This is a developer-tools-style panel with a dark theme.

> **📸 SCREENSHOT NEEDED:** The browser window with the Loupe Inspector panel open at the bottom, showing the DOM tree, viewport buttons, and the Capture Selected button.

### DOM Tree

The inspector shows a navigable DOM tree of the current page:

- **Hover** over a row to highlight that element on the page with a blue outline
- **Click** a row to select it (highlighted in the tree and on the page)
- **Click the arrow (▶)** to expand/collapse child elements
- Elements show their tag, ID, classes, and (for images/iframes) the `src` attribute
- **Shadow DOM** roots are indicated with a `#shadow-root` label
- **Iframes** can be expanded to inspect their contents (same-origin only; cross-origin iframes show a note)
- Click **Refresh** in the header to rebuild the tree if the DOM has changed

### Viewport Switcher

The inspector includes a viewport switcher bar between the header and the DOM tree. Use it to resize the browser window to standard device sizes **without leaving the browser**:

| Preset  | Width  | Height |
|---------|--------|--------|
| Mobile  | 375px  | 812px  |
| Tablet  | 768px  | 1024px |
| Desktop | 1200px | 800px  |

Click a button to immediately resize. The active preset is highlighted in indigo.

> **📸 SCREENSHOT NEEDED:** Close-up of the viewport switcher bar in the inspector panel, with one of the presets (e.g. Mobile) selected.

### Capturing an Element

1. Select an element in the DOM tree (click the row)
2. The **Capture Selected** button in the header becomes active
3. Click **Capture Selected**
4. A toast notification confirms the capture ("Captured! Check the Loupe app.")
5. Switch back to the Loupe app — the capture is in the Web tab

The inspector also extracts computed CSS properties (font, color, spacing, borders, shadows, etc.) from the selected element and sends them to the app for use in the Inspect tab.

### Resizing the Inspector

The panel can be resized by dragging the top edge. Drag up to make it taller, drag down to make it shorter (minimum 120px).

---

## Comparison Settings

Configure these on the **Compare** tab before running a comparison:

| Setting          | Description                                                                                   | Default                    |
|------------------|-----------------------------------------------------------------------------------------------|----------------------------|
| **Threshold**    | How sensitive the diff is (0–100%). Lower values catch smaller color differences. Maps to the `pixelmatch` threshold. | 10%                        |
| **Output directory** | Folder where diff images are saved when you click Download.                              | `~/Pictures/Loupe/`        |
| **Filename**     | Pattern for saved files. `{timestamp}` is replaced with the current Unix timestamp.           | `diff-{timestamp}.png`     |

**How the diff works:**
- Both images are trimmed of surrounding whitespace/transparency
- The web capture is scaled to match the Figma frame dimensions (Figma is the source of truth)
- `pixelmatch` runs a per-pixel comparison and generates a heatmap of differences

---

## Result View Modes

The Result tab offers three ways to view the comparison:

### Heatmap (Default)

Shows the raw `pixelmatch` diff output — differing pixels are highlighted in red against a yellow background. Matching pixels appear transparent/black.

> **📸 SCREENSHOT NEEDED:** The heatmap view showing pixel differences between a Figma frame and web capture.

### Side by Side

Displays the web capture and Figma frame next to each other for visual comparison.

> **📸 SCREENSHOT NEEDED:** The side-by-side view with "Web" and "Figma" labels.

### Overlay

Layers the Figma frame over the web capture with an adjustable opacity slider (0–100%). Useful for spotting subtle alignment or sizing differences.

> **📸 SCREENSHOT NEEDED:** The overlay view with the opacity slider visible.

### Stats Bar

All view modes show a persistent stats bar at the top:
- **Similarity percentage** (e.g. "98.45% similar") — shown in green
- **Diff pixel count** (e.g. "1,234 / 500,000 pixels differ")

### Downloading

Click **Download Diff** to save the heatmap image to the configured output directory. The save path is confirmed in a message below the button.

---

## Property Inspection & Export

The **Inspect** tab compares CSS properties extracted from both captures. Properties are organized into categories:

| Category    | Properties Compared                                                                 |
|-------------|------------------------------------------------------------------------------------|
| **Dimensions** | `width`, `height`                                                               |
| **Typography** | `font-family`, `font-size`, `font-weight`, `line-height`, `letter-spacing`, `text-align`, `color` |
| **Colors**     | `background-color`, `opacity`                                                   |
| **Borders**    | `border-width`, `border-color`, `border-radius`                                 |
| **Spacing**    | `padding-top`, `padding-right`, `padding-bottom`, `padding-left`, `gap`         |
| **Effects**    | `box-shadow`, `filter`                                                           |

### Match Indicators

- **Green dot (●)** — values match (within tolerance)
- **Red dot (●)** — values differ
- **Gray dot (●)** — cannot compare (one side missing or value is "mixed")

### Tolerances

Comparisons use smart tolerances:
- **Dimensions/spacing:** ±0.5px (sub-pixel tolerance)
- **Border width:** ±0.1px (strict)
- **Colors:** ±3 per RGB channel, ±0.05 alpha
- **Font family:** fuzzy substring match
- **Opacity:** ±0.05
- **Font weight:** exact match

### Excluding Properties

- **Uncheck a property** checkbox to exclude it from the similarity score
- **Uncheck a category** checkbox to exclude all its properties
- The summary bar updates in real-time showing how many properties match and how many are excluded

### Exporting Results

Two export buttons are available in the summary bar:

- **Copy Markdown** — copies a formatted Markdown table (ideal for GitHub PR comments)
- **Copy Text** — copies a plain-text summary

The Markdown output includes a color-coded header (🟢 ≥80%, 🟡 ≥50%, 🔴 <50%) and per-category tables with match indicators.

> **📸 SCREENSHOT NEEDED:** The Inspect tab summary bar showing the match percentage, Expand All / Collapse All buttons, and the Copy Markdown / Copy Text buttons.

---

## Output & Saving

- **Diff images** are saved via File → Open Output Folder (or the Compare tab's output directory setting)
- Default output folder: `~/Pictures/Loupe/`
- Filename pattern supports `{timestamp}` which is replaced with the Unix timestamp at save time
- Images are saved as PNG

You can open the output folder at any time via the **File → Open Output Folder** menu item.

---

## Troubleshooting

### "Could not reach Loupe. Is the app running?"

The Figma plugin can't reach the desktop app. Make sure:
- Loupe is running
- Nothing else is using port **7700**
- Your firewall isn't blocking localhost connections

### Figma frame doesn't appear

- Make sure you have a **frame selected** in Figma before running the plugin
- Check the plugin status message for errors
- The plugin only works with the **desktop Figma app** (not the browser version) since it needs network access to `localhost`

### Web capture looks wrong or is blank

- Verify the URL is correct and the page is fully loaded before capturing
- Try clicking **Refresh** in the inspector to rebuild the DOM tree
- Some elements inside cross-origin iframes cannot be captured
- If box shadows appear oversized, ensure you're running the latest version (this is a known fixed issue)

### Comparison shows unexpected differences

- **Threshold too low?** Try increasing the threshold slider (e.g. 10–20%)
- **Different viewport sizes?** The web capture is scaled to match Figma dimensions, but large size differences can introduce scaling artifacts. Try to capture at a size close to the Figma frame.
- **Whitespace differences?** Both images are auto-trimmed, but backgrounds that aren't pure white may affect trimming

### Inspector panel doesn't appear

- Click **Start Capture** in the Web tab — the inspector is injected only after this step
- If the page has a strict Content Security Policy (CSP), it may block the injected script. Try using a local dev server without CSP restrictions.

---

## Releasing New Versions

The repo includes a GitHub Actions workflow that builds platform-specific installers automatically.

### Prerequisites (One-Time Setup)

1. **Enable GitHub Actions** on the repo: Settings → Actions → General → "Allow all actions"
2. **Set workflow permissions**: On the same page, under "Workflow permissions", select **"Read and write permissions"** so the `GITHUB_TOKEN` can create releases

No additional secrets need to be configured.

### Creating a Release

1. Update the version in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`
2. Commit the version bump
3. Tag the commit and push:
   ```sh
   git tag v0.2.0
   git push && git push --tags
   ```
4. The workflow runs **3 parallel builds**: macOS (ARM), Windows, and Linux
5. A **draft release** appears in GitHub Releases with all platform installers attached
6. Review the draft, edit the release notes if desired, and click **Publish**

### Manual Trigger

You can also trigger the workflow manually from the GitHub Actions tab using **"Run workflow"** (workflow_dispatch) without creating a tag.

---

## Architecture Overview

```
loupe/
├── src/                     # Svelte 5 frontend (SvelteKit + adapter-static)
│   ├── routes/+page.svelte  # Main app shell with tab routing
│   └── lib/
│       ├── state.svelte.js  # Reactive app state (Svelte 5 runes)
│       ├── diff.js          # pixelmatch integration + image trimming
│       ├── property-utils.js # Figma/Web property normalization + comparison
│       └── components/      # Tab components (FigmaTab, WebTab, CompareTab, ResultTab, InspectTab, TabNav)
├── src-tauri/               # Rust backend
│   ├── src/lib.rs           # HTTP server (port 7700), browser window management,
│   │                        #   inspector injection, Tauri commands
│   └── src/modern-screenshot.bundle.js  # Bundled modern-screenshot library
├── figma-plugin/            # Figma companion plugin
│   ├── manifest.json        # Plugin manifest (loaded locally, not published)
│   ├── code.js              # Plugin backend (frame export + property extraction)
│   └── ui.html              # Plugin UI (Send to Loupe button)
└── .github/workflows/
    └── build.yml            # CI: builds macOS, Windows, Linux installers on tag push
```

### Key Technologies

- **Tauri v2** — desktop app framework (Rust backend + webview frontend)
- **Svelte 5** — frontend framework with SvelteKit + adapter-static
- **pixelmatch** — pixel-level image comparison
- **modern-screenshot** — DOM-to-image capture (bundled, injected at compile time)
- **Axum** — HTTP server for receiving images from Figma plugin and browser captures
