//! # Paperclip Vision
//!
//! Screenshot capture and visual documentation for Paperclip components.
//!
//! ## Design Philosophy
//!
//! This is **not** a visual testing framework or Storybook alternative.
//! This is **design truth extraction** - materializing canonical visual states
//! directly from Paperclip components.
//!
//! ## Core Principles
//!
//! - Views live with components (via doc comments)
//! - No runtime JS required
//! - Deterministic rendering
//! - Component-scoped capture (not full viewport)
//! - Outputs are treated as build artifacts
//!
//! ## Usage
//!
//! ```rust,no_run
//! use paperclip_vision::{VisionCapture, CaptureOptions, Viewport};
//! use std::path::PathBuf;
//!
//! let capture = VisionCapture::new(PathBuf::from("./vision")).unwrap();
//! let screenshots = capture.capture_file(
//!     &PathBuf::from("button.pc"),
//!     CaptureOptions::default()
//! ).unwrap();
//!
//! for screenshot in screenshots {
//!     println!("Captured: {} -> {}", screenshot.view_name, screenshot.path.display());
//! }
//! ```

mod capture;
mod parser;
mod renderer;
mod server;
mod types;

pub use capture::VisionCapture;
pub use types::{CaptureArea, CaptureOptions, ImageFormat, Screenshot, ViewSpec, Viewport};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum VisionError {
    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Render error: {0}")]
    Render(String),

    #[error("Capture error: {0}")]
    Capture(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Browser error: {0}")]
    Browser(String),

    #[error("Component not found: {0}")]
    ComponentNotFound(String),
}

pub type Result<T> = std::result::Result<T, VisionError>;
