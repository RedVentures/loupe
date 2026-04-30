use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use base64::Engine;
use serde::Deserialize;
use tauri::menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri::{AppHandle, Emitter, Manager};
use tower_http::cors::CorsLayer;

// --- HTTP Server (port 7700) ---

#[derive(Clone)]
struct ServerState {
    app: AppHandle,
}

#[derive(Deserialize)]
struct ImagePayload {
    image: String,
    properties: Option<serde_json::Value>,
}

async fn handle_figma(
    State(state): State<ServerState>,
    Json(payload): Json<ImagePayload>,
) -> StatusCode {
    let _ = state.app.emit("figma-image", &payload.image);
    if let Some(props) = &payload.properties {
        let _ = state.app.emit("figma-properties", props);
    }
    StatusCode::OK
}

async fn handle_capture(
    State(state): State<ServerState>,
    Json(payload): Json<ImagePayload>,
) -> StatusCode {
    let _ = state.app.emit("web-capture", &payload.image);
    if let Some(props) = &payload.properties {
        let _ = state.app.emit("web-properties", props);
    }
    StatusCode::OK
}

#[derive(Deserialize)]
struct ResizePayload {
    width: f64,
    height: f64,
}

async fn handle_resize(
    State(state): State<ServerState>,
    Json(payload): Json<ResizePayload>,
) -> StatusCode {
    if let Some(webview) = state.app.get_webview_window("browse") {
        let _ = webview.set_size(tauri::Size::Logical(tauri::LogicalSize {
            width: payload.width,
            height: payload.height,
        }));
    }
    StatusCode::OK
}

fn start_server(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let state = ServerState { app };
        let router = Router::new()
            .route("/figma", post(handle_figma))
            .route("/capture", post(handle_capture))
            .route("/resize", post(handle_resize))
            .layer(
                CorsLayer::new()
                    .allow_origin(tower_http::cors::Any)
                    .allow_methods([axum::http::Method::POST])
                    .allow_headers([axum::http::header::CONTENT_TYPE]),
            )
            .with_state(state);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:7700")
            .await
            .expect("Failed to bind port 7700");
        axum::serve(listener, router)
            .await
            .expect("HTTP server error");
    });
}

// --- Inspector injection script ---

// Bundle modern-screenshot at compile time (pinned npm dep, not CDN)
const MODERN_SCREENSHOT_BUNDLE: &str = include_str!("modern-screenshot.bundle.js");

