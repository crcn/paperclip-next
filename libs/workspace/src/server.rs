//! Workspace gRPC server implementation

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use paperclip_evaluator::{evaluate, GraphManager};
use paperclip_proto::virt::EvaluatedModule;

use crate::proto::*;
use crate::mutations::apply_mutation;
use crate::serializer::serialize_document;

/// History entry for undo/redo
#[derive(Clone)]
struct HistoryEntry {
    source: String,
    evaluated: EvaluatedModule,
}

/// Per-file state
struct FileState {
    source: String,
    evaluated: Option<EvaluatedModule>,
    history: Vec<HistoryEntry>,
    history_index: usize,
}

/// The workspace server state
pub struct WorkspaceServer {
    /// Project root directory
    root: PathBuf,
    
    /// Open files
    files: Arc<RwLock<HashMap<PathBuf, FileState>>>,
    
    /// Dependency graph
    graph: Arc<RwLock<GraphManager>>,
    
    /// Event subscribers
    subscribers: Arc<RwLock<Vec<mpsc::Sender<ServerEvent>>>>,
}

impl WorkspaceServer {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            files: Arc::new(RwLock::new(HashMap::new())),
            graph: Arc::new(RwLock::new(GraphManager::new())),
            subscribers: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Subscribe to server events
    pub async fn subscribe(&self) -> mpsc::Receiver<ServerEvent> {
        let (tx, rx) = mpsc::channel(100);
        self.subscribers.write().await.push(tx);
        rx
    }
    
    /// Broadcast an event to all subscribers
    async fn broadcast(&self, event: ServerEvent) {
        let subscribers = self.subscribers.read().await;
        for tx in subscribers.iter() {
            let _ = tx.send(event.clone()).await;
        }
    }
    
    /// Open a file
    pub async fn open_file(&self, request: OpenFileRequest) -> Result<OpenFileResponse, String> {
        let path = self.root.join(&request.path);
        
        // Read file content
        let source = tokio::fs::read_to_string(&path)
            .await
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        // Evaluate
        let evaluated = evaluate(&source, &request.path)
            .map_err(|e| format!("Evaluation error: {}", e))?;
        
        // Store in open files
        {
            let mut files = self.files.write().await;
            files.insert(path.clone(), FileState {
                source: source.clone(),
                evaluated: Some(evaluated.clone()),
                history: vec![HistoryEntry {
                    source: source.clone(),
                    evaluated: evaluated.clone(),
                }],
                history_index: 0,
            });
        }
        
        Ok(OpenFileResponse {
            path: request.path,
            source,
            evaluated: Some(evaluated),
        })
    }
    
    /// Apply mutations to a file
    pub async fn apply_mutations(&self, request: ApplyMutationsRequest) -> Result<MutationResult, String> {
        let path = self.root.join(&request.path);
        
        let mut files = self.files.write().await;
        let file_state = files.get_mut(&path)
            .ok_or_else(|| "File not open".to_string())?;
        
        // Parse current source
        let mut doc = paperclip_parser::parse(&file_state.source)
            .map_err(|e| format!("Parse error: {} errors", e.len()))?;
        
        // Apply each mutation
        for mutation in &request.mutations {
            apply_mutation(&mut doc, mutation)
                .map_err(|e| format!("Mutation error: {}", e))?;
        }
        
        // Serialize back to source
        let new_source = serialize_document(&doc, &file_state.source);
        
        // Re-evaluate
        let evaluated = evaluate(&new_source, &request.path)
            .map_err(|e| format!("Evaluation error: {}", e))?;
        
        // Update state and history
        file_state.source = new_source.clone();
        file_state.evaluated = Some(evaluated.clone());
        
        // Truncate future history and add new entry
        file_state.history.truncate(file_state.history_index + 1);
        file_state.history.push(HistoryEntry {
            source: new_source.clone(),
            evaluated: evaluated.clone(),
        });
        file_state.history_index = file_state.history.len() - 1;
        
        // Write to disk
        tokio::fs::write(&path, &new_source)
            .await
            .map_err(|e| format!("Failed to write file: {}", e))?;
        
        // Broadcast change
        self.broadcast(ServerEvent::FileChanged {
            path: request.path,
            source: new_source.clone(),
            evaluated: evaluated.clone(),
        }).await;
        
        Ok(MutationResult {
            success: true,
            new_source: Some(new_source),
            evaluated: Some(evaluated),
            error: None,
        })
    }
    
