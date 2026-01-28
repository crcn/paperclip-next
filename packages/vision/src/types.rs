//! Core types for Paperclip Vision

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Normalized view specification extracted from @view doc comments
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ViewSpec {
    /// View name (e.g., "default", "hover", "disabled")
    pub name: String,

    /// Optional description
    pub description: Option<String>,

    /// Target viewport (defaults to Desktop)
    pub viewport: Viewport,

    /// Component name this view belongs to
    pub component_name: String,
}

impl ViewSpec {
    /// Create a default view for a component
    pub fn default_for(component_name: String) -> Self {
        Self {
            name: "default".to_string(),
            description: None,
            viewport: Viewport::Desktop,
            component_name,
        }
    }
}

/// Viewport dimensions for rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Viewport {
    /// Mobile: 375x667 (iPhone SE)
    Mobile,

    /// Tablet: 768x1024 (iPad)
    Tablet,

    /// Desktop: 1920x1080 (HD)
    Desktop,

    /// Custom dimensions (width, height)
    Custom(u32, u32),
}

impl Viewport {
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            Viewport::Mobile => (375, 667),
            Viewport::Tablet => (768, 1024),
            Viewport::Desktop => (1920, 1080),
            Viewport::Custom(w, h) => (*w, *h),
        }
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Viewport::Desktop
    }
}

/// Area to capture in screenshot
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaptureArea {
    /// Capture only the component's bounding box (default)
    ComponentBounds,

    /// Capture entire viewport
    Viewport,
}

impl Default for CaptureArea {
    fn default() -> Self {
        CaptureArea::ComponentBounds
    }
}

/// Image output format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageFormat {
    Png,
    Jpeg,
}

impl Default for ImageFormat {
    fn default() -> Self {
        ImageFormat::Png
    }
}

impl ImageFormat {
    pub fn extension(&self) -> &str {
        match self {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpg",
        }
    }
}

/// Options for screenshot capture
#[derive(Debug, Clone)]
pub struct CaptureOptions {
    /// Viewport size
    pub viewport: Viewport,

    /// Area to capture
    pub capture_area: CaptureArea,

    /// Output format
    pub format: ImageFormat,

    /// Device pixel ratio (1.0 = standard, 2.0 = retina)
    pub scale: f64,

    /// Whether to generate metadata JSON
    pub emit_metadata: bool,
}

impl Default for CaptureOptions {
    fn default() -> Self {
        Self {
            viewport: Viewport::Desktop,
            capture_area: CaptureArea::ComponentBounds,
            format: ImageFormat::Png,
            scale: 1.0,
            emit_metadata: true,
        }
    }
}

/// Result of a screenshot capture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Screenshot {
    /// View name (e.g., "default", "hover")
    pub view_name: String,

    /// Component name
    pub component_name: String,

    /// Output file path
    pub path: PathBuf,

    /// Image width in pixels
    pub width: u32,

    /// Image height in pixels
    pub height: u32,

    /// Viewport used for capture
    pub viewport: Viewport,

    /// Timestamp of capture
    pub timestamp: String,
}

/// Metadata manifest for a component's views
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentManifest {
    /// Component name
    pub component_name: String,

    /// Source file path
    pub source_path: PathBuf,

    /// List of captured views
    pub views: Vec<Screenshot>,

    /// Timestamp of manifest generation
    pub generated_at: String,
}

impl ComponentManifest {
    pub fn new(component_name: String, source_path: PathBuf) -> Self {
        Self {
            component_name,
            source_path,
            views: Vec::new(),
            generated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn add_view(&mut self, screenshot: Screenshot) {
        self.views.push(screenshot);
    }
}
