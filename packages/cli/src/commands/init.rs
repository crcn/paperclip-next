use crate::config::{CompilerOption, Config, DEFAULT_CONFIG_NAME};
use anyhow::Result;
use clap::Args;
use colored::Colorize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct InitArgs {
    /// Target format (react, html, css, all)
    #[arg(short, long, default_value = "react")]
    pub target: String,

    /// Source directory
    #[arg(short, long, default_value = "src")]
    pub src_dir: String,

    /// Force overwrite existing config
    #[arg(short, long)]
    pub force: bool,
}

pub fn init(args: InitArgs, cwd: &str) -> Result<()> {
    let config_path = PathBuf::from(cwd).join(DEFAULT_CONFIG_NAME);

    // Check if config already exists
    if config_path.exists() && !args.force {
        println!(
            "{} {} already exists",
            "⚠️".yellow(),
            DEFAULT_CONFIG_NAME.bright_white()
        );
        println!("Use --force to overwrite");
        return Ok(());
    }

    println!(
        "{}",
        "📝 Initializing Paperclip project...".bright_blue().bold()
    );

    // Create source directory if it doesn't exist
    let src_dir = PathBuf::from(cwd).join(&args.src_dir);
    if !src_dir.exists() {
        fs::create_dir_all(&src_dir)?;
        println!("  {} Created {}/", "✓".green(), args.src_dir);
    }

    // Create example .pc file
    let example_file = src_dir.join("example.pc");
    if !example_file.exists() {
        let example_content = r#"public component Button {
    render button(type="button") {
        style {
            padding: 8px 16px
            background: #3366FF
            color: white
            border: none
            border-radius: 4px
            cursor: pointer
        }
        text "Click me"
    }
}
"#;
        fs::write(&example_file, example_content)?;
        println!("  {} Created example.pc", "✓".green());
    }

    // Determine emit targets
    let emit = match args.target.as_str() {
        "all" => vec!["react".to_string(), "html".to_string(), "css".to_string()],
        target => vec![target.to_string()],
    };

    // Create config
    let config = Config {
        src_dir: args.src_dir.clone(),
        module_dirs: vec!["node_modules".to_string()],
        compiler_options: vec![CompilerOption {
            emit,
            out_dir: Some("dist".to_string()),
        }],
    };

    // Write config file
    let config_json = serde_json::to_string_pretty(&config)?;
    fs::write(&config_path, config_json)?;

    println!("  {} Created {}", "✓".green(), DEFAULT_CONFIG_NAME);
    println!();
    println!("{}", "✅ Project initialized!".green().bold());
    println!();
    println!("Next steps:");
    println!("  1. Edit {}/example.pc", args.src_dir);
    println!("  2. Run: paperclip compile");
    println!("  3. Check output in dist/");

    Ok(())
}
