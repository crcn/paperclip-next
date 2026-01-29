pub mod server;
pub mod state;
pub mod watcher;

#[cfg(test)]
mod tests_comprehensive;

pub use server::{proto, WorkspaceServer};
pub use state::{FileState, StateError, WorkspaceState};
pub use watcher::{FileWatcher, WatcherError, WatcherResult};

// Re-export asset types from bundle
pub use paperclip_bundle::{AssetReference, AssetType};
