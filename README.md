# Loupe

Compare live web components against Figma frames using pixel-level diffing and heatmap visualization. One desktop app, one companion Figma plugin, one repo.

## Run from Source

This version of loupe must only be run from source, not downloaded from any
other sources.

### Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/)
- Platform-specific dependencies (see below)

**Linux (Debian/Ubuntu):**

```sh
sudo apt-get install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libgtk-3-dev libsoup-3.0-dev libjavascriptcoregtk-4.1-dev
```

**Linux (Fedora):**

```sh
sudo dnf install webkit2gtk4.1-devel openssl-devel libappindicator-gtk3-devel librsvg2-devel pango-devel gtk3-devel libsoup3-devel javascriptcoregtk4.1-devel
```

**macOS / Windows:** No additional system dependencies required.

### Build & Run

```sh
git clone https://github.com/TitaniumCladStudios/loupe.git
cd loupe
npm install
npm run tauri dev
```

To create a production build:

```sh
npm run tauri build
```

Output will be in `src-tauri/target/release/bundle/`.

## Figma Plugin Setup

The Figma plugin is included in the repo and loaded locally (it is not published to the Figma Community).

1. Open Figma
2. Go to **Plugins > Development > Import plugin from manifest...**
3. Select `figma-plugin/manifest.json` from the cloned repo
4. The plugin will now appear under **Plugins > Development > Loupe**

## How It Works

Loupe is a four-step wizard:

### 1. Figma Tab

Open the Figma plugin, select a frame, and click **Send to Loupe**. The plugin exports the frame as a PNG and sends it to the app over `localhost:7700`.

### 2. Web Tab

Enter a URL (e.g. your local dev server) and click **Open Browser**. A browser window opens with an element picker — hover to highlight elements, click to capture one. The capture appears back in the app.

### 3. Compare Tab

Review both captures side by side, adjust the pixelmatch threshold, and click **Run Comparison**. The diff runs automatically and advances to the result.

### 4. Result Tab

View the output three ways:

- **Heatmap** — pixelmatch diff output showing where the pixels differ
- **Side by Side** — web capture and Figma frame next to each other
- **Overlay** — Figma frame layered over the web capture with an opacity slider

Similarity percentage and diff pixel count are shown persistently. The diff image can be downloaded.

## Architecture

```
loupe/
├── src/                  # Svelte 5 frontend (SvelteKit + Tauri)
├── src-tauri/            # Rust backend (HTTP server, window management)
├── figma-plugin/         # Figma plugin (loaded via manifest)
│   ├── manifest.json
│   ├── code.js
│   └── ui.html
└── package.json
```

- **Tauri v2** with a Svelte 5 frontend and Rust backend
- **Local HTTP server** on port `7700` receives images from the Figma plugin and the browser capture
- **pixelmatch** handles pixel-level image comparison
- **html2canvas** is injected into the browser window for element capture

## License

MIT