fn picker_script() -> String {
    // Inject the bundled modern-screenshot IIFE first, then the picker
    let mut script = String::new();
    script.push_str(MODERN_SCREENSHOT_BUNDLE);
    script.push('\n');
    script.push_str(r#"
(function() {
    if (window.__loupeInspectorActive) return;
    window.__loupeInspectorActive = true;

    /* ── Highlight overlay ── */
    const overlay = document.createElement('div');
    overlay.id = '__loupe_overlay';
    overlay.style.cssText = 'position:fixed;pointer-events:none;border:2px solid #6366f1;background:rgba(99,102,241,0.1);z-index:2147483646;display:none;transition:all 0.05s;';
    document.body.appendChild(overlay);

    /* ── Toast ── */
    const toast = document.createElement('div');
    toast.style.cssText = 'position:fixed;bottom:300px;left:50%;transform:translateX(-50%);padding:8px 16px;border-radius:6px;font:13px/1.4 system-ui,sans-serif;z-index:2147483647;pointer-events:none;opacity:0;transition:opacity 0.3s;';
    document.body.appendChild(toast);
    function showToast(msg, err) {
        toast.textContent = msg;
        toast.style.background = err ? '#ef4444' : '#22c55e';
        toast.style.color = '#fff';
        toast.style.opacity = '1';
        setTimeout(() => toast.style.opacity = '0', 3000);
    }

    /* ── Panel ── */
    const panel = document.createElement('div');
    panel.id = '__loupe_inspector';
    panel.style.cssText = `
        position:fixed;bottom:0;left:0;right:0;height:280px;
        background:#1e1e2e;color:#cdd6f4;
        font-family:'SF Mono',Monaco,Consolas,'Liberation Mono',monospace;font-size:12px;
        z-index:2147483645;display:flex;flex-direction:column;
        border-top:2px solid #6366f1;box-shadow:0 -4px 20px rgba(0,0,0,0.3);
    `;

    /* resize handle */
    const resizer = document.createElement('div');
    resizer.style.cssText = 'height:6px;cursor:ns-resize;background:transparent;position:absolute;top:-3px;left:0;right:0;z-index:1;';
    panel.appendChild(resizer);
    let resizing = false;
    resizer.addEventListener('mousedown', (e) => { resizing = true; e.preventDefault(); });
    document.addEventListener('mousemove', (e) => {
        if (!resizing) return;
        const h = window.innerHeight - e.clientY;
        panel.style.height = Math.max(120, Math.min(h, window.innerHeight - 60)) + 'px';
    });
    document.addEventListener('mouseup', () => { resizing = false; });

    /* header */
    const header = document.createElement('div');
    header.style.cssText = 'display:flex;align-items:center;justify-content:space-between;padding:6px 12px;background:#181825;border-bottom:1px solid #313244;user-select:none;flex-shrink:0;';
    header.innerHTML = `
        <span style="font-weight:600;color:#cba6f7;">Loupe Inspector</span>
        <div style="display:flex;gap:8px;align-items:center;">
            <button id="__loupe_refresh" style="padding:3px 10px;background:#313244;color:#cdd6f4;border:1px solid #45475a;border-radius:4px;font-size:11px;cursor:pointer;" title="Refresh DOM tree">Refresh</button>
            <button id="__loupe_capture_btn" style="padding:3px 12px;background:#6366f1;color:#fff;border:none;border-radius:4px;font-size:11px;cursor:pointer;font-weight:600;opacity:0.5;" disabled>Capture Selected</button>
        </div>
    `;
    panel.appendChild(header);

    /* ── Viewport switcher ── */
    const viewports = [
        { id: 'mobile',  label: 'Mobile',  w: 375,  h: 812  },
        { id: 'tablet',  label: 'Tablet',  w: 768,  h: 1024 },
        { id: 'desktop', label: 'Desktop', w: 1200, h: 800  },
    ];
    const vpBar = document.createElement('div');
    vpBar.style.cssText = 'display:flex;gap:2px;padding:4px 12px;background:#181825;border-bottom:1px solid #313244;flex-shrink:0;';
    let activeVp = 'desktop';
    const vpBtns = {};

    function updateVpButtons() {
        for (const v of viewports) {
            const btn = vpBtns[v.id];
            if (v.id === activeVp) {
                btn.style.background = '#6366f1';
                btn.style.color = '#fff';
                btn.style.borderColor = '#6366f1';
            } else {
                btn.style.background = '#313244';
                btn.style.color = '#cdd6f4';
                btn.style.borderColor = '#45475a';
            }
        }
    }

    for (const v of viewports) {
        const btn = document.createElement('button');
        btn.style.cssText = 'flex:1;display:flex;align-items:center;justify-content:center;gap:4px;padding:4px 8px;font-size:11px;font-family:system-ui,sans-serif;border:1px solid #45475a;border-radius:4px;cursor:pointer;transition:all 0.1s;';
        btn.innerHTML = `<span>${v.label}</span><span style="opacity:0.6;font-size:10px;">${v.w}\u00d7${v.h}</span>`;
        btn.title = `${v.label} (${v.w}\u00d7${v.h})`;
        btn.addEventListener('click', () => {
            activeVp = v.id;
            updateVpButtons();
            fetch('http://localhost:7700/resize', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ width: v.w, height: v.h }),
            }).catch(() => {});
        });
        vpBtns[v.id] = btn;
        vpBar.appendChild(btn);
    }
    updateVpButtons();
    panel.appendChild(vpBar);

    /* tree container */
    const treeWrap = document.createElement('div');
    treeWrap.style.cssText = 'flex:1;overflow:auto;padding:4px 0;';
    panel.appendChild(treeWrap);

    document.body.appendChild(panel);

    const captureBtn = panel.querySelector('#__loupe_capture_btn');
    const refreshBtn = panel.querySelector('#__loupe_refresh');

    let selectedEl = null;
    let selectedWin = null;
    let selectedRow = null;
    let selectedOffsets = [];

    /* ── Highlight helper ── */
    function highlight(el, offsets) {
        if (!el) { overlay.style.display = 'none'; return; }
        const r = el.getBoundingClientRect();
        let top = r.top, left = r.left;
        for (const o of offsets) { top += o.top; left += o.left; }
        overlay.style.display = 'block';
        overlay.style.top = top + 'px';
        overlay.style.left = left + 'px';
        overlay.style.width = r.width + 'px';
        overlay.style.height = r.height + 'px';
    }

    function selectNode(el, win, row, offsets) {
        if (selectedRow) selectedRow.style.background = '';
        selectedEl = el;
        selectedWin = win;
        selectedRow = row;
        selectedOffsets = offsets;
        row.style.background = '#313244';
        captureBtn.disabled = false;
        captureBtn.style.opacity = '1';
        highlight(el, offsets);
    }

    /* ── Format element tag ── */
    function fmtTag(el) {
        const tag = el.tagName.toLowerCase();
        let s = '<span style="color:#89b4fa">&lt;' + tag + '</span>';
        if (el.id) s += '<span style="color:#fab387"> #' + el.id + '</span>';
        if (el.className && typeof el.className === 'string') {
            const c = el.className.trim();
            if (c) s += '<span style="color:#a6e3a1"> .' + c.split(/\s+/).join('.') + '</span>';
        }
        // Show src for images/iframes
        if ((tag === 'img' || tag === 'iframe') && el.src) {
            const src = el.src.length > 60 ? el.src.slice(0, 57) + '...' : el.src;
            s += '<span style="color:#585b70"> src="' + src + '"</span>';
        }
        s += '<span style="color:#89b4fa">&gt;</span>';
        // Show text content preview for leaf nodes
        if (!el.children.length && !el.shadowRoot && el.textContent) {
            const txt = el.textContent.trim();
            if (txt && txt.length < 60) {
                s += '<span style="color:#585b70;font-style:italic;"> ' + txt.replace(/</g,'&lt;') + '</span>';
            }
        }
        return s;
    }

    /* ── Get visible children (works on elements, shadow roots, and document fragments) ── */
    function visibleChildren(node) {
        const children = node.children || node.childNodes;
        return Array.from(children).filter(c =>
            c.nodeType === 1 &&
            c.id !== '__loupe_inspector' &&
            c.id !== '__loupe_overlay' &&
            c !== toast && c !== overlay && c !== panel
        );
    }

    /* ── Build tree node ── */
    function buildNode(el, depth, parent, win, offsets) {
        const kids = visibleChildren(el);
        const isIframe = el.tagName === 'IFRAME';
        const hasShadow = !!(el.shadowRoot);
        let iframeDoc = null;

        if (isIframe) {
            try {
                iframeDoc = el.contentDocument || (el.contentWindow && el.contentWindow.document);
                if (iframeDoc && !iframeDoc.body) iframeDoc = null;
            } catch(e) { iframeDoc = null; }
        }

        const shadowKids = hasShadow ? visibleChildren(el.shadowRoot) : [];
        const expandable = kids.length > 0 || shadowKids.length > 0 || (iframeDoc && iframeDoc.body.children.length > 0);

        const row = document.createElement('div');
        row.style.cssText = 'display:flex;align-items:center;padding:1px 8px 1px ' + (12 + depth * 16) + 'px;cursor:pointer;white-space:nowrap;border-radius:2px;min-height:22px;';

        const arrow = document.createElement('span');
        arrow.style.cssText = 'display:inline-flex;align-items:center;justify-content:center;width:16px;height:16px;flex-shrink:0;color:#585b70;font-size:9px;user-select:none;';
        arrow.textContent = expandable ? '▶' : '';

        const label = document.createElement('span');
        label.innerHTML = fmtTag(el);
        if (hasShadow) {
            label.innerHTML += '<span style="color:#f9e2af;font-size:10px;margin-left:6px;">#shadow-root</span>';
        }
        if (isIframe && iframeDoc) {
            label.innerHTML += '<span style="color:#f38ba8;font-size:10px;margin-left:6px;">IFRAME</span>';
        } else if (isIframe) {
            label.innerHTML += '<span style="color:#585b70;font-size:10px;margin-left:6px;">iframe (cross-origin)</span>';
        }

        row.appendChild(arrow);
        row.appendChild(label);
        parent.appendChild(row);

        const childBox = document.createElement('div');
        childBox.style.display = 'none';
        parent.appendChild(childBox);

        let expanded = false;
        let childrenBuilt = false;

        row.addEventListener('mouseenter', () => {
            if (row !== selectedRow) row.style.background = '#262637';
            highlight(el, offsets);
        });
        row.addEventListener('mouseleave', () => {
            if (row !== selectedRow) row.style.background = '';
            if (selectedEl) highlight(selectedEl, selectedOffsets);
            else overlay.style.display = 'none';
        });

        arrow.addEventListener('click', (e) => {
            e.stopPropagation();
            if (!expandable) return;
            expanded = !expanded;
            arrow.textContent = expanded ? '▼' : '▶';
            childBox.style.display = expanded ? '' : 'none';

            if (expanded && !childrenBuilt) {
                childrenBuilt = true;
                // Shadow DOM children
                if (hasShadow) {
                    shadowKids.forEach(child => {
                        buildNode(child, depth + 1, childBox, win, offsets);
                    });
                }
                // iframe children
                if (isIframe && iframeDoc) {
                    const iRect = el.getBoundingClientRect();
                    const nestedOffsets = [...offsets, { top: iRect.top, left: iRect.left }];
                    visibleChildren(iframeDoc.body).forEach(child => {
                        buildNode(child, depth + 1, childBox, el.contentWindow, nestedOffsets);
                    });
                }
                // Light DOM children
                kids.forEach(child => {
                    buildNode(child, depth + 1, childBox, win, offsets);
                });
            }
        });

        row.addEventListener('click', (e) => {
            e.stopPropagation();
            selectNode(el, win, row, offsets);
        });
    }

    /* ── Build tree ── */
    function buildTree() {
        treeWrap.innerHTML = '';
        selectedEl = null;
        selectedWin = null;
        selectedRow = null;
        captureBtn.disabled = true;
        captureBtn.style.opacity = '0.5';
        overlay.style.display = 'none';

        visibleChildren(document.body).forEach(child => {
            buildNode(child, 0, treeWrap, window, []);
        });
    }

    buildTree();
    refreshBtn.addEventListener('click', buildTree);

    /* ── Capture ── */
    captureBtn.addEventListener('click', async () => {
        if (!selectedEl) return;
        captureBtn.disabled = true;
        captureBtn.textContent = 'Capturing...';

        try {
            const targetWin = selectedWin || window;

            // modern-screenshot is bundled at compile time (pinned npm dep, not CDN)
            const dataUrl = await modernScreenshot.domToPng(selectedEl, {
                scale: 2,
                backgroundColor: null,
                style: { margin: '0', boxShadow: 'none' },
                onEmbedNode: (cloned) => {
                    // Strip box-shadow from all descendants after styles have
                    // been embedded, so shadows don't render at 2× scale.
                    if (cloned && cloned.style) {
                        cloned.style.setProperty('box-shadow', 'none', 'important');
                    }
                    if (cloned && cloned.querySelectorAll) {
                        cloned.querySelectorAll('*').forEach((el) => {
                            if (el.style) el.style.setProperty('box-shadow', 'none', 'important');
                        });
                    }
                },
            });

            // Extract computed CSS properties (flat, for legacy inspect)
            const cs = targetWin.getComputedStyle(selectedEl);
            const properties = {
                'width': cs.width, 'height': cs.height,
                'font-family': cs.fontFamily, 'font-size': cs.fontSize,
                'font-weight': cs.fontWeight, 'line-height': cs.lineHeight,
                'letter-spacing': cs.letterSpacing, 'text-align': cs.textAlign,
                'color': cs.color, 'background-color': cs.backgroundColor,
                'opacity': cs.opacity,
                'border-top-width': cs.borderTopWidth, 'border-right-width': cs.borderRightWidth,
                'border-bottom-width': cs.borderBottomWidth, 'border-left-width': cs.borderLeftWidth,
                'border-top-color': cs.borderTopColor, 'border-right-color': cs.borderRightColor,
                'border-bottom-color': cs.borderBottomColor, 'border-left-color': cs.borderLeftColor,
                'border-top-left-radius': cs.borderTopLeftRadius, 'border-top-right-radius': cs.borderTopRightRadius,
                'border-bottom-right-radius': cs.borderBottomRightRadius, 'border-bottom-left-radius': cs.borderBottomLeftRadius,
                'padding-top': cs.paddingTop, 'padding-right': cs.paddingRight,
                'padding-bottom': cs.paddingBottom, 'padding-left': cs.paddingLeft,
                'gap': cs.gap, 'row-gap': cs.rowGap, 'column-gap': cs.columnGap,
                'box-shadow': cs.boxShadow, 'filter': cs.filter,
            };

            // Recursive DOM tree extraction
            function extractElProps(el, win) {
                const s = win.getComputedStyle(el);
                return {
                    'width': s.width, 'height': s.height,
                    'font-family': s.fontFamily, 'font-size': s.fontSize,
                    'font-weight': s.fontWeight, 'line-height': s.lineHeight,
                    'letter-spacing': s.letterSpacing, 'text-align': s.textAlign,
                    'color': s.color, 'background-color': s.backgroundColor,
                    'opacity': s.opacity,
                    'border-top-width': s.borderTopWidth, 'border-right-width': s.borderRightWidth,
                    'border-bottom-width': s.borderBottomWidth, 'border-left-width': s.borderLeftWidth,
                    'border-top-color': s.borderTopColor, 'border-right-color': s.borderRightColor,
                    'border-bottom-color': s.borderBottomColor, 'border-left-color': s.borderLeftColor,
                    'border-top-left-radius': s.borderTopLeftRadius, 'border-top-right-radius': s.borderTopRightRadius,
                    'border-bottom-right-radius': s.borderBottomRightRadius, 'border-bottom-left-radius': s.borderBottomLeftRadius,
                    'padding-top': s.paddingTop, 'padding-right': s.paddingRight,
                    'padding-bottom': s.paddingBottom, 'padding-left': s.paddingLeft,
                    'gap': s.gap, 'row-gap': s.rowGap, 'column-gap': s.columnGap,
                    'box-shadow': s.boxShadow, 'filter': s.filter,
                };
            }

            function extractDomTree(el, rootRect, win, depth) {
                if (depth <= 0) return null;
                const r = el.getBoundingClientRect();
                const tag = el.tagName.toLowerCase();
                const isLeafText = el.children.length === 0 && el.textContent.trim().length > 0;
                const node = {
                    type: tag,
                    name: tag + (el.className && typeof el.className === 'string' ? '.' + el.className.trim().split(/\s+/).join('.') : ''),
                    bbox: {
                        x: Math.round(r.left - rootRect.left),
                        y: Math.round(r.top - rootRect.top),
                        width: Math.round(r.width),
                        height: Math.round(r.height),
                    },
                    props: extractElProps(el, win),
                    children: [],
                };
                if (isLeafText) node.textContent = el.textContent.trim();
                for (let i = 0; i < el.children.length; i++) {
                    const child = el.children[i];
                    if (child.offsetParent === null && win.getComputedStyle(child).position !== 'fixed') continue;
                    const cn = extractDomTree(child, rootRect, win, depth - 1);
                    if (cn) node.children.push(cn);
                }
                return node;
            }

            const rootRect = selectedEl.getBoundingClientRect();
            const domTree = extractDomTree(selectedEl, rootRect, targetWin, 10);
            properties.__domTree = domTree;

            const resp = await fetch('http://localhost:7700/capture', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ image: dataUrl, properties: properties })
            });

            if (resp.ok) {
                showToast('Captured! Check the Loupe app.', false);
            } else {
                showToast('Server error: ' + resp.status, true);
            }
        } catch(err) {
            showToast('Error: ' + err.message, true);
            console.error('Loupe capture error:', err);
        }

        captureBtn.disabled = false;
        captureBtn.textContent = 'Capture Selected';
    });
})();
"#);
    script
}

