//! Embedded MCP server for Loupe.
//!
//! Loupe is a GUI desktop app, so it cannot be spawned as a stdio MCP server by
//! a client. Instead we expose an MCP server over **Streamable HTTP** on port
//! 7701, served from the same process as the rest of the Tauri backend.
//!
//! The tools here drive the *running* app: they read shared state that the HTTP
//! handlers populate (Figma frames, web captures, properties) and trigger the
//! frontend to run the real pixel diff via Tauri events, collecting the result
//! through a one-shot callback channel.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    schemars,
    transport::streamable_http_server::{
        session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
    },
    tool, tool_handler, tool_router, ErrorData as McpError, ServerHandler,
};
// rmcp 2.x renamed `Content` to `ContentBlock`; alias keeps the call sites terse.
use rmcp::model::ContentBlock as Content;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::sync::{oneshot, Mutex};

/// Monotonic id source for correlating `run_comparison` round-trips.
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// The result of a pixel-level comparison, mirroring the frontend `runDiff`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub similarity: f64,
    pub diff_pixels: u64,
    pub total_pixels: u64,
}

/// State shared between the plugin HTTP server (7700) and the MCP server (7701).
#[derive(Default)]
pub struct LoupeData {
    pub figma_image: Option<String>,
    pub web_image: Option<String>,
    pub figma_properties: Option<serde_json::Value>,
    pub web_properties: Option<serde_json::Value>,
    pub last_result: Option<ComparisonResult>,
}

/// Pending `run_comparison` requests awaiting a result from the frontend.
type PendingMap = Arc<Mutex<HashMap<u64, oneshot::Sender<Result<ComparisonResult, String>>>>>;

/// Pending `capture_web_element` requests awaiting the injected capture script.
type CapturePending = Arc<Mutex<HashMap<u64, oneshot::Sender<Result<(), String>>>>>;

/// Cloneable bundle of everything the servers and tools need.
#[derive(Clone)]
pub struct Shared {
    pub app: AppHandle,
    pub data: Arc<Mutex<LoupeData>>,
    pub pending: PendingMap,
    pub captures: CapturePending,
}

