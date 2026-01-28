use crate::config::Config;
use anyhow::{anyhow, Result};
use clap::Args;
use colored::Colorize;
use paperclip_compiler_css::compile_to_css;
use paperclip_compiler_html::{compile_to_html, CompileOptions as HtmlOptions};
use paperclip_compiler_react::{compile_definitions, compile_to_react, CompileOptions};
use paperclip_parser::parse;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Args)]
pub struct CompileArgs {
    /// Directory to compile (defaults to current directory)
    #[arg(default_value = ".")]
    pub path: String,

    /// Target format (react, html, css)
    #[arg(short, long, default_value = "react")]
    pub target: String,

    /// Output to stdout instead of files
    #[arg(long)]
    pub stdout: bool,

    /// Output directory (overrides config)
    #[arg(short, long)]
    pub out_dir: Option<String>,

    /// Generate TypeScript definitions
    #[arg(long)]
    pub typescript: bool,

    /// Watch for file changes
    #[arg(short, long)]
    pub watch: bool,
}

pub fn compile(args: CompileArgs, cwd: &str) -> Result<()> {
    let config = Config::load(cwd)?;
    let src_dir = config.get_src_dir(cwd);

    if !src_dir.exists() {
        return Err(anyhow!("Source directory does not exist: {:?}", src_dir));
    }

    println!("{}", "🔨 Compiling Paperclip files...".bright_blue().bold());

    // Find all .pc files
    let pc_files = find_pc_files(&src_dir)?;

    if pc_files.is_empty() {
        println!("{}", "⚠️  No .pc files found".yellow());
        return Ok(());
    }

    println!("Found {} files", pc_files.len());

    // Compile each file
    let mut success_count = 0;
    let mut error_count = 0;

    for pc_file in &pc_files {
        match compile_file(pc_file, &args, &src_dir, cwd) {
            Ok(output_path) => {
                success_count += 1;
                let relative_path = pc_file.strip_prefix(&src_dir).unwrap_or(pc_file);
                println!(
                    "  {} {} → {}",
                    "✓".green(),
                    relative_path.display(),
                    output_path
                );
            }
            Err(e) => {
                error_count += 1;
                let relative_path = pc_file.strip_prefix(&src_dir).unwrap_or(pc_file);
                eprintln!(
                    "  {} {} - {}",
                    "✗".red(),
                    relative_path.display(),
                    e.to_string().red()
                );
            }
        }
    }

    println!();
    if error_count == 0 {
        println!(
            "{} Compiled {} files successfully",
            "✅".green(),
            success_count
        );
    } else {
        println!(
            "{} Compiled {} files, {} errors",
            "⚠️".yellow(),
            success_count,
            error_count
        );
    }

    if args.watch {
        println!("\n{}", "👀 Watching for changes...".bright_blue());
        println!("{}", "(Watch mode not yet implemented)".dimmed());
    }

    Ok(())
}

fn find_pc_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("pc") {
            files.push(path.to_path_buf());
        }
    }

    Ok(files)
}

fn compile_file(
    file_path: &Path,
    args: &CompileArgs,
    src_dir: &Path,
    cwd: &str,
) -> Result<String> {
    // Read source file
    let source = fs::read_to_string(file_path)?;

    // Parse
    let document = parse(&source).map_err(|e| {
        // Use pretty error formatting
        use paperclip_parser::error::pretty;
        let file_name = file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        anyhow!("\n{}", pretty::format_error(&e, file_name, &source))
    })?;

    // Compile based on target
    let output = match args.target.as_str() {
        "react" => {
            let options = CompileOptions {
                use_typescript: args.typescript,
                include_css_imports: true,
            };
            compile_to_react(&document, options).map_err(|e| anyhow!(e))?
        }
        "css" => {
            compile_to_css(&document).map_err(|e| anyhow!(e.to_string()))?
        }
        "html" => {
            let options = HtmlOptions::default();
            compile_to_html(&document, options).map_err(|e| anyhow!(e))?
        }
        other => {
            return Err(anyhow!("Unknown target: {}", other));
        }
    };

    // Output
    if args.stdout {
        println!("{}", output);
        Ok("stdout".to_string())
    } else {
        // Determine output path
        let relative_path = file_path.strip_prefix(src_dir).unwrap_or(file_path);
        let out_dir = if let Some(ref out) = args.out_dir {
            PathBuf::from(cwd).join(out)
        } else {
            PathBuf::from(cwd).join("dist")
        };

        let extension = match args.target.as_str() {
            "react" => "jsx",
            "css" => "css",
            "html" => "html",
            _ => "txt",
        };

        let output_file = out_dir.join(relative_path).with_extension(extension);

        // Create output directory
        if let Some(parent) = output_file.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write output
        fs::write(&output_file, output)?;

        // Generate TypeScript definitions if requested
        if args.typescript && args.target == "react" {
            let defs = compile_definitions(&document, CompileOptions::default())
                .map_err(|e| anyhow!(e))?;
            let defs_file = output_file.with_extension("d.ts");
            fs::write(&defs_file, defs)?;
        }

        Ok(output_file.display().to_string())
    }
}
