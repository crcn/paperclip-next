use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WatcherError {
    #[error("Failed to create watcher: {0}")]
    CreateError(#[from] notify::Error),

    #[error("Watch error: {0}")]
    WatchError(String),
}

pub type WatcherResult<T> = Result<T, WatcherError>;

pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    receiver: Receiver<notify::Result<Event>>,
}

impl FileWatcher {
    pub fn new(path: PathBuf) -> WatcherResult<Self> {
        let (tx, rx) = channel();

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            Config::default(),
        )?;

        watcher.watch(&path, RecursiveMode::Recursive)?;

        Ok(Self {
            _watcher: watcher,
            receiver: rx,
        })
    }

    pub fn next_event(&self) -> Option<Event> {
        match self.receiver.recv() {
            Ok(Ok(event)) => Some(event),
            _ => None,
        }
    }

    pub fn try_next_event(&self) -> Option<Event> {
        match self.receiver.try_recv() {
            Ok(Ok(event)) => Some(event),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_file_watcher() {
        let temp_dir = std::env::temp_dir().join("paperclip_watcher_test");
        fs::create_dir_all(&temp_dir).unwrap();

        let watcher = FileWatcher::new(temp_dir.clone()).unwrap();

        // Create a test file
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            fs::write(temp_dir.join("test.pc"), "component Test {}").unwrap();
        });

        // Wait for event
        let event = watcher.next_event();
        assert!(event.is_some());
    }
}