fn deactivate_picker_script() -> &'static str {
    r#"
(function() {
    window.__loupeInspectorActive = false;
    const el = document.getElementById('__loupe_inspector');
    if (el) el.remove();
    const ov = document.getElementById('__loupe_overlay');
    if (ov) ov.remove();
})();
"#
}

// --- Tauri Commands ---

#[tauri::command]
async fn open_browser(app: AppHandle, url: String, width: Option<f64>, height: Option<f64>) -> Result<(), String> {
    // Rec #6: Restrict to http/https schemes only
    let parsed: url::Url = url.parse().map_err(|e| format!("{e}"))?;
    match parsed.scheme() {
        "http" | "https" => {}
        scheme => return Err(format!("Blocked URL scheme: {scheme}. Only http and https are allowed.")),
    }

    // Close existing browser window if open
    if let Some(existing) = app.get_webview_window("browse") {
        let _ = existing.close();
    }

    let w = width.unwrap_or(1200.0);
    let h = height.unwrap_or(800.0);

    tauri::WebviewWindowBuilder::new(
        &app,
        "browse",
        tauri::WebviewUrl::External(parsed),
    )
    .title("Loupe Browser")
    .inner_size(w, h)
    .build()
    .map_err(|e| format!("{e}"))?;

    Ok(())
}

