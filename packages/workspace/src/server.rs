use crate::state::{StateError, WorkspaceState};
use crate::watcher::FileWatcher;
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::{Arc, Mutex, atomic::{AtomicUsize, Ordering}};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
use tokio::time::timeout;
use tokio_stream::{wrappers::ReceiverStream, Stream, StreamExt as _};
use tonic::{Request, Response, Status};
use unicode_normalization::UnicodeNormalization;

// Production limits
const MAX_CLIENT_STATES: usize = 100;
const MAX_TOTAL_VDOM_BYTES: usize = 500 * 1024 * 1024;  // 500MB
const CLIENT_TIMEOUT_SECS: u64 = 300;  // 5 minutes
const PARSE_TIMEOUT_SECS: u64 = 5;
const MAX_CONTENT_SIZE: usize = 10 * 1024 * 1024;  // 10MB
const RATE_LIMIT_PER_PROCESS: usize = 100;  // per minute

/// Helper to convert various errors to Status
fn to_status<E: std::fmt::Display>(e: E) -> Status {
    Status::internal(e.to_string())
}

// Include generated proto code
// workspace.proto uses extern_path to reference evaluator's patches
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/paperclip.workspace.rs"));
}

use proto::{
    workspace_service_server::{WorkspaceService, WorkspaceServiceServer},
    FileEvent, PreviewRequest, PreviewUpdate, WatchRequest,
};

// Client state for buffer streaming
#[derive(Clone)]
struct ClientState {
    vdom_size: usize,
    version: u64,
    last_update: Instant,
}

// Rate limiter per process
struct ProcessRateLimiter {
    requests_per_process: HashMap<u32, VecDeque<Instant>>,
    max_requests_per_minute: usize,
}

impl ProcessRateLimiter {
    fn new(max_requests_per_minute: usize) -> Self {
        Self {
            requests_per_process: HashMap::new(),
            max_requests_per_minute,
        }
    }

    fn check(&mut self, pid: u32) -> Result<(), Status> {
        let now = Instant::now();
        let requests = self.requests_per_process.entry(pid).or_default();

        // Remove requests older than 1 minute
        requests.retain(|&time| now.duration_since(time) < Duration::from_secs(60));

        if requests.len() >= self.max_requests_per_minute {
            return Err(Status::resource_exhausted(
                format!("Process {} exceeded rate limit", pid)
            ));
        }

        requests.push_back(now);
        Ok(())
    }
}

/// Update broadcast for SSE subscribers
#[derive(Clone, Debug)]
pub struct BroadcastUpdate {
    pub file_path: String,
    pub patches_json: String,
    pub error: Option<String>,
    pub version: u64,
}

#[derive(Clone)]
pub struct WorkspaceServer {
    root_dir: PathBuf,
    root_dir_canonical: PathBuf,
    state: Arc<Mutex<WorkspaceState>>,
    // Buffer streaming state
    client_states: Arc<Mutex<HashMap<String, ClientState>>>,
    client_heartbeats: Arc<Mutex<HashMap<String, Instant>>>,
    total_vdom_bytes: Arc<AtomicUsize>,
    rate_limiter: Arc<Mutex<ProcessRateLimiter>>,
    // Broadcast channel for SSE subscribers
    update_sender: tokio::sync::broadcast::Sender<BroadcastUpdate>,
}

impl WorkspaceServer {
    pub fn new(root_dir: PathBuf) -> Self {
        let root_dir_canonical = root_dir
            .canonicalize()
            .unwrap_or_else(|_| root_dir.clone());

        // Create broadcast channel for SSE subscribers (capacity 100 messages)
        let (update_sender, _) = tokio::sync::broadcast::channel(100);

        let server = Self {
            root_dir,
            root_dir_canonical,
            state: Arc::new(Mutex::new(WorkspaceState::new())),
            client_states: Arc::new(Mutex::new(HashMap::new())),
            client_heartbeats: Arc::new(Mutex::new(HashMap::new())),
            total_vdom_bytes: Arc::new(AtomicUsize::new(0)),
            rate_limiter: Arc::new(Mutex::new(ProcessRateLimiter::new(RATE_LIMIT_PER_PROCESS))),
            update_sender,
        };

        // Start background cleanup task
        server.start_cleanup_task();

        server
    }

