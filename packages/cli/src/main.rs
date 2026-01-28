mod commands;
mod config;

use clap::{Parser, Subcommand};
use colored::Colorize;
use commands::{compile, designer, init, CompileArgs, DesignerArgs, InitArgs};

/// Paperclip CLI - Visual component builder for the AI age
#[derive(Parser, Debug)]
#[command(name = "paperclip")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Initialize a new Paperclip project
    Init(InitArgs),

    /// Compile .pc files to target format
    Compile(CompileArgs),

    /// Start the visual designer (coming soon)
    Designer(DesignerArgs),

    #[cfg(feature = "vision")]
    /// Capture component screenshots
    Vision {
        #[command(subcommand)]
        command: VisionCommand,
    },
}

#[cfg(feature = "vision")]
#[derive(Subcommand, Debug)]
enum VisionCommand {
    /// Capture screenshots of components
    Capture {
        /// Input .pc file or directory
        input: std::path::PathBuf,

        /// Output directory for screenshots
        #[arg(short, long, default_value = "./vision")]
        output: std::path::PathBuf,

        /// Viewport size (mobile, tablet, desktop)
        #[arg(short, long, default_value = "desktop")]
        viewport: String,

        /// Device pixel ratio (1.0 = standard, 2.0 = retina)
        #[arg(short, long, default_value = "1.0")]
        scale: f64,
    },
}

#[cfg(feature = "vision")]
fn vision_capture(
    input: std::path::PathBuf,
    output: std::path::PathBuf,
    viewport_str: String,
    scale: f64,
) -> Result<(), anyhow::Error> {
    use paperclip_vision::{CaptureOptions, VisionCapture, Viewport};

    println!("🎥 {} Paperclip Vision", "Starting".green().bold());
    println!("   Input:  {}", input.display());
    println!("   Output: {}", output.display());
    println!();

    // Parse viewport
    let viewport = match viewport_str.as_str() {
        "mobile" => Viewport::Mobile,
        "tablet" => Viewport::Tablet,
        "desktop" => Viewport::Desktop,
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid viewport: {}. Use: mobile, tablet, or desktop",
                viewport_str
            ));
        }
    };

    // Create capture instance
    let capture = VisionCapture::new(output.clone())?;

    let mut options = CaptureOptions::default();
    options.viewport = viewport;
    options.scale = scale;

    // Capture file or directory
    if input.is_file() {
        println!("📸 Capturing file: {}", input.display());
        let screenshots = capture.capture_file(&input, options)?;

        for screenshot in screenshots {
            println!(
                "   {} {} → {}",
                "✓".green(),
                screenshot.view_name,
                screenshot.path.display()
            );
        }
    } else if input.is_dir() {
        println!("📸 Capturing directory: {}", input.display());

        // Find all .pc files
        let pc_files = find_pc_files(&input)?;
        println!("   Found {} .pc files", pc_files.len());
        println!();

        for file in pc_files {
            println!("   Capturing: {}", file.display());
            match capture.capture_file(&file, options.clone()) {
                Ok(screenshots) => {
                    for screenshot in screenshots {
                        println!(
                            "     {} {} → {}",
                            "✓".green(),
                            screenshot.view_name,
                            screenshot.path.file_name().unwrap().to_string_lossy()
                        );
                    }
                }
                Err(e) => {
                    println!("     {} Failed: {}", "✗".red(), e);
                }
            }
        }
    } else {
        return Err(anyhow::anyhow!("Input path does not exist: {}", input.display()));
    }

    println!();
    println!("✨ {} Vision capture complete!", "Done".green().bold());
    println!("   Output: {}", output.display());

    Ok(())
}

#[cfg(feature = "vision")]
fn find_pc_files(dir: &std::path::Path) -> Result<Vec<std::path::PathBuf>, std::io::Error> {
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

fn main() {
    let cli = Cli::parse();

    let cwd = std::env::current_dir()
        .expect("Cannot get current directory")
        .display()
        .to_string();

    let result = match cli.command {
        Command::Init(args) => init(args, &cwd),
        Command::Compile(args) => compile(args, &cwd),
        Command::Designer(args) => designer(args, &cwd),

        #[cfg(feature = "vision")]
        Command::Vision { command } => match command {
            VisionCommand::Capture { input, output, viewport, scale } => {
                vision_capture(input, output, viewport, scale)
            }
        },
    };

    if let Err(err) = result {
        eprintln!();
        eprintln!("{} {}", "Error:".red().bold(), err);
        eprintln!();
        std::process::exit(1);
    }
}
