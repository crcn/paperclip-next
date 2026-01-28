use crate::watcher::FileWatcher;
use crate::state::WorkspaceState;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, Stream};
use tonic::{Request, Response, Status};

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

    fn process_file(&self, file_path: &str) -> Result<Vec<paperclip_evaluator::VDocPatch>, String> {
        // Read file
        let full_path = self.root_dir.join(file_path);
        let source = std::fs::read_to_string(&full_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        // Update state and get patches
        let patches = self
            .state
            .lock()
            .unwrap()
            .update_file(full_path, source, &self.root_dir)
            .map_err(|e| format!("State update error: {:?}", e))?;

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
                };
                let _ = tx.send(Ok(update)).await;
            }
            Err(error) => {
                let update = PreviewUpdate {
                    file_path: root_path.clone(),
                    patches: vec![],
                    error: Some(error),
                    timestamp: chrono::Utc::now().timestamp_millis(),
                    version: 0,
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