    /// Get root directory
    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    /// Subscribe to update broadcasts (for SSE)
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<BroadcastUpdate> {
        self.update_sender.subscribe()
    }

    /// Broadcast an update to all SSE subscribers
    fn broadcast_update(&self, update: BroadcastUpdate) {
        // Ignore send errors (no subscribers)
        let _ = self.update_sender.send(update);
    }

    fn start_cleanup_task(&self) {
        let heartbeats = self.client_heartbeats.clone();
        let states = self.client_states.clone();
        let state = self.state.clone();
        let total_bytes = self.total_vdom_bytes.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));

            loop {
                interval.tick().await;

                let now = Instant::now();
                let mut heartbeats = heartbeats.lock().unwrap();
                let mut client_states = states.lock().unwrap();
                let mut workspace_state = state.lock().unwrap();

                // Remove stale clients
                heartbeats.retain(|client_id, last_heartbeat| {
                    let is_stale = now.duration_since(*last_heartbeat)
                        > Duration::from_secs(CLIENT_TIMEOUT_SECS);

                    if is_stale {
                        if let Some(client_state) = client_states.remove(client_id) {
                            total_bytes.fetch_sub(client_state.vdom_size, Ordering::Relaxed);
                            tracing::warn!("Removed stale client: {}", client_id);
                        }
                        // Note: workspace state cleanup happens automatically
                        false
                    } else {
                        true
                    }
                });
            }
        });
    }

    fn validate_path(&self, file_path: &str) -> Result<PathBuf, Status> {
        // 1. Unicode normalization and sanitization
        let normalized = file_path
            .chars()
            .nfc()
            .collect::<String>()
            .replace('\u{2215}', "/")  // Division slash
            .replace('\u{2044}', "/"); // Fraction slash

        let path = PathBuf::from(&normalized);

        // 2. Canonicalize (follows symlinks)
        let canonical = path
            .canonicalize()
            .map_err(|e| Status::invalid_argument(format!("Path error: {}", e)))?;

        // 3. Check within workspace
        if !canonical.starts_with(&self.root_dir_canonical) {
            return Err(Status::permission_denied("Path escapes workspace"));
        }

        Ok(canonical)
    }

    fn ensure_capacity(&self, new_vdom_size: usize) -> Result<(), Status> {
        // Check total memory limit
        let current = self.total_vdom_bytes.load(Ordering::Relaxed);
        if current + new_vdom_size > MAX_TOTAL_VDOM_BYTES {
            return Err(Status::resource_exhausted(
                "Total VDOM memory limit exceeded"
            ));
        }

        // Check client count limit
        let states = self.client_states.lock().unwrap();
        if states.len() >= MAX_CLIENT_STATES {
            // Would need to evict LRU client
            return Err(Status::resource_exhausted("Too many active clients"));
        }

        Ok(())
    }

    pub fn into_service(self) -> WorkspaceServiceServer<Self> {
        WorkspaceServiceServer::new(self)
    }

    /// Create service from Arc (allows sharing with HTTP server)
    pub fn into_service_arc(self: Arc<Self>) -> WorkspaceServiceServer<Arc<Self>> {
        WorkspaceServiceServer::new(self)
    }

    fn process_file(
        &self,
        file_path: &str,
    ) -> Result<Vec<paperclip_evaluator::VDocPatch>, StateError> {
        // Read file
        let full_path = self.root_dir.join(file_path);
        let source = std::fs::read_to_string(&full_path)?;

        // Update state and get patches
        let patches = self
            .state
            .lock()
            .unwrap()
            .update_file(full_path, source, &self.root_dir)?;

        Ok(patches)
    }
}

