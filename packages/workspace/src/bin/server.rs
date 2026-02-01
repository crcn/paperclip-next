use axum::{
    extract::{Query, State},
    response::{Html, sse::{Event, Sse}},
    routing::get,
    Router,
};
use futures::stream::{self, Stream};
use paperclip_bundle::Bundle;
use paperclip_evaluator::Evaluator;
use paperclip_parser::parse_with_path;
use paperclip_workspace::{convert_vdom_to_proto, WorkspaceServer};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::Arc;
use tokio_stream::StreamExt;
use tonic::transport::Server;
use tower_http::cors::{CorsLayer, Any};
use tower_http::services::ServeDir;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parse arguments
    let args: Vec<String> = std::env::args().collect();
    let mut port: u16 = 50051;
    let mut http_port: u16 = 3030;
    let mut root_dir = std::env::current_dir().unwrap();
    let mut designer_dir: Option<PathBuf> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--port" | "-p" => {
                if i + 1 < args.len() {
                    port = args[i + 1].parse().expect("Invalid port number");
                    i += 2;
                } else {
                    eprintln!("--port requires a value");
                    std::process::exit(1);
                }
            }
            "--http-port" => {
                if i + 1 < args.len() {
                    http_port = args[i + 1].parse().expect("Invalid HTTP port number");
                    i += 2;
                } else {
                    eprintln!("--http-port requires a value");
                    std::process::exit(1);
                }
            }
            "--designer-dir" => {
                if i + 1 < args.len() {
                    designer_dir = Some(PathBuf::from(&args[i + 1]));
                    i += 2;
                } else {
                    eprintln!("--designer-dir requires a value");
                    std::process::exit(1);
                }
            }
            "--help" | "-h" => {
                println!("Usage: paperclip-server [OPTIONS] [ROOT_DIR]");
                println!();
                println!("Options:");
                println!("  -p, --port <PORT>       gRPC port to listen on (default: 50051)");
                println!("  --http-port <PORT>      HTTP port for designer (default: 8080)");
                println!("  --designer-dir <DIR>    Directory containing designer build");
                println!("  -h, --help              Show this help message");
                println!();
                println!("Arguments:");
                println!("  [ROOT_DIR]              Root directory for workspace (default: current dir)");
                std::process::exit(0);
            }
            arg if !arg.starts_with('-') => {
                root_dir = PathBuf::from(arg);
                i += 1;
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                std::process::exit(1);
            }
        }
    }

    println!("Starting Paperclip workspace server...");
    println!("Root directory: {:?}", root_dir);
    println!("gRPC listening on 127.0.0.1:{}", port);
    println!("HTTP listening on 127.0.0.1:{}", http_port);

    // Create workspace server (shared between gRPC and HTTP)
    let workspace = WorkspaceServer::new(root_dir.clone());
    let workspace_for_http = workspace.clone();
    let grpc_service = workspace.into_service();

    // Wrap with tonic-web for browser gRPC support
    let grpc_web_service = tonic_web::enable(grpc_service);

    // Start gRPC server
    let grpc_addr = format!("127.0.0.1:{}", port).parse()?;
    let grpc_handle = tokio::spawn(async move {
        Server::builder()
            .accept_http1(true)
            .layer(
                tower::ServiceBuilder::new()
                    .layer(CorsLayer::new()
                        .allow_origin(Any)
                        .allow_headers(Any)
                        .allow_methods(Any)
                        .expose_headers(Any))
            )
            .add_service(grpc_web_service)
            .serve(grpc_addr)
            .await
    });

    // Create shared state for HTTP handlers (includes workspace for broadcast subscription)
    let http_state = Arc::new(HttpState {
        root_dir: root_dir.clone(),
        workspace: workspace_for_http,
    });

    // Start HTTP server for designer
    let http_addr = format!("127.0.0.1:{}", http_port);

    let app = if let Some(designer_path) = designer_dir {
        // Serve static files from designer build directory + API routes
        Router::new()
            .route("/api/preview", get(preview_sse_handler))
            .with_state(http_state)
            .fallback_service(ServeDir::new(designer_path).append_index_html_on_directories(true))
            .layer(CorsLayer::permissive())
    } else {
        // Serve a placeholder page if no designer directory specified
        Router::new()
            .route("/api/preview", get(preview_sse_handler))
            .with_state(http_state)
            .route("/", get(|| async {
                Html(r#"
                    <!DOCTYPE html>
                    <html>
                    <head><title>Paperclip Designer</title></head>
                    <body>
                        <h1>Paperclip Designer</h1>
                        <p>No designer directory specified. Use --designer-dir to serve the designer app.</p>
                        <p>gRPC server is running.</p>
                    </body>
                    </html>
                "#)
            }))
            .layer(CorsLayer::permissive())
    };

    let http_handle = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(&http_addr).await.unwrap();
        axum::serve(listener, app).await
    });

    // Wait for both servers
    tokio::select! {
        result = grpc_handle => {
            if let Err(e) = result {
                eprintln!("gRPC server error: {}", e);
            }
        }
        result = http_handle => {
            if let Err(e) = result {
                eprintln!("HTTP server error: {}", e);
            }
        }
    }

    Ok(())
}

// ============================================================================
// HTTP API for Designer
// ============================================================================

