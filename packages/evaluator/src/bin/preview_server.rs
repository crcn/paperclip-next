//! Simple HTTP preview server with WebSocket support for live updates
//!
//! Usage:
//!   cargo run --bin preview_server -- path/to/component.pc
//!
//! Opens browser preview at http://localhost:3030 and watches for file changes,
//! sending VDOM patches over WebSocket for live updates without page reload.

use paperclip_evaluator::{
    Evaluator,
    VirtualDomDocument,
};
use paperclip_evaluator::css_differ::{diff_css_rules, CssDiff};
use paperclip_parser::parse_with_path;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use warp::{Filter, ws::{Message, WebSocket}};
use futures_util::{StreamExt, SinkExt};
use tokio::sync::broadcast;
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Config};

#[derive(Debug, Clone)]
struct PreviewState {
    file_path: PathBuf,
    last_vdom: Option<VirtualDomDocument>,
    previous_css: Vec<paperclip_evaluator::vdom::CssRule>,
    version: u64,
}

impl PreviewState {
    fn new(file_path: PathBuf) -> Self {
        Self {
            file_path,
            last_vdom: None,
            previous_css: Vec::new(),
            version: 0,
        }
    }

    fn update(&mut self) -> Result<Vec<u8>, String> {
        // Read and parse file
        let source = fs::read_to_string(&self.file_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let document = parse_with_path(&source, self.file_path.to_str().unwrap())
            .map_err(|e| format!("Parse error: {:?}", e))?;

        // Evaluate to VDOM
        let mut evaluator = Evaluator::with_document_id(self.file_path.to_str().unwrap());
        let new_vdom = evaluator.evaluate(&document)
            .map_err(|e| format!("Evaluation error: {:?}", e))?;

        // Compute CSS diff for incremental updates
        let css_diff = diff_css_rules(&self.previous_css, &new_vdom.styles);

        // Update state
        self.previous_css = new_vdom.styles.clone();
        self.last_vdom = Some(new_vdom.clone());
        self.version += 1;

        // Send incremental CSS patches if not the first update
        let update_data = if self.version == 1 || css_diff.is_empty() {
            // First update or no CSS changes - send full VDOM
            serde_json::json!({
                "type": "update",
                "version": self.version,
                "vdom": new_vdom,
            })
        } else {
            // Incremental update - send only CSS patches + DOM structure
            serde_json::json!({
                "type": "update",
                "version": self.version,
                "vdom": {
                    "nodes": new_vdom.nodes,
                    "styles": []  // Empty - will be patched
                },
                "css_patches": css_diff.patches,
            })
        };

        serde_json::to_vec(&update_data)
            .map_err(|e| format!("Serialization error: {}", e))
    }
}

#[tokio::main]
async fn main() {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path/to/component.pc>", args[0]);
        std::process::exit(1);
    }

    let file_path = PathBuf::from(&args[1]);
    if !file_path.exists() {
        eprintln!("File not found: {}", file_path.display());
        std::process::exit(1);
    }

    println!("📦 Paperclip Preview Server");
    println!("Watching: {}", file_path.display());
    println!();

    // Shared state
    let state = Arc::new(Mutex::new(PreviewState::new(file_path.clone())));
    let (tx, _rx) = broadcast::channel::<String>(100);

    // Initial evaluation
    {
        let mut state_guard = state.lock().unwrap();
        match state_guard.update() {
            Ok(_) => println!("✓ Initial evaluation successful"),
            Err(e) => eprintln!("⚠ Initial evaluation failed: {}", e),
        }
    }

    // File watcher
    let watch_tx = tx.clone();
    let watch_state = state.clone();
    let watch_path = file_path.clone();