#[tonic::async_trait]
impl WorkspaceService for WorkspaceServer {
    type StreamPreviewStream =
        Pin<Box<dyn Stream<Item = Result<PreviewUpdate, Status>> + Send + 'static>>;

    async fn stream_preview(
        &self,
        request: Request<PreviewRequest>,
    ) -> Result<Response<Self::StreamPreviewStream>, Status> {
        let req = request.into_inner();
        let root_path = req.root_path.clone();
        let root_dir = self.root_dir.clone();
        let state = self.state.clone();

        let (tx, rx) = mpsc::channel(100);

        // Send initial update
        match self.process_file(&root_path) {
            Ok(patches) => {
                let version = state
                    .lock()
                    .unwrap()
                    .get_file(&root_dir.join(&root_path))
                    .map(|s| s.version)
                    .unwrap_or(0);

                let update = PreviewUpdate {
                    file_path: root_path.clone(),
                    patches,
                    error: None,
                    timestamp: chrono::Utc::now().timestamp_millis(),
                    version,
                    acknowledged_mutation_ids: vec![],
                    changed_by_client_id: None,
                };
                let _ = tx.send(Ok(update)).await;
            }
            Err(error) => {
                let update = PreviewUpdate {
                    file_path: root_path.clone(),
                    patches: vec![],
                    error: Some(error.to_string()),
                    timestamp: chrono::Utc::now().timestamp_millis(),
                    version: 0,
                    acknowledged_mutation_ids: vec![],
                    changed_by_client_id: None,
                };
                let _ = tx.send(Ok(update)).await;
            }
        }

        // Spawn watcher task
        tokio::spawn(async move {
            // Watch the directory containing the file
            let watch_path = root_dir.join(
                PathBuf::from(&root_path)
                    .parent()
                    .unwrap_or(std::path::Path::new(".")),
            );

            let watcher = match FileWatcher::new(watch_path) {
                Ok(w) => w,
                Err(e) => {
                    tracing::error!("Failed to create watcher: {}", e);
                    return;
                }
            };

            loop {
                if let Some(event) = watcher.next_event() {
                    // Check if the event is for our file
                    let is_our_file = event.paths.iter().any(|p| {
                        p.file_name()
                            .and_then(|n| n.to_str())
                            .map(|n| root_path.contains(n))
                            .unwrap_or(false)
                    });

                    if !is_our_file {
                        continue;
                    }

                    // Process file and send update with patches
                    let full_path = root_dir.join(&root_path);
                    let source = match std::fs::read_to_string(&full_path) {
                        Ok(s) => s,
                        Err(e) => {
                            let update = PreviewUpdate {
                                file_path: root_path.clone(),
                                patches: vec![],
                                error: Some(format!("Failed to read file: {}", e)),
                                timestamp: chrono::Utc::now().timestamp_millis(),
                                version: 0,
                                acknowledged_mutation_ids: vec![],
                                changed_by_client_id: None,
                            };
                            if tx.send(Ok(update)).await.is_err() {
                                break;
                            }
                            continue;
                        }
                    };

                    // Update state and get patches
                    let result = {
                        let mut state_guard = state.lock().unwrap();
                        let patches = state_guard.update_file(full_path.clone(), source, &root_dir);
                        let version = state_guard
                            .get_file(&full_path)
                            .map(|s| s.version)
                            .unwrap_or(0);
                        (patches, version)
                    }; // Lock is dropped here

                    let (patches, version) = match result.0 {
                        Ok(p) => (p, result.1),
                        Err(e) => {
                            let update = PreviewUpdate {
                                file_path: root_path.clone(),
                                patches: vec![],
                                error: Some(format!("Processing error: {:?}", e)),
                                timestamp: chrono::Utc::now().timestamp_millis(),
                                version: 0,
                                acknowledged_mutation_ids: vec![],
                                changed_by_client_id: None,
                            };
                            if tx.send(Ok(update)).await.is_err() {
                                break;
                            }
                            continue;
                        }
                    };

                    let update = PreviewUpdate {
                        file_path: root_path.clone(),
                        patches,
                        error: None,
                        timestamp: chrono::Utc::now().timestamp_millis(),
                        version,
                        acknowledged_mutation_ids: vec![],
                        changed_by_client_id: None,
                    };

                    if tx.send(Ok(update)).await.is_err() {
                        break;
                    }
                }
            }
        });

        let output_stream = ReceiverStream::new(rx);
        Ok(Response::new(
            Box::pin(output_stream) as Self::StreamPreviewStream
        ))
    }

