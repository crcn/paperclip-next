pub mod ast_index;
pub mod crdt;
pub mod mutation_handler;
pub mod server;
pub mod state;
pub mod watcher;

#[cfg(test)]
mod crdt_integration_tests;
#[cfg(test)]
mod mutation_handler_tests;
#[cfg(test)]
mod tests_comprehensive;
#[cfg(test)]
mod tests_typing_simulation;
#[cfg(test)]
mod tests_e2e_typing;

pub use ast_index::{AstIndex, ConflictError, NodePosition, NodeType};
pub use crdt::{CrdtBroadcast, CrdtClient, CrdtDocument, CrdtError, CrdtSession, CrdtSessionManager};
pub use mutation_handler::{Mutation, MutationError, MutationHandler, MutationResult};
pub use server::{proto, BroadcastUpdate, WorkspaceServer};
pub use state::{convert_vdom_to_proto, FileState, StateError, WorkspaceState};
pub use watcher::{FileWatcher, WatcherError, WatcherResult};

// Re-export asset types from bundle
pub use paperclip_bundle::{AssetReference, AssetType};