    tokio::spawn(async move {
        let (tx_notify, mut rx_notify) = tokio::sync::mpsc::channel(100);

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<notify::Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx_notify.blocking_send(event);
                }
            },
            Config::default(),
        ).expect("Failed to create file watcher");

        watcher.watch(&watch_path, RecursiveMode::NonRecursive)
            .expect("Failed to watch file");

        println!("👀 Watching for file changes...\n");

        while let Some(event) = rx_notify.recv().await {
            if matches!(event.kind, notify::EventKind::Modify(_)) {
                println!("📝 File changed, recompiling...");

                let patch_data = {
                    let mut state_guard = watch_state.lock().unwrap();
                    match state_guard.update() {
                        Ok(data) => {
                            println!("✓ Recompiled successfully (v{})", state_guard.version);
                            Some(String::from_utf8_lossy(&data).to_string())
                        }
                        Err(e) => {
                            eprintln!("✗ Compilation error: {}", e);
                            // Send error to clients
                            let error_data = serde_json::json!({
                                "type": "error",
                                "message": e,
                            });
                            Some(error_data.to_string())
                        }
                    }
                };

                if let Some(data) = patch_data {
                    let _ = watch_tx.send(data);
                }
            }
        }
    });

    // WebSocket route
    let state_ws = state.clone();
    let tx_ws = tx.clone();

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::any().map(move || state_ws.clone()))
        .and(warp::any().map(move || tx_ws.clone()))
        .map(|ws: warp::ws::Ws, state: Arc<Mutex<PreviewState>>, tx: broadcast::Sender<String>| {
            ws.on_upgrade(move |socket| handle_client(socket, state, tx))
        });

    // Static HTML preview page
    let html_route = warp::path::end()
        .map(|| {
            warp::reply::html(PREVIEW_HTML)
        });

    // Combine routes
    let routes = html_route.or(ws_route);

    println!("🚀 Server running at http://localhost:3030");
    println!("   Open in browser to see live preview\n");

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

async fn handle_client(
    ws: WebSocket,
    state: Arc<Mutex<PreviewState>>,
    tx: broadcast::Sender<String>
) {
    let (mut ws_tx, mut ws_rx) = ws.split();
    let mut rx = tx.subscribe();

    // Send initial state
    let initial_json = {
        let state_guard = state.lock().unwrap();
        if let Some(ref vdom) = state_guard.last_vdom {
            let initial_data = serde_json::json!({
                "type": "initial",
                "version": state_guard.version,
                "vdom": vdom,
            });

            serde_json::to_string(&initial_data).ok()
        } else {
            None
        }
    };

    if let Some(json) = initial_json {
        let _ = ws_tx.send(Message::text(json)).await;
    }

    println!("🔌 Client connected");

    // Spawn task to send updates to client
    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if ws_tx.send(Message::text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Keep connection alive (receive messages from client if any)
    while let Some(result) = ws_rx.next().await {
        if result.is_err() {
            break;
        }
    }

    println!("🔌 Client disconnected");
}

const PREVIEW_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Paperclip Live Preview</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            background: #f5f5f5;
            padding: 20px;
        }
        .header {
            max-width: 1200px;
            margin: 0 auto 20px;
            padding: 16px 20px;
            background: white;
            border-radius: 8px;
            box-shadow: 0 1px 3px rgba(0,0,0,0.1);
            display: flex;
            align-items: center;
            justify-content: space-between;
        }
        .header h1 {
            font-size: 18px;
            color: #333;
        }
        .status {
            display: flex;
            align-items: center;
            gap: 8px;
            font-size: 14px;
            color: #666;
        }
        .status-dot {
            width: 8px;
            height: 8px;
            border-radius: 50%;
            background: #22c55e;
            animation: pulse 2s infinite;
        }
        .status-dot.disconnected {
            background: #ef4444;
            animation: none;
        }
        @keyframes pulse {
            0%, 100% { opacity: 1; }
            50% { opacity: 0.5; }
        }
        .preview-container {
            max-width: 1200px;
            margin: 0 auto;
            padding: 24px;
            background: white;
            border-radius: 8px;
            box-shadow: 0 1px 3px rgba(0,0,0,0.1);
            min-height: 400px;
        }
        #preview {
            /* Preview content will be rendered here */
        }
        .error {
            padding: 20px;
            background: #fef2f2;
            border-left: 4px solid #ef4444;
            color: #991b1b;
            border-radius: 4px;
            margin: 20px;
        }
        .error strong {
            display: block;
            margin-bottom: 12px;
            font-size: 16px;
            font-weight: 600;
            color: #dc2626;
        }
        .error pre {
            background: #fee2e2;
            padding: 12px;
            border-radius: 4px;
            overflow-x: auto;
            font-family: 'Monaco', 'Menlo', 'Courier New', monospace;
            font-size: 13px;
            line-height: 1.5;
            margin: 0;
        }
    </style>