    type WatchFilesStream = Pin<Box<dyn Stream<Item = Result<FileEvent, Status>> + Send + 'static>>;

    async fn watch_files(
        &self,
        request: Request<WatchRequest>,
    ) -> Result<Response<Self::WatchFilesStream>, Status> {
        let req = request.into_inner();
        let watch_path = self.root_dir.join(&req.directory);

        let (tx, rx) = mpsc::channel(100);

        tokio::spawn(async move {
            let watcher = match FileWatcher::new(watch_path) {
                Ok(w) => w,
                Err(e) => {
                    tracing::error!("Failed to create watcher: {}", e);
                    return;
                }
            };

            loop {
                if let Some(event) = watcher.next_event() {
                    for path in event.paths {
                        let event_type = match event.kind {
                            notify::EventKind::Create(_) => 0, // CREATED
                            notify::EventKind::Modify(_) => 1, // MODIFIED
                            notify::EventKind::Remove(_) => 2, // DELETED
                            _ => continue,
                        };

                        let file_event = FileEvent {
                            event_type,
                            file_path: path.to_string_lossy().to_string(),
                            timestamp: chrono::Utc::now().timestamp_millis(),
                        };

                        if tx.send(Ok(file_event)).await.is_err() {
                            return;
                        }
                    }
                }
            }
        });

        let output_stream = ReceiverStream::new(rx);
        Ok(Response::new(
            Box::pin(output_stream) as Self::WatchFilesStream
        ))
    }

    async fn apply_mutation(
        &self,
        request: Request<proto::MutationRequest>,
    ) -> Result<Response<proto::MutationResponse>, Status> {
        let req = request.into_inner();

        // For now, return a stub ack response
        // TODO: Implement full mutation handling with CRDT integration
        let response = proto::MutationResponse {
            result: Some(proto::mutation_response::Result::Ack(proto::MutationAck {
                mutation_id: req
                    .mutation
                    .as_ref()
                    .and_then(|m| Some(m.mutation_id.clone()))
                    .unwrap_or_default(),
                new_version: req.expected_version + 1,
                timestamp: chrono::Utc::now().timestamp_millis(),
            })),
        };

        Ok(Response::new(response))
    }

    async fn get_document_outline(
        &self,
        request: Request<proto::OutlineRequest>,
    ) -> Result<Response<proto::OutlineResponse>, Status> {
        let req = request.into_inner();
        let file_path = self.root_dir.join(&req.file_path);

        // Read and parse file
        let source = std::fs::read_to_string(&file_path).map_err(to_status)?;

        let ast = paperclip_parser::parse(&source).map_err(to_status)?;

        // Build outline from AST
        let mut nodes = vec![];

        // Extract components and their children
        for component in ast.components.iter() {
            let component_id = component.span.id.clone();
            let mut child_ids = vec![];

            // Extract children from component body
            if let Some(body) = &component.body {
                extract_element_nodes(
                    body,
                    Some(&component_id),
                    &mut nodes,
                    &mut child_ids,
                    &source,
                );
            }

            nodes.push(proto::OutlineNode {
                node_id: component_id,
                r#type: proto::NodeType::Component as i32,
                parent_id: None,
                child_ids,
                span: Some(span_to_source_span(&component.span, &source)),
                label: Some(component.name.clone()),
            });
        }

        let version = self
            .state
            .lock()
            .unwrap()
            .get_file(&file_path)
            .map(|s| s.version)
            .unwrap_or(0);

        Ok(Response::new(proto::OutlineResponse { nodes, version }))
    }

