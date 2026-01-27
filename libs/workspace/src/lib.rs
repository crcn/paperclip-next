pub mod server;
pub mod watcher;

#[cfg(test)]
mod tests_comprehensive;

pub use server::{proto, WorkspaceServer};
pub use watcher::{FileWatcher, WatcherError, WatcherResult};