impl Shared {
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            data: Arc::new(Mutex::new(LoupeData::default())),
            pending: Arc::new(Mutex::new(HashMap::new())),
            captures: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

// --- Tool argument types ---

#[derive(Clone, Copy, Debug, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
enum Target {
    Figma,
    Web,
}

impl Target {
    fn label(self) -> &'static str {
        match self {
            Target::Figma => "Figma",
            Target::Web => "Web",
        }
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct SendImageArgs {
    /// Which Loupe tab to populate.
    target: Target,
    /// PNG as a data URL (`data:image/png;base64,...`) or a raw base64 string.
    image: String,
    /// Optional extracted properties to attach to the image.
    #[serde(default)]
    properties: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct RunComparisonArgs {
    /// Pixelmatch threshold percent (0–100). Defaults to the app's current value.
    #[serde(default)]
    threshold: Option<f64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct GetPropertiesArgs {
    /// Which side's extracted properties to return.
    target: Target,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct CaptureWebArgs {
    /// The page URL to open (http/https only), e.g. http://localhost:3000.
    url: String,
    /// CSS selector for the element to capture, e.g. ".card" or "#hero h1".
    selector: String,
}

// --- MCP server ---

#[derive(Clone)]
pub struct LoupeMcp {
    shared: Shared,
    tool_router: ToolRouter<LoupeMcp>,
}

#[tool_router]
impl LoupeMcp {
    pub fn new(shared: Shared) -> Self {
        Self {
            shared,
            tool_router: Self::tool_router(),
        }
    }

    fn json_result(value: &impl Serialize) -> Result<CallToolResult, McpError> {
        let text = serde_json::to_string_pretty(value)
            .map_err(|e| McpError::internal_error(format!("serialize error: {e}"), None))?;
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    #[tool(description = "Report what the running Loupe app currently has loaded: \
        whether a Figma frame and a web capture are present, and the most recent \
        comparison result (if any).")]
    async fn get_status(&self) -> Result<CallToolResult, McpError> {
        let d = self.shared.data.lock().await;
        let status = serde_json::json!({
            "figmaLoaded": d.figma_image.is_some(),
            "webLoaded": d.web_image.is_some(),
            "figmaPropertiesAvailable": d.figma_properties.is_some(),
            "webPropertiesAvailable": d.web_properties.is_some(),
            "lastResult": d.last_result,
        });
        Self::json_result(&status)
    }

    #[tool(description = "Push a PNG into the running Loupe app, exactly like the \
        Figma plugin or browser capture would. target \"figma\" fills the Figma \
        tab; \"web\" fills the Web tab. Optionally include extracted properties.")]
    async fn send_image(
        &self,
        Parameters(args): Parameters<SendImageArgs>,
    ) -> Result<CallToolResult, McpError> {
        let (img_event, prop_event) = match args.target {
            Target::Figma => ("figma-image", "figma-properties"),
            Target::Web => ("web-capture", "web-properties"),
        };

        let _ = self.shared.app.emit(img_event, &args.image);
        {
            let mut d = self.shared.data.lock().await;
            match args.target {
                Target::Figma => d.figma_image = Some(args.image.clone()),
                Target::Web => d.web_image = Some(args.image.clone()),
            }
            if let Some(props) = &args.properties {
                match args.target {
                    Target::Figma => d.figma_properties = Some(props.clone()),
                    Target::Web => d.web_properties = Some(props.clone()),
                }
            }
        }
        if let Some(props) = &args.properties {
            let _ = self.shared.app.emit(prop_event, props);
        }

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Sent image to the {} tab.",
            args.target.label()
        ))]))
    }

    #[tool(description = "Open the integrated browser at a URL, automatically \
        capture the element matching a CSS selector, and load it into the Web \
        tab — no manual clicking required. This is the agent-driven equivalent \
        of opening the browser and using the element picker. Returns once the \
        capture completes or fails.")]
    async fn capture_web_element(
        &self,
        Parameters(args): Parameters<CaptureWebArgs>,
    ) -> Result<CallToolResult, McpError> {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = oneshot::channel();
        self.shared.captures.lock().await.insert(id, tx);

        if let Err(message) =
            crate::open_browser_for_capture(&self.shared.app, &args.url, &args.selector, id)
        {
            self.shared.captures.lock().await.remove(&id);
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Could not open browser: {message}"
            ))]));
        }

        // The page must load and the selector must appear before capture; allow
        // more time than the in-page deadline (15s) for navigation overhead.
        let outcome = tokio::time::timeout(Duration::from_secs(25), rx).await;
        self.shared.captures.lock().await.remove(&id);

        match outcome {
            Ok(Ok(Ok(()))) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Captured element matching \"{}\" from {} into the Web tab.",
                args.selector, args.url
            ))])),
            Ok(Ok(Err(message))) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Capture failed: {message}"
            ))])),
            Ok(Err(_)) => Ok(CallToolResult::error(vec![Content::text(
                "The capture result channel closed unexpectedly.",
            )])),
            Err(_) => Ok(CallToolResult::error(vec![Content::text(
                "Timed out waiting for the capture. The page may be slow to load \
                 or the selector may not match any element.",
            )])),
        }
    }

    #[tool(description = "Run a pixel-level comparison between the loaded Figma \
        frame and web capture in the running app, returning similarity percent, \
        differing pixel count, and total pixels. Both images must be loaded first \
        (via the plugin/capture or send_image).")]
    async fn run_comparison(
        &self,
        Parameters(args): Parameters<RunComparisonArgs>,
    ) -> Result<CallToolResult, McpError> {
        {
            let d = self.shared.data.lock().await;
            if d.figma_image.is_none() || d.web_image.is_none() {
                return Ok(CallToolResult::error(vec![Content::text(
                    "Both a Figma frame and a web capture must be loaded before running a comparison.",
                )]));
            }
        }

        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = oneshot::channel();
        self.shared.pending.lock().await.insert(id, tx);

        let mut payload = serde_json::json!({ "requestId": id });
        if let Some(threshold) = args.threshold {
            payload["threshold"] = serde_json::json!(threshold);
        }
        let _ = self.shared.app.emit("mcp-run-comparison", payload);

        let outcome = tokio::time::timeout(Duration::from_secs(30), rx).await;
        self.shared.pending.lock().await.remove(&id);

        match outcome {
            Ok(Ok(Ok(result))) => {
                self.shared.data.lock().await.last_result = Some(result.clone());
                Self::json_result(&result)
            }
            Ok(Ok(Err(message))) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Comparison failed in the app: {message}"
            ))])),
            Ok(Err(_)) => Ok(CallToolResult::error(vec![Content::text(
                "The comparison result channel closed unexpectedly.",
            )])),
            Err(_) => Ok(CallToolResult::error(vec![Content::text(
                "Timed out waiting for the app to run the comparison. Is the Loupe window open?",
            )])),
        }
    }

    #[tool(description = "Return the most recent comparison result computed by the \
        running app, or a message if none has been run yet.")]
    async fn get_last_comparison(&self) -> Result<CallToolResult, McpError> {
        let d = self.shared.data.lock().await;
        match &d.last_result {
            Some(result) => Self::json_result(result),
            None => Ok(CallToolResult::success(vec![Content::text(
                "No comparison has been run yet.",
            )])),
        }
    }

    #[tool(description = "Return the extracted properties for one side: the Figma \
        node properties or the web computed styles, as captured by the app.")]
    async fn get_properties(
        &self,
        Parameters(args): Parameters<GetPropertiesArgs>,
    ) -> Result<CallToolResult, McpError> {
        let d = self.shared.data.lock().await;
        let props = match args.target {
            Target::Figma => &d.figma_properties,
            Target::Web => &d.web_properties,
        };
        match props {
            Some(value) => Self::json_result(value),
            None => Ok(CallToolResult::success(vec![Content::text(format!(
                "No {} properties have been captured yet.",
                args.target.label()
            ))])),
        }
    }
}

#[tool_handler]
impl ServerHandler for LoupeMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_protocol_version(ProtocolVersion::V_2024_11_05)
            .with_instructions(
                "Loupe MCP server. Drives the running Loupe desktop app, which \
                 compares Figma frames against live web components. Tools: \
                 get_status, send_image, capture_web_element, run_comparison, \
                 get_last_comparison, get_properties. Load a Figma frame and a \
                 web capture (via the Figma plugin and browser capture, \
                 send_image, or capture_web_element) before calling \
                 run_comparison.",
            )
    }
}

/// Start the MCP server on port 7701 (Streamable HTTP, mounted at `/mcp`).
pub fn start_mcp_server(shared: Shared) {
    tauri::async_runtime::spawn(async move {
        let service = StreamableHttpService::new(
            move || Ok(LoupeMcp::new(shared.clone())),
            LocalSessionManager::default().into(),
            StreamableHttpServerConfig::default(),
        );

        let router = axum::Router::new().nest_service("/mcp", service);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:7701")
            .await
            .expect("Failed to bind MCP port 7701");
        if let Err(e) = axum::serve(listener, router).await {
            eprintln!("MCP server error: {e}");
        }
    });
}
