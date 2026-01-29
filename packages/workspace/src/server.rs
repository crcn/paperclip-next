use crate::state::{StateError, WorkspaceState};
use crate::watcher::FileWatcher;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, Stream};
use tonic::{Request, Response, Status};

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

pub struct WorkspaceServer {
    root_dir: PathBuf,
    state: Arc<Mutex<WorkspaceState>>,
}

impl WorkspaceServer {
    pub fn new(root_dir: PathBuf) -> Self {
        Self {
            root_dir,
            state: Arc::new(Mutex::new(WorkspaceState::new())),
        }
    }

    pub fn into_service(self) -> WorkspaceServiceServer<Self> {
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
}

/// Convert byte offset span to line/column span
fn span_to_source_span(span: &paperclip_parser::Span, source: &str) -> proto::SourceSpan {
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
    element: &paperclip_parser::Element,
    parent_id: Option<&str>,
    nodes: &mut Vec<proto::OutlineNode>,
    parent_child_ids: &mut Vec<String>,
    source: &str,
) {
    use paperclip_parser::Element;

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
        Element::Text { content, span } => {
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