#[tauri::command]
async fn start_capture(app: AppHandle) -> Result<(), String> {
    let webview = app
        .get_webview_window("browse")
        .ok_or("Browser window is not open")?;
    webview
        .eval(&picker_script())
        .map_err(|e| format!("{e}"))?;
    Ok(())
}

#[tauri::command]
async fn stop_capture(app: AppHandle) -> Result<(), String> {
    let webview = app
        .get_webview_window("browse")
        .ok_or("Browser window is not open")?;
    webview
        .eval(deactivate_picker_script())
        .map_err(|e| format!("{e}"))?;
    Ok(())
}

#[tauri::command]
async fn resize_browser(app: AppHandle, width: f64, height: f64) -> Result<(), String> {
    let webview = app
        .get_webview_window("browse")
        .ok_or("Browser window is not open")?;
    webview
        .set_size(tauri::Size::Logical(tauri::LogicalSize { width, height }))
        .map_err(|e| format!("{e}"))?;
    Ok(())
}

#[tauri::command]
async fn close_browser(app: AppHandle) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("browse") {
        w.close().map_err(|e| format!("{e}"))?;
    }
    Ok(())
}

#[tauri::command]
async fn get_default_output_dir() -> Result<String, String> {
    let dir = dirs::picture_dir()
        .or_else(dirs::home_dir)
        .ok_or("Could not determine home directory")?
        .join("Loupe");
    std::fs::create_dir_all(&dir).map_err(|e| format!("{e}"))?;
    dir.to_str()
        .map(String::from)
        .ok_or_else(|| "Invalid path".into())
}

