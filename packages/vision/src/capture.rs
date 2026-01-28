//! Screenshot capture using headless Chrome

use crate::renderer::render_component_html;
use crate::server::start_disposable_server;
use crate::types::{
    CaptureArea, CaptureOptions, ComponentManifest, Screenshot, ViewSpec,
};
use crate::{Result, VisionError};
use headless_chrome::{Browser, LaunchOptions};
use paperclip_parser::ast::Document;
use std::path::{Path, PathBuf};

/// Main capture interface
pub struct VisionCapture {
    output_dir: PathBuf,
    browser: Browser,
}

impl VisionCapture {
    /// Create a new VisionCapture instance
    pub fn new(output_dir: PathBuf) -> Result<Self> {
        // Create output directory
        std::fs::create_dir_all(&output_dir)
            .map_err(|e| VisionError::Io(e))?;

        // Launch headless Chrome
        let browser = Browser::new(LaunchOptions {
            headless: true,
            window_size: Some((1920, 1080)),
            ..Default::default()
        })
        .map_err(|e| VisionError::Browser(e.to_string()))?;

        Ok(Self {
            output_dir,
            browser,
        })
    }

    /// Capture screenshots for all views in a file
    pub fn capture_file(
        &self,
        path: &Path,
        options: CaptureOptions,
    ) -> Result<Vec<Screenshot>> {
        let (doc, views) = crate::parser::load_views_from_file(path)?;

        let mut screenshots = Vec::new();
        let first_component_name = views.first().map(|v| v.component_name.clone());

        for view in &views {
            let screenshot = self.capture_view(&doc, view, &options)?;
            screenshots.push(screenshot);
        }

        // Generate manifest if requested
        if options.emit_metadata {
            let manifest = ComponentManifest {
                component_name: first_component_name.unwrap_or_default(),
                source_path: path.to_path_buf(),
                views: screenshots.clone(),
                generated_at: chrono::Utc::now().to_rfc3339(),
            };

            self.write_manifest(&manifest)?;
        }

        Ok(screenshots)
    }

    /// Capture a single view
    fn capture_view(
        &self,
        doc: &Document,
        view: &ViewSpec,
        options: &CaptureOptions,
    ) -> Result<Screenshot> {
        // Render component to HTML
        let html = render_component_html(doc, &view.component_name)?;

        // Start disposable server
        let (url, server_handle) = start_disposable_server(html)?;

        // Open in headless Chrome
        let tab = self.browser.new_tab()
            .map_err(|e| VisionError::Browser(e.to_string()))?;

        // Set viewport
        let (width, height) = view.viewport.dimensions();
        tab.set_bounds(headless_chrome::types::Bounds::Normal {
            left: Some(0),
            top: Some(0),
            width: Some(width as f64),
            height: Some(height as f64),
        })
        .map_err(|e| VisionError::Browser(e.to_string()))?;

        // Navigate to page
        tab.navigate_to(&url)
            .map_err(|e| VisionError::Browser(e.to_string()))?;

        // Wait for page load
        tab.wait_until_navigated()
            .map_err(|e| VisionError::Browser(e.to_string()))?;

        // Capture screenshot based on capture area
        let screenshot_data = match options.capture_area {
            CaptureArea::ComponentBounds => self.capture_component_bounds(&tab)?,
            CaptureArea::Viewport => self.capture_full_viewport(&tab)?,
        };

        // Determine output path
        let filename = format!(
            "{}.{}.{}",
            view.component_name,
            view.name,
            options.format.extension()
        );
        let output_path = self.output_dir.join(&filename);

        // Save image
        std::fs::write(&output_path, &screenshot_data)
            .map_err(|e| VisionError::Io(e))?;

        // Close tab
        tab.close(true)
            .map_err(|e| VisionError::Browser(e.to_string()))?;

        // Wait for server thread
        let _ = server_handle.join();

        Ok(Screenshot {
            view_name: view.name.clone(),
            component_name: view.component_name.clone(),
            path: output_path,
            width,
            height,
            viewport: view.viewport,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Capture screenshot of component bounding box
    fn capture_component_bounds(
        &self,
        tab: &headless_chrome::Tab,
    ) -> Result<Vec<u8>> {
        // Find element with data-pc-root attribute
        let element = tab
            .wait_for_element("[data-pc-root]")
            .map_err(|e| VisionError::Capture(e.to_string()))?;

        // Get bounding box via JavaScript
        // This is more reliable than trying to use box_model directly
        let script = r#"
            const el = document.querySelector('[data-pc-root]');
            const rect = el.getBoundingClientRect();
            JSON.stringify({
                x: rect.x,
                y: rect.y,
                width: rect.width,
                height: rect.height
            });
        "#;

        let bounds_json = tab
            .evaluate(script, false)
            .map_err(|e| VisionError::Capture(e.to_string()))?
            .value
            .ok_or_else(|| VisionError::Capture("Failed to get bounding box".to_string()))?;

        // Parse JSON
        let bounds: serde_json::Value = serde_json::from_str(
            bounds_json.as_str().ok_or_else(|| VisionError::Capture("Invalid bounds JSON".to_string()))?
        )
        .map_err(|e| VisionError::Capture(e.to_string()))?;

        let x = bounds["x"].as_f64().unwrap_or(0.0);
        let y = bounds["y"].as_f64().unwrap_or(0.0);
        let width = bounds["width"].as_f64().unwrap_or(100.0);
        let height = bounds["height"].as_f64().unwrap_or(100.0);

        // Capture screenshot of bounding box
        let screenshot_data = tab
            .capture_screenshot(
                headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png,
                None,
                Some(headless_chrome::protocol::cdp::Page::Viewport {
                    x,
                    y,
                    width,
                    height,
                    scale: 1.0,
                }),
                true,
            )
            .map_err(|e| VisionError::Capture(e.to_string()))?;

        Ok(screenshot_data)
    }

    /// Capture full viewport screenshot
    fn capture_full_viewport(
        &self,
        tab: &headless_chrome::Tab,
    ) -> Result<Vec<u8>> {
        let screenshot_data = tab
            .capture_screenshot(
                headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png,
                None,
                None,
                true,
            )
            .map_err(|e| VisionError::Capture(e.to_string()))?;

        Ok(screenshot_data)
    }

    /// Write manifest JSON
    fn write_manifest(&self, manifest: &ComponentManifest) -> Result<()> {
        let manifest_path = self.output_dir.join("manifest.json");
        let json = serde_json::to_string_pretty(manifest)
            .map_err(|e| VisionError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            )))?;

        std::fs::write(manifest_path, json)
            .map_err(|e| VisionError::Io(e))?;

        Ok(())
    }
}

impl Drop for VisionCapture {
    fn drop(&mut self) {
        // Browser cleanup is handled by headless_chrome's Drop
    }
}
