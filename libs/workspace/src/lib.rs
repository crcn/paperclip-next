//! Paperclip Workspace Server
//!
//! gRPC server for live editing with:
//! - File watching
//! - Real-time preview streaming
//! - Mutation application
//! - Undo/redo support

pub mod server;
pub mod proto;
pub mod watcher;
pub mod mutations;
pub mod serializer;

pub use server::WorkspaceServer;
