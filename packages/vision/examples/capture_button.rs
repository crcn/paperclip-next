//! Example: Capture screenshots of a button component

use paperclip_vision::{CaptureOptions, Viewport, VisionCapture};
use std::path::PathBuf;
use tempfile::tempdir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create temporary directory for output
    let temp_dir = tempdir()?;
    let output_dir = temp_dir.path().to_path_buf();

    println!("Output directory: {}", output_dir.display());

    // Create a simple button component
    let button_pc = r#"
/// @view default
/// @view hover - Hover state with darker background
/// @viewport desktop
public component Button {
    render button {
        style {
            padding: 8px 16px
            background: #3366ff
            color: white
            border: none
            border-radius: 4px
            cursor: pointer
        }
        text "Click me"
    }
}
"#;

    // Write to temp file
    let pc_file = temp_dir.path().join("button.pc");
    std::fs::write(&pc_file, button_pc)?;

    // Create capture instance
    let capture = VisionCapture::new(output_dir.clone())?;

    // Capture with desktop viewport
    let mut options = CaptureOptions::default();
    options.viewport = Viewport::Desktop;

    println!("Capturing button.pc...");
    let screenshots = capture.capture_file(&pc_file, options)?;

    println!("\n✨ Captured {} views:", screenshots.len());
    for screenshot in &screenshots {
        println!(
            "  - {} ({}x{})",
            screenshot.view_name, screenshot.width, screenshot.height
        );
        println!("    → {}", screenshot.path.display());
    }

    // Capture with mobile viewport
    let mut mobile_options = CaptureOptions::default();
    mobile_options.viewport = Viewport::Mobile;

    println!("\nCapturing with mobile viewport...");
    let mobile_screenshots = capture.capture_file(&pc_file, mobile_options)?;

    println!("✨ Captured {} mobile views:", mobile_screenshots.len());
    for screenshot in &mobile_screenshots {
        println!(
            "  - {} ({}x{})",
            screenshot.view_name, screenshot.width, screenshot.height
        );
    }

    println!("\n📁 All files saved to: {}", output_dir.display());
    println!("📄 Manifest: {}/manifest.json", output_dir.display());

    // Keep temp dir alive for inspection
    println!("\n⚠️  Temp directory will be cleaned up on exit");
    println!("Press Enter to exit...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    Ok(())
}