#[tauri::command]
async fn open_output_dir(path: String) -> Result<(), String> {
    if path.is_empty() {
        return Err("No output directory set".into());
    }
    // Ensure the directory exists before opening
    std::fs::create_dir_all(&path).map_err(|e| format!("{e}"))?;
    tauri_plugin_opener::open_path(path, None::<&str>)
        .map_err(|e| format!("{e}"))?;
    Ok(())
}

#[tauri::command]
async fn save_image(path: String, data: String) -> Result<(), String> {
    let base64_data = data
        .strip_prefix("data:image/png;base64,")
        .unwrap_or(&data);

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(base64_data)
        .map_err(|e| format!("base64 decode error: {e}"))?;

    std::fs::write(&path, bytes).map_err(|e| format!("file write error: {e}"))?;
    Ok(())
}

// --- App Entry ---

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            open_browser,
            close_browser,
            resize_browser,
            start_capture,
            stop_capture,
            save_image,
            open_output_dir,
            get_default_output_dir
        ])
        .setup(|app| {
            start_server(app.handle().clone());

            // --- Menus ---
            let open_output_dir = MenuItemBuilder::with_id("open_output_dir", "Open Output Folder")
                .build(app)?;
            let file_menu = SubmenuBuilder::new(app, "File")
                .item(&open_output_dir)
                .separator()
                .quit()
                .build()?;

            let github_readme = MenuItemBuilder::with_id("github_readme", "Loupe on GitHub")
                .build(app)?;
            let help_menu = SubmenuBuilder::new(app, "Help")
                .item(&github_readme)
                .build()?;

            let menu = MenuBuilder::new(app)
                .item(&file_menu)
                .item(&help_menu)
                .build()?;

            app.set_menu(menu)?;

            app.on_menu_event(move |app, event| {
                match event.id().as_ref() {
                    "open_output_dir" => {
                        // Emit event to frontend to get the output dir from state
                        let _ = app.emit("menu-open-output-dir", ());
                    }
                    "github_readme" => {
                        let _ = tauri_plugin_opener::open_url(
                            "https://github.com/TitaniumCladStudios/loupe#readme",
                            None::<&str>,
                        );
                    }
                    _ => {}
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
