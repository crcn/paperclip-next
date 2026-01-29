//! Paperclip Vision CLI
//!
//! A thin glue layer for capturing component screenshots.

use clap::{Parser, Subcommand};
use paperclip_vision::{CaptureOptions, Viewport, VisionCapture};
use std::path::PathBuf;
use tracing::{error, info};

#[derive(Parser)]
#[command(name = "paperclip-vision")]
#[command(about = "Screenshot capture for Paperclip components", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Capture screenshots of components
    Capture {
        /// Input .pc file or directory
        input: PathBuf,

        /// Output directory for screenshots
        #[arg(short, long, default_value = "./vision")]
        output: PathBuf,

        /// Viewport size (mobile, tablet, desktop)
        #[arg(short, long, default_value = "desktop")]
        viewport: String,

        /// Device pixel ratio (1.0 = standard, 2.0 = retina)
        #[arg(short, long, default_value = "1.0")]
        scale: f64,
    },
}

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Capture {
            input,
            output,
            viewport,
            scale,
        } => {
            if let Err(e) = run_capture(input, output, viewport, scale) {
                error!("Capture failed: {}", e);
                std::process::exit(1);
            }
        }
    }
}

fn run_capture(
    input: PathBuf,
    output: PathBuf,
    viewport_str: String,
    scale: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting Paperclip Vision capture");
    info!("Input: {}", input.display());
    info!("Output: {}", output.display());

    // Parse viewport
    let viewport = match viewport_str.as_str() {
        "mobile" => Viewport::Mobile,
        "tablet" => Viewport::Tablet,
        "desktop" => Viewport::Desktop,
        _ => {
            error!(
                "Invalid viewport: {}. Use: mobile, tablet, or desktop",
                viewport_str
            );
            std::process::exit(1);
        }
    };

    // Create capture instance
    let capture = VisionCapture::new(output)?;

    let mut options = CaptureOptions::default();
    options.viewport = viewport;
    options.scale = scale;

    // Capture file or directory
    if input.is_file() {
        info!("Capturing file: {}", input.display());
        let screenshots = capture.capture_file(&input, options)?;

        for screenshot in screenshots {
            info!(
                "✅ Captured {} -> {}",
                screenshot.view_name,
                screenshot.path.display()
            );
        }
    } else if input.is_dir() {
        info!("Capturing directory: {}", input.display());

        // Find all .pc files
        let pc_files = find_pc_files(&input)?;
        info!("Found {} .pc files", pc_files.len());

        for file in pc_files {
            info!("Capturing: {}", file.display());
            match capture.capture_file(&file, options.clone()) {
                Ok(screenshots) => {
                    for screenshot in screenshots {
                        info!(
                            "  ✅ {} -> {}",
                            screenshot.view_name,
                            screenshot.path.display()
                        );
                    }
                }
                Err(e) => {
                    error!("  ❌ Failed: {}", e);
                }
            }
        }
    } else {
        error!("Input path does not exist: {}", input.display());
        std::process::exit(1);
    }

    info!("✨ Vision capture complete!");
    Ok(())
}

fn find_pc_files(dir: &PathBuf) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut files = Vec::new();

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().map(|e| e == "pc").unwrap_or(false) {
            files.push(path);
        } else if path.is_dir() {
            files.extend(find_pc_files(&path)?);
        }
    }

    Ok(files)
}