    /// Undo last change
    pub async fn undo(&self, request: UndoRequest) -> Result<UndoResponse, String> {
        let path = self.root.join(&request.path);
        
        let mut files = self.files.write().await;
        let file_state = files.get_mut(&path)
            .ok_or_else(|| "File not open".to_string())?;
        
        if file_state.history_index == 0 {
            return Ok(UndoResponse {
                success: false,
                source: None,
                evaluated: None,
            });
        }
        
        file_state.history_index -= 1;
        let entry = &file_state.history[file_state.history_index];
        
        file_state.source = entry.source.clone();
        file_state.evaluated = Some(entry.evaluated.clone());
        
        // Write to disk
        tokio::fs::write(&path, &entry.source)
            .await
            .map_err(|e| format!("Failed to write file: {}", e))?;
        
        // Broadcast change
        self.broadcast(ServerEvent::FileChanged {
            path: request.path,
            source: entry.source.clone(),
            evaluated: entry.evaluated.clone(),
        }).await;
        
        Ok(UndoResponse {
            success: true,
            source: Some(entry.source.clone()),
            evaluated: Some(entry.evaluated.clone()),
        })
    }
    
    /// Redo undone change
    pub async fn redo(&self, request: UndoRequest) -> Result<UndoResponse, String> {
        let path = self.root.join(&request.path);
        
        let mut files = self.files.write().await;
        let file_state = files.get_mut(&path)
            .ok_or_else(|| "File not open".to_string())?;
        
        if file_state.history_index >= file_state.history.len() - 1 {
            return Ok(UndoResponse {
                success: false,
                source: None,
                evaluated: None,
            });
        }
        
        file_state.history_index += 1;
        let entry = &file_state.history[file_state.history_index];
        
        file_state.source = entry.source.clone();
        file_state.evaluated = Some(entry.evaluated.clone());
        
        // Write to disk
        tokio::fs::write(&path, &entry.source)
            .await
            .map_err(|e| format!("Failed to write file: {}", e))?;
        
        // Broadcast change
        self.broadcast(ServerEvent::FileChanged {
            path: request.path,
            source: entry.source.clone(),
            evaluated: entry.evaluated.clone(),
        }).await;
        
        Ok(UndoResponse {
            success: true,
            source: Some(entry.source.clone()),
            evaluated: Some(entry.evaluated.clone()),
        })
    }
    
    /// Handle external file change (from file watcher)
    pub async fn on_file_changed(&self, path: PathBuf) -> Result<(), String> {
        let relative_path = path.strip_prefix(&self.root)
            .map_err(|_| "Path not in project")?
            .to_string_lossy()
            .to_string();
        
        // Read new content
        let source = tokio::fs::read_to_string(&path)
            .await
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        // Evaluate
        let evaluated = evaluate(&source, &relative_path)
            .map_err(|e| format!("Evaluation error: {}", e))?;
        
        // Update state if file is open
        {
            let mut files = self.files.write().await;
            if let Some(file_state) = files.get_mut(&path) {
                file_state.source = source.clone();
                file_state.evaluated = Some(evaluated.clone());
            }
        }
        
        // Invalidate dependent files in graph
        {
            let mut graph = self.graph.write().await;
            graph.invalidate(&path);
        }
        
        // Broadcast change
        self.broadcast(ServerEvent::FileChanged {
            path: relative_path,
            source,
            evaluated,
        }).await;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;
    use tokio::fs;

    #[tokio::test]
    async fn test_open_and_mutate() {
        // Create temp directory with a .pc file
        let temp = temp_dir().join("paperclip-test");
        fs::create_dir_all(&temp).await.unwrap();
        
        let file_path = temp.join("test.pc");
        fs::write(&file_path, r#"
public component Button {
    render button {
        text "Click me"
    }
}
"#).await.unwrap();
        
        let server = WorkspaceServer::new(temp.clone());
        
        // Open file
        let response = server.open_file(OpenFileRequest {
            path: "test.pc".to_string(),
        }).await.unwrap();
        
        assert!(response.evaluated.is_some());
        assert_eq!(response.evaluated.as_ref().unwrap().components.len(), 1);
        
        // Cleanup
        fs::remove_dir_all(&temp).await.ok();
    }
}