</head>
<body>
    <div class="header">
        <h1>📦 Paperclip Live Preview</h1>
        <div class="status">
            <div class="status-dot" id="status-dot"></div>
            <span id="status-text">Connecting...</span>
        </div>
    </div>
    <div class="preview-container">
        <div id="preview">Loading...</div>
    </div>
    <script>
        const preview = document.getElementById('preview');
        const statusDot = document.getElementById('status-dot');
        const statusText = document.getElementById('status-text');

        let ws = null;
        let reconnectDelay = 1000;
        let currentCssRules = [];

        function displayError(message) {
            preview.innerHTML = `
                <div class="error">
                    <strong>Error</strong>
                    <pre>${escapeHtml(message)}</pre>
                    <p style="margin-top: 16px; color: #ef4444; font-size: 13px;">
                        Save the file again to retry compilation
                    </p>
                </div>
            `;
        }

        function escapeHtml(unsafe) {
            return unsafe
                .replace(/&/g, "&amp;")
                .replace(/</g, "&lt;")
                .replace(/>/g, "&gt;")
                .replace(/"/g, "&quot;")
                .replace(/'/g, "&#039;");
        }

        function connect() {
            ws = new WebSocket(`ws://${location.host}/ws`);

            ws.onopen = () => {
                console.log('Connected to preview server');
                statusDot.classList.remove('disconnected');
                statusText.textContent = 'Connected';
                reconnectDelay = 1000;
            };

            ws.onmessage = (event) => {
                try {
                    const data = JSON.parse(event.data);
                    console.log('Received:', data.type, data);

                    if (data.type === 'error') {
                        displayError(data.message);
                    } else if (data.type === 'initial' || data.type === 'update') {
                        try {
                            // Apply incremental CSS patches if present
                            if (data.css_patches && data.css_patches.length > 0) {
                                console.log(`Applying ${data.css_patches.length} CSS patches`);
                                applyCssPatches(data.css_patches);
                            }

                            // Render VDOM (may have full styles or empty styles if patched)
                            renderVDOM(data.vdom);
                        } catch (err) {
                            displayError('Rendering error: ' + err.message);
                            console.error('VDOM rendering failed:', err);
                        }
                    }
                } catch (err) {
                    displayError('Failed to parse server message');
                    console.error('Message parse error:', err);
                }
            };

            ws.onerror = (error) => {
                console.error('WebSocket error:', error);
            };

            ws.onclose = () => {
                console.log('Disconnected from preview server');
                statusDot.classList.add('disconnected');
                statusText.textContent = 'Disconnected - Reconnecting...';

                setTimeout(connect, reconnectDelay);
                reconnectDelay = Math.min(reconnectDelay * 1.5, 10000);
            };
        }

        function renderVDOM(vdom) {
            // Simple VDOM renderer - converts VDOM to actual DOM
            preview.innerHTML = '';

            if (!vdom || !vdom.nodes || vdom.nodes.length === 0) {
                preview.innerHTML = '<p style="color: #666;">No content to display</p>';
                return;
            }

            // Render styles
            renderStyles(vdom.styles || []);

            // Render nodes
            vdom.nodes.forEach(node => {
                const element = renderNode(node);
                if (element) {
                    preview.appendChild(element);
                }
            });
        }

        function applyCssPatches(patches) {
            // Apply CSS patches incrementally
            for (const patch of patches) {
                const key = `${patch.selector || ''}|${patch.media_query || ''}`;

                if (patch.type === 'Add') {
                    // Add new rule
                    currentCssRules.push(patch.rule);
                } else if (patch.type === 'Update') {
                    // Update existing rule
                    const index = currentCssRules.findIndex(r =>
                        r.selector === patch.selector &&
                        (r.media_query || '') === (patch.media_query || '')
                    );
                    if (index >= 0) {
                        currentCssRules[index].properties = patch.properties;
                    }
                } else if (patch.type === 'Remove') {
                    // Remove rule
                    const index = currentCssRules.findIndex(r =>
                        r.selector === patch.selector &&
                        (r.media_query || '') === (patch.media_query || '')
                    );
                    if (index >= 0) {
                        currentCssRules.splice(index, 1);
                    }
                }
            }

            // Re-render all styles
            renderStyles(currentCssRules);
        }

        function renderStyles(styles) {
            // Store current rules
            if (styles && styles.length > 0) {
                currentCssRules = styles;
            }

            // Remove old styles
            const oldStyle = document.getElementById('paperclip-styles');
            if (oldStyle) {
                oldStyle.remove();
            }

            if (!currentCssRules || currentCssRules.length === 0) {
                return;
            }

            // Create new style tag
            const styleTag = document.createElement('style');
            styleTag.id = 'paperclip-styles';

            // Group rules by media query
            const regularRules = [];
            const mediaRules = {};

            for (const rule of currentCssRules) {
                if (rule.media_query) {
                    if (!mediaRules[rule.media_query]) {
                        mediaRules[rule.media_query] = [];
                    }
                    mediaRules[rule.media_query].push(rule);
                } else {
                    regularRules.push(rule);
                }
            }

            // Convert regular rules to CSS text
            let cssText = regularRules.map(rule => {
                const properties = Object.entries(rule.properties || {})
                    .map(([key, value]) => `  ${key}: ${value};`)
                    .join('\n');
                return `${rule.selector} {\n${properties}\n}`;
            }).join('\n\n');

            // Add media query rules
            for (const [mediaQuery, rules] of Object.entries(mediaRules)) {
                const mediaRulesText = rules.map(rule => {
                    const properties = Object.entries(rule.properties || {})
                        .map(([key, value]) => `  ${key}: ${value};`)
                        .join('\n');
                    return `  ${rule.selector} {\n  ${properties}\n  }`;
                }).join('\n\n  ');

                cssText += `\n\n${mediaQuery} {\n${mediaRulesText}\n}`;
            }

            styleTag.textContent = cssText;
            document.head.appendChild(styleTag);
        }

        function renderNode(node) {
            if (!node) return null;

            switch (node.type) {
                case 'Element': {
                    const el = document.createElement(node.tag || 'div');

                    // Apply attributes
                    if (node.attributes) {
                        Object.entries(node.attributes).forEach(([key, value]) => {
                            if (key === 'class') {
                                el.className = value;
                            } else if (key === 'style') {
                                el.style.cssText = value;
                            } else {
                                el.setAttribute(key, value);
                            }
                        });
                    }

                    // Render children
                    if (node.children) {
                        node.children.forEach(child => {
                            const childEl = renderNode(child);
                            if (childEl) {
                                el.appendChild(childEl);
                            }
                        });
                    }

                    return el;
                }

                case 'Text':
                    return document.createTextNode(node.content || '');

                case 'Comment':
                    return document.createComment(node.content || '');

                case 'Error': {
                    const el = document.createElement('div');
                    el.className = 'error';
                    el.textContent = `Error: ${node.message || 'Unknown error'}`;
                    return el;
                }

                default:
                    console.warn('Unknown node type:', node.type);
                    return null;
            }
        }

        // Start connection
        connect();
    </script>
</body>
</html>
"#;