    // NEW: Production-hardened buffer streaming
    type StreamBufferStream = Pin<Box<dyn Stream<Item = Result<PreviewUpdate, Status>> + Send + 'static>>;

    async fn stream_buffer(
        &self,
        request: Request<proto::BufferRequest>,
    ) -> Result<Response<Self::StreamBufferStream>, Status> {
        let req = request.into_inner();

        // Rate limiting (using process ID as proxy)
        let pid = std::process::id();
        self.rate_limiter.lock().unwrap().check(pid)?;

        // Validate content size
        if req.content.len() > MAX_CONTENT_SIZE {
            return Err(Status::invalid_argument("Content exceeds 10MB limit"));
        }

        // Note: For buffer streaming, we don't validate file path existence
        // since content is provided directly (not read from disk).
        // The file_path is used as an identifier only.

        // Update heartbeat
        self.client_heartbeats
            .lock()
            .unwrap()
            .insert(req.client_id.clone(), Instant::now());

        // Get previous state
        let prev_state = self
            .client_states
            .lock()
            .unwrap()
            .get(&req.client_id)
            .cloned();

        // Parse with timeout
        let content = req.content.clone();
        let parse_result = timeout(
            Duration::from_secs(PARSE_TIMEOUT_SECS),
            tokio::task::spawn_blocking(move || paperclip_parser::parse(&content))
        )
        .await;

        let ast = match parse_result {
            Ok(Ok(Ok(ast))) => ast,
            Ok(Ok(Err(e))) => {
                return Ok(Response::new(Box::pin(tokio_stream::once(Ok(
                    PreviewUpdate {
                        file_path: req.file_path,
                        patches: vec![],
                        error: Some(format!("Parse error: {}", e)),
                        timestamp: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as i64,
                        version: prev_state.map(|s| s.version).unwrap_or(0),
                        acknowledged_mutation_ids: vec![],
                        changed_by_client_id: None,
                    },
                ))) as Self::StreamBufferStream));
            }
            _ => {
                return Err(Status::deadline_exceeded("Parse timeout"));
            }
        };

        // Process through state (evaluate + diff)
        let root_dir = self.root_dir.clone();
        let source = req.content;
        let file_path_for_state = self.root_dir.join(&req.file_path);

        let result = {
            let mut state_guard = self.state.lock().unwrap();
            let patches = state_guard.update_file(file_path_for_state.clone(), source, &root_dir);
            let version = state_guard
                .get_file(&file_path_for_state)
                .map(|s| s.version)
                .unwrap_or(0);
            (patches, version)
        };

        let (patches, version) = match result.0 {
            Ok(p) => (p, result.1),
            Err(e) => {
                return Ok(Response::new(Box::pin(tokio_stream::once(Ok(
                    PreviewUpdate {
                        file_path: req.file_path,
                        patches: vec![],
                        error: Some(format!("Eval error: {:?}", e)),
                        timestamp: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as i64,
                        version: 0,
                        acknowledged_mutation_ids: vec![],
                        changed_by_client_id: None,
                    },
                ))) as Self::StreamBufferStream));
            }
        };

        // Rough VDOM size estimate
        let vdom_size = patches.len() * 500;

        // Check capacity
        self.ensure_capacity(vdom_size)?;

        // Update client state
        {
            let mut client_states = self.client_states.lock().unwrap();

            // Update memory tracking
            if let Some(prev_state) = prev_state {
                self.total_vdom_bytes
                    .fetch_sub(prev_state.vdom_size, Ordering::Relaxed);
            }
            self.total_vdom_bytes.fetch_add(vdom_size, Ordering::Relaxed);

            client_states.insert(
                req.client_id.clone(),
                ClientState {
                    vdom_size,
                    version,
                    last_update: Instant::now(),
                },
            );
        }

        // Serialize patches for broadcast
        let patches_json = serde_json::to_string(&patches).unwrap_or_default();

        // Broadcast to SSE subscribers
        self.broadcast_update(BroadcastUpdate {
            file_path: req.file_path.clone(),
            patches_json,
            error: None,
            version,
        });

        // Return stream with single update
        let update = PreviewUpdate {
            file_path: req.file_path,
            patches,
            error: None,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64,
            version,
            acknowledged_mutation_ids: vec![],
            changed_by_client_id: None,
        };

        Ok(Response::new(
            Box::pin(tokio_stream::once(Ok(update))) as Self::StreamBufferStream
        ))
    }

