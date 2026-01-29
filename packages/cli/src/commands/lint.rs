use anyhow::Result;
use clap::Args;
use colored::Colorize;
use paperclip_linter::{lint_document, DiagnosticLevel, LintOptions};
use paperclip_parser::{parse_with_path, ParseError};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Args, Debug)]
pub struct LintArgs {
    /// Input .pc file or directory to lint
    pub input: PathBuf,

    /// Show all diagnostics including info level
    #[arg(short, long)]
    pub verbose: bool,

    /// Output format (text, json)
    #[arg(short, long, default_value = "text")]
    pub format: String,
}

pub fn lint(args: LintArgs, _cwd: &str) -> Result<()> {
    println!("🔍 {} Paperclip Linter", "Starting".green().bold());
    println!("   Input: {}", args.input.display());
    println!();

    let mut total_diagnostics = 0;
    let mut total_errors = 0;
    let mut total_warnings = 0;
    let mut files_checked = 0;

    if args.input.is_file() {
        let (file_diagnostics, file_errors, file_warnings) =
            lint_file(&args.input, args.verbose, &args.format)?;
        total_diagnostics += file_diagnostics;
        total_errors += file_errors;
        total_warnings += file_warnings;
        files_checked += 1;
    } else if args.input.is_dir() {
        // Find all .pc files
        let pc_files = find_pc_files(&args.input)?;
        println!("   Found {} .pc files", pc_files.len());
        println!();

        for file in pc_files {
            let (file_diagnostics, file_errors, file_warnings) =
                lint_file(&file, args.verbose, &args.format)?;
            total_diagnostics += file_diagnostics;
            total_errors += file_errors;
            total_warnings += file_warnings;
            files_checked += 1;
        }
    } else {
        return Err(anyhow::anyhow!(
            "Input path does not exist: {}",
            args.input.display()
        ));
    }

    println!();
    println!(
        "✨ {} Linting complete!",
        if total_errors > 0 {
            "Done".red().bold()
        } else {
            "Done".green().bold()
        }
    );
    println!("   Files checked: {}", files_checked);
    println!("   Total diagnostics: {}", total_diagnostics);

    if total_errors > 0 {
        println!("   {} {}", "Errors:".red(), total_errors);
    }
    if total_warnings > 0 {
        println!("   {} {}", "Warnings:".yellow(), total_warnings);
    }

    if total_errors == 0 && total_warnings == 0 {
        println!("   {} No issues found!", "✓".green());
    }

    // Exit with error code if there are errors
    if total_errors > 0 {
        std::process::exit(1);
    }

    Ok(())
}

fn lint_file(
    file_path: &Path,
    verbose: bool,
    format: &str,
) -> Result<(usize, usize, usize)> {
    let source = fs::read_to_string(file_path)?;

    // Parse the file
    let document = match parse_with_path(&source, &file_path.to_string_lossy()) {
        Ok(doc) => doc,
        Err(err) => {
            eprintln!(
                "{} Failed to parse {}: {}",
                "✗".red(),
                file_path.display(),
                format_parse_error(&err)
            );
            return Ok((0, 1, 0));
        }
    };

    // Run the linter
    let diagnostics = lint_document(&document, LintOptions::default());

    if diagnostics.is_empty() {
        if verbose {
            println!("{} {}", "✓".green(), file_path.display());
        }
        return Ok((0, 0, 0));
    }

    // Count errors and warnings
    let errors = diagnostics
        .iter()
        .filter(|d| matches!(d.level, DiagnosticLevel::Error))
        .count();
    let warnings = diagnostics
        .iter()
        .filter(|d| matches!(d.level, DiagnosticLevel::Warning))
        .count();

    // Output diagnostics
    if format == "json" {
        let json = serde_json::to_string_pretty(&diagnostics)?;
        println!("{}", json);
    } else {
        // Text format
        println!("{}", file_path.display());

        for diagnostic in &diagnostics {
            let level_str = match diagnostic.level {
                DiagnosticLevel::Error => "error".red().bold(),
                DiagnosticLevel::Warning => "warning".yellow().bold(),
                DiagnosticLevel::Info => "info".blue().bold(),
            };

            if !verbose && matches!(diagnostic.level, DiagnosticLevel::Info) {
                continue;
            }

            println!("  {} [{}] {}", level_str, diagnostic.rule, diagnostic.message);

            if let Some(suggestion) = &diagnostic.suggestion {
                println!("    {} {}", "💡".dimmed(), suggestion.dimmed());
            }
        }

        println!();
    }

    Ok((diagnostics.len(), errors, warnings))
}

fn find_pc_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().map(|e| e == "pc").unwrap_or(false) {
            files.push(path.to_path_buf());
        }
    }

    Ok(files)
}

fn format_parse_error(err: &ParseError) -> String {
    format!("{:?}", err)
}