struct HttpState {
    root_dir: PathBuf,
    workspace: WorkspaceServer,
}

#[derive(Debug, Deserialize)]
struct PreviewQuery {
    file: String,
}

#[derive(Debug, Serialize)]
struct PreviewEvent {
    file_path: String,
    patches: Vec<serde_json::Value>,
    error: Option<String>,
    timestamp: i64,
    version: u64,
}

/// SSE endpoint for streaming preview updates from broadcast channel
async fn preview_sse_handler(
    State(state): State<Arc<HttpState>>,
    Query(query): Query<PreviewQuery>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let file_path_str = query.file.clone();
    let root_dir = state.root_dir.clone();

    // Resolve full path for initial processing
    let full_path = if std::path::Path::new(&file_path_str).is_absolute() {
        std::path::PathBuf::from(&file_path_str)
    } else {
        root_dir.join(&file_path_str)
    };

    tracing::info!("Starting preview stream for: {:?}", full_path);

    // Subscribe to broadcast channel for updates from gRPC buffer changes
    let mut broadcast_rx = state.workspace.subscribe();

    // Process initial file state
    let initial_event = match process_file_to_json(&full_path, &root_dir) {
        Ok(patches) => {
            let event = PreviewEvent {
                file_path: file_path_str.clone(),
                patches,
                error: None,
                timestamp: chrono::Utc::now().timestamp_millis(),
                version: 1,
            };
            let json = serde_json::to_string(&event).unwrap_or_default();
            Event::default().data(json)
        }
        Err(e) => {
            let event = PreviewEvent {
                file_path: file_path_str.clone(),
                patches: vec![],
                error: Some(e.to_string()),
                timestamp: chrono::Utc::now().timestamp_millis(),
                version: 0,
            };
            let json = serde_json::to_string(&event).unwrap_or_default();
            Event::default().data(json)
        }
    };

    // Create stream: initial event + broadcast updates filtered by file path
    let initial_stream = stream::once(async move { Ok(initial_event) });

    let broadcast_stream = stream::unfold(
        (file_path_str.clone(), full_path.clone(), broadcast_rx),
        move |(file_path_str, full_path, mut rx)| async move {
            loop {
                match rx.recv().await {
                    Ok(update) => {
                        tracing::info!(
                            "[SSE] Received broadcast: {:?} (v{})",
                            update.file_path,
                            update.version
                        );

                        // Check if this update is for our file
                        // Match by file path (could be relative or absolute)
                        let update_path = std::path::PathBuf::from(&update.file_path);
                        let is_match = update.file_path == file_path_str
                            || update_path == full_path
                            || full_path.ends_with(&update.file_path)
                            || update_path.file_name() == full_path.file_name();

                        tracing::info!(
                            "[SSE] Matching: update={:?} vs sse={:?} match={}",
                            update.file_path,
                            file_path_str,
                            is_match
                        );

                        if is_match {
                            tracing::info!(
                                "[SSE] Forwarding update for {:?} (v{})",
                                update.file_path,
                                update.version
                            );

                            // Parse patches from JSON
                            let patches: Vec<serde_json::Value> =
                                serde_json::from_str(&update.patches_json).unwrap_or_default();

                            let event = PreviewEvent {
                                file_path: file_path_str.clone(),
                                patches,
                                error: update.error,
                                timestamp: chrono::Utc::now().timestamp_millis(),
                                version: update.version,
                            };
                            let json = serde_json::to_string(&event).unwrap_or_default();
                            let sse_event = Event::default().data(json);

                            return Some((Ok(sse_event), (file_path_str, full_path, rx)));
                        }
                        // Not our file, continue listening
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("[SSE] Subscriber lagged by {} messages", n);
                        // Continue listening
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        tracing::info!("[SSE] Broadcast channel closed");
                        return None;
                    }
                }
            }
        },
    );

    // Chain initial event with broadcast stream
    let combined_stream = initial_stream.chain(broadcast_stream);

    Sse::new(combined_stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
            .text("ping"),
    )
}

/// Process a file and return JSON patches (in proto format)
fn process_file_to_json(
    file_path: &std::path::Path,
    root_dir: &std::path::Path,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error + Send + Sync>> {
    // Read and parse file
    let source = std::fs::read_to_string(file_path)?;
    let path_str = file_path.to_string_lossy();
    let ast = parse_with_path(&source, &path_str)?;

    // Create bundle and add document
    let mut bundle = Bundle::new();
    bundle.add_document(file_path.to_path_buf(), ast);

    // Build dependencies for cross-file imports
    if let Err(e) = bundle.build_dependencies(root_dir) {
        tracing::warn!(error = ?e, "Failed to build dependencies, continuing with single-file evaluation");
    }

    // Evaluate using bundle
    let mut evaluator = Evaluator::with_document_id(&path_str);
    let vdom = evaluator.evaluate_bundle(&bundle, file_path)?;

    // Convert internal VDOM to proto format for JSON serialization
    // This produces {"element": {...}} format instead of {"type": "Element", ...}
    let proto_vdom = convert_vdom_to_proto(&vdom);
    let vdom_json = serde_json::to_value(&proto_vdom)?;

    let patch = serde_json::json!({
        "initialize": {
            "vdom": vdom_json
        }
    });

    Ok(vec![patch])
}