    async fn close_preview(
        &self,
        request: Request<proto::ClosePreviewRequest>,
    ) -> Result<Response<proto::ClosePreviewResponse>, Status> {
        let client_id = request.into_inner().client_id;

        let mut client_states = self.client_states.lock().unwrap();
        let mut heartbeats = self.client_heartbeats.lock().unwrap();
        let mut workspace_state = self.state.lock().unwrap();

        let existed = if let Some(client_state) = client_states.remove(&client_id) {
            self.total_vdom_bytes
                .fetch_sub(client_state.vdom_size, Ordering::Relaxed);
            heartbeats.remove(&client_id);
            // Note: workspace state cleanup happens automatically
            tracing::info!("Cleaned up state for client_id: {}", client_id);
            true
        } else {
            tracing::warn!("Attempted to close non-existent client_id: {}", client_id);
            false
        };

        Ok(Response::new(proto::ClosePreviewResponse {
            success: existed,
            message: if existed {
                Some("State cleaned up successfully".to_string())
            } else {
                Some("Client not found".to_string())
            },
        }))
    }

    async fn heartbeat(
        &self,
        request: Request<proto::HeartbeatRequest>,
    ) -> Result<Response<proto::HeartbeatResponse>, Status> {
        let client_id = request.into_inner().client_id;

        self.client_heartbeats
            .lock()
            .unwrap()
            .insert(client_id, Instant::now());

        Ok(Response::new(proto::HeartbeatResponse {
            acknowledged: true,
            server_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }))
    }
}

/// Convert byte offset span to line/column span
fn span_to_source_span(span: &paperclip_parser::ast::Span, source: &str) -> proto::SourceSpan {
    let (start_line, start_col) = byte_offset_to_line_col(source, span.start);
    let (end_line, end_col) = byte_offset_to_line_col(source, span.end);

    proto::SourceSpan {
        start_line: start_line as u32,
        start_col: start_col as u32,
        end_line: end_line as u32,
        end_col: end_col as u32,
    }
}

/// Convert byte offset to line and column (0-indexed)
fn byte_offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 0;
    let mut col = 0;
    let mut current_offset = 0;

    for ch in source.chars() {
        if current_offset >= offset {
            break;
        }

        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }

        current_offset += ch.len_utf8();
    }

    (line, col)
}

