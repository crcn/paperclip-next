use crate::watcher::FileWatcher;
use paperclip_evaluator::Evaluator;
use paperclip_parser::parse;
use std::path::PathBuf;
use std::pin::Pin;
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, Stream};
use tonic::{Request, Response, Status};

// Include generated proto code
pub mod proto {
    tonic::include_proto!("paperclip.workspace");
}

use proto::{
    workspace_service_server::{WorkspaceService, WorkspaceServiceServer},
    FileEvent, PreviewRequest, PreviewUpdate, WatchRequest,
};

pub struct WorkspaceServer {
    root_dir: PathBuf,
}

impl WorkspaceServer {
    pub fn new(root_dir: PathBuf) -> Self {
        Self { root_dir }
    }

    pub fn into_service(self) -> WorkspaceServiceServer<Self> {
        WorkspaceServiceServer::new(self)
    }

    fn process_file(&self, file_path: &str) -> Result<String, String> {
        // Read file
        let full_path = self.root_dir.join(file_path);
        let source = std::fs::read_to_string(&full_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        // Parse
        let doc = parse(&source).map_err(|e| format!("Parse error: {:?}", e))?;

        // Evaluate
        let mut evaluator = Evaluator::new();
        let vdoc = evaluator
            .evaluate(&doc)
            .map_err(|e| format!("Evaluation error: {:?}", e))?;

        // Serialize to JSON
        serde_json::to_string(&vdoc).map_err(|e| format!("Serialization error: {}", e))
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
        let file_path = req.file_path.clone();
        let root_dir = self.root_dir.clone();

        let (tx, rx) = mpsc::channel(100);

        // Send initial update
        match self.process_file(&file_path) {
            Ok(vdom_json) => {
                let update = PreviewUpdate {
                    file_path: file_path.clone(),
                    vdom_json,
                    error: None,
                    timestamp: chrono::Utc::now().timestamp_millis(),
                };
                let _ = tx.send(Ok(update)).await;
            }
            Err(error) => {
                let update = PreviewUpdate {
                    file_path: file_path.clone(),
                    vdom_json: String::new(),
                    error: Some(error),
                    timestamp: chrono::Utc::now().timestamp_millis(),
                };
                let _ = tx.send(Ok(update)).await;
            }
        }

        // Spawn watcher task
        tokio::spawn(async move {
            // Watch the directory containing the file
            let watch_path = root_dir.join(
                PathBuf::from(&file_path)
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
                            .map(|n| file_path.contains(n))
                            .unwrap_or(false)
                    });

                    if !is_our_file {
                        continue;
                    }

                    // Process file and send update
                    let full_path = root_dir.join(&file_path);
                    let source = match std::fs::read_to_string(&full_path) {
                        Ok(s) => s,
                        Err(e) => {
                            let update = PreviewUpdate {
                                file_path: file_path.clone(),
                                vdom_json: String::new(),
                                error: Some(format!("Failed to read file: {}", e)),
                                timestamp: chrono::Utc::now().timestamp_millis(),
                            };
                            if tx.send(Ok(update)).await.is_err() {
                                break;
                            }
                            continue;
                        }
                    };

                    let doc = match parse(&source) {
                        Ok(d) => d,
                        Err(e) => {
                            let update = PreviewUpdate {
                                file_path: file_path.clone(),
                                vdom_json: String::new(),
                                error: Some(format!("Parse error: {:?}", e)),
                                timestamp: chrono::Utc::now().timestamp_millis(),
                            };
                            if tx.send(Ok(update)).await.is_err() {
                                break;
                            }
                            continue;
                        }
                    };

                    let mut evaluator = Evaluator::new();
                    let vdoc = match evaluator.evaluate(&doc) {
                        Ok(v) => v,
                        Err(e) => {
                            let update = PreviewUpdate {
                                file_path: file_path.clone(),
                                vdom_json: String::new(),
                                error: Some(format!("Evaluation error: {:?}", e)),
                                timestamp: chrono::Utc::now().timestamp_millis(),
                            };
                            if tx.send(Ok(update)).await.is_err() {
                                break;
                            }
                            continue;
                        }
                    };

                    let vdom_json = match serde_json::to_string(&vdoc) {
                        Ok(j) => j,
                        Err(e) => {
                            let update = PreviewUpdate {
                                file_path: file_path.clone(),
                                vdom_json: String::new(),
                                error: Some(format!("Serialization error: {}", e)),
                                timestamp: chrono::Utc::now().timestamp_millis(),
                            };
                            if tx.send(Ok(update)).await.is_err() {
                                break;
                            }
                            continue;
                        }
                    };

                    let update = PreviewUpdate {
                        file_path: file_path.clone(),
                        vdom_json,
                        error: None,
                        timestamp: chrono::Utc::now().timestamp_millis(),
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

    type WatchFilesStream =
        Pin<Box<dyn Stream<Item = Result<FileEvent, Status>> + Send + 'static>>;

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
}
