use std::env;
use std::process::{Command, ExitStatus};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let task = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    match task {
        "migrate" => migrate(),
        "dev" => dev(),
        "watch" => watch(),
        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }
        _ => {
            eprintln!("Unknown task: {}", task);
            print_help();
            std::process::exit(1);
        }
    }
}

fn print_help() {
    eprintln!(
        r#"
xtask - Development task runner for ForkForge

USAGE:
    cargo xtask <TASK>

TASKS:
    migrate    Run database migrations
    dev        Start API server in development mode
    watch      Run API and CLI in watch mode (requires cargo-watch)
    help       Show this help message
"#
    );
}

fn migrate() -> Result<()> {
    println!("Running database migrations...");

    let status = Command::new("cargo")
        .args(&["run", "--bin", "migrate"])
        .status()?;

    check_status(status)?;
    println!("✅ Migrations completed successfully");
    Ok(())
}

fn dev() -> Result<()> {
    println!("Starting API server in development mode...");

    let status = Command::new("cargo")
        .args(&["run", "--bin", "api"])
        .env("RUST_LOG", "debug")
        .env("FORKFORGE_PROFILE", "default")
        .status()?;

    check_status(status)
}

fn watch() -> Result<()> {
    if !command_exists("cargo-watch") {
        eprintln!("cargo-watch is not installed. Install it with:");
        eprintln!("  cargo install cargo-watch");
        std::process::exit(1);
    }

    println!("Starting API in watch mode...");

    let status = Command::new("cargo")
        .args(&[
            "watch",
            "-x",
            "run --bin api",
            "-w",
            "crates",
            "-w",
            "Cargo.toml",
            "-w",
            "config.toml",
        ])
        .env("RUST_LOG", "debug")
        .env("FORKFORGE_PROFILE", "default")
        .status()?;

    check_status(status)
}

fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn check_status(status: ExitStatus) -> Result<()> {
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    Ok(())
}