/// Extract outline nodes from an element tree
fn extract_element_nodes(
    element: &paperclip_parser::ast::Element,
    parent_id: Option<&str>,
    nodes: &mut Vec<proto::OutlineNode>,
    parent_child_ids: &mut Vec<String>,
    source: &str,
) {
    use paperclip_parser::ast::Element;

    match element {
        Element::Tag {
            tag_name,
            name,
            children,
            span,
            ..
        } => {
            let node_id = span.id.clone();
            let mut child_ids = vec![];

            // Recursively extract children
            for child in children {
                extract_element_nodes(child, Some(&node_id), nodes, &mut child_ids, source);
            }

            parent_child_ids.push(node_id.clone());

            nodes.push(proto::OutlineNode {
                node_id,
                r#type: proto::NodeType::Element as i32,
                parent_id: parent_id.map(|s| s.to_string()),
                child_ids,
                span: Some(span_to_source_span(span, source)),
                label: name.clone().or_else(|| Some(tag_name.clone())),
            });
        }
        Element::Text { content, span, .. } => {
            let node_id = span.id.clone();
            parent_child_ids.push(node_id.clone());

            nodes.push(proto::OutlineNode {
                node_id,
                r#type: proto::NodeType::Text as i32,
                parent_id: parent_id.map(|s| s.to_string()),
                child_ids: vec![],
                span: Some(span_to_source_span(span, source)),
                label: Some("text".to_string()),
            });
        }
        Element::Instance {
            name,
            children,
            span,
            ..
        } => {
            let node_id = span.id.clone();
            let mut child_ids = vec![];

            // Recursively extract children
            for child in children {
                extract_element_nodes(child, Some(&node_id), nodes, &mut child_ids, source);
            }

            parent_child_ids.push(node_id.clone());

            nodes.push(proto::OutlineNode {
                node_id,
                r#type: proto::NodeType::Element as i32,
                parent_id: parent_id.map(|s| s.to_string()),
                child_ids,
                span: Some(span_to_source_span(span, source)),
                label: Some(format!("<{}>", name)),
            });
        }
        Element::Conditional {
            then_branch,
            else_branch,
            span,
            ..
        } => {
            let node_id = span.id.clone();
            let mut child_ids = vec![];

            // Extract children from both branches
            for child in then_branch {
                extract_element_nodes(child, Some(&node_id), nodes, &mut child_ids, source);
            }
            if let Some(else_branch) = else_branch {
                for child in else_branch {
                    extract_element_nodes(child, Some(&node_id), nodes, &mut child_ids, source);
                }
            }

            parent_child_ids.push(node_id.clone());

            nodes.push(proto::OutlineNode {
                node_id,
                r#type: proto::NodeType::Conditional as i32,
                parent_id: parent_id.map(|s| s.to_string()),
                child_ids,
                span: Some(span_to_source_span(span, source)),
                label: Some("if".to_string()),
            });
        }
        Element::Repeat {
            item_name,
            body,
            span,
            ..
        } => {
            let node_id = span.id.clone();
            let mut child_ids = vec![];

            // Extract children from repeat body
            for child in body {
                extract_element_nodes(child, Some(&node_id), nodes, &mut child_ids, source);
            }

            parent_child_ids.push(node_id.clone());

            nodes.push(proto::OutlineNode {
                node_id,
                r#type: proto::NodeType::Repeat as i32,
                parent_id: parent_id.map(|s| s.to_string()),
                child_ids,
                span: Some(span_to_source_span(span, source)),
                label: Some(format!("repeat {}", item_name)),
            });
        }
        Element::Insert {
            slot_name,
            content,
            span,
        } => {
            let node_id = span.id.clone();
            let mut child_ids = vec![];

            // Extract children from insert content
            for child in content {
                extract_element_nodes(child, Some(&node_id), nodes, &mut child_ids, source);
            }

            parent_child_ids.push(node_id.clone());

            nodes.push(proto::OutlineNode {
                node_id,
                r#type: proto::NodeType::Insert as i32,
                parent_id: parent_id.map(|s| s.to_string()),
                child_ids,
                span: Some(span_to_source_span(span, source)),
                label: Some(format!("insert {}", slot_name)),
            });
        }
        Element::SlotInsert { name, span } => {
            let node_id = span.id.clone();
            parent_child_ids.push(node_id.clone());

            nodes.push(proto::OutlineNode {
                node_id,
                r#type: proto::NodeType::Insert as i32,
                parent_id: parent_id.map(|s| s.to_string()),
                child_ids: vec![],
                span: Some(span_to_source_span(span, source)),
                label: Some(format!("slot {}", name)),
            });
        }
    }
}
