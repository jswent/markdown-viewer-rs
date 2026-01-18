mod daemon;
mod markdown;
mod server;
mod state;
mod template;
mod watcher;

use chrono::Utc;
use clap::{Parser, Subcommand};
use crossbeam_channel::unbounded;
use daemon::{daemonize, get_pid, DaemonizeResult};
use markdown::convert_markdown;
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use server::{run_server, MarkdownServer};
use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook::flag;
use state::{get_log_path, Instance, StateFile};
use std::fs;
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use template::build_html_page;
use watcher::watch_file;

#[derive(Parser, Debug)]
#[command(
    name = "mdview",
    version = "0.2.0",
    about = "A markdown viewer with live reload and GitHub styling",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to markdown file (runs in foreground, for backwards compatibility)
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start viewer in background daemon mode
    Serve {
        /// Path to the markdown file to view
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Don't open browser automatically
        #[arg(long)]
        no_open: bool,
    },

    /// Stop a running background instance
    Stop {
        /// Path to the markdown file
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },

    /// List all running instances
    List {
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },
}

/// Finds an available port starting from the specified port
fn find_available_port(start_port: u16, max_attempts: u16) -> Option<u16> {
    (start_port..start_port + max_attempts)
        .find(|port| TcpListener::bind(("127.0.0.1", *port)).is_ok())
}

/// Validate that a file exists and is readable
fn validate_file(file: &PathBuf) -> Result<PathBuf, String> {
    if !file.exists() {
        return Err(format!("File '{}' not found", file.display()));
    }
    if !file.is_file() {
        return Err(format!("'{}' is not a file", file.display()));
    }
    file.canonicalize()
        .map_err(|e| format!("Error resolving file path: {}", e))
}

/// Run the viewer in foreground mode (original behavior)
fn run_foreground(file: &PathBuf) {
    let file_path = match validate_file(file) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let port = match find_available_port(6914, 100) {
        Some(p) => p,
        None => {
            eprintln!("Error: Could not find an available port");
            std::process::exit(1);
        }
    };

    let content = match fs::read_to_string(&file_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            std::process::exit(1);
        }
    };

    let html_content = convert_markdown(&content);
    let filename = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Markdown");
    let initial_html = build_html_page(&html_content, filename);

    let (reload_tx, reload_rx) = unbounded();

    let file_path_arc: Arc<std::path::Path> = Arc::from(file_path.as_path());

    let base_dir = match file_path.parent() {
        Some(dir) => Arc::from(dir),
        None => {
            eprintln!("Error: Could not determine parent directory");
            std::process::exit(1);
        }
    };

    let server = Arc::new(MarkdownServer::new(
        initial_html,
        reload_rx,
        base_dir,
        file_path_arc,
    ));

    println!("Serving '{}' at http://localhost:{}", filename, port);

    let url = format!("http://localhost:{}", port);
    if let Err(e) = open::that(&url) {
        eprintln!("Warning: Could not open browser: {}", e);
        eprintln!("Please open {} manually", url);
    }

    let watcher_file_path = file_path.clone();
    let watcher_handle = std::thread::spawn(move || {
        if let Err(e) = watch_file(watcher_file_path, reload_tx) {
            eprintln!("File watcher error: {}", e);
        }
    });

    // Set up signal handlers using signal-hook
    let shutdown = Arc::new(AtomicBool::new(false));
    flag::register(SIGINT, Arc::clone(&shutdown)).expect("Failed to register SIGINT handler");
    flag::register(SIGTERM, Arc::clone(&shutdown)).expect("Failed to register SIGTERM handler");

    println!("Press Ctrl+C to stop the server");

    // Check for shutdown signal periodically in a separate thread
    let shutdown_clone = Arc::clone(&shutdown);
    std::thread::spawn(move || {
        while !shutdown_clone.load(Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        println!("\nShutting down server...");
        std::process::exit(0);
    });

    if let Err(e) = run_server(port, server) {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }

    let _ = watcher_handle.join();
}

/// Run the viewer as a background daemon
fn run_serve(file: &PathBuf, no_open: bool) {
    let file_path = match validate_file(file) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    // Load state and check for existing instance
    let mut state = match StateFile::load() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error loading state: {}", e);
            std::process::exit(1);
        }
    };

    // Clean up stale instances
    let stale = state.cleanup_stale();
    for inst in &stale {
        eprintln!(
            "Cleaned up stale instance for '{}'",
            inst.file_path.display()
        );
    }

    // Check if already running
    if let Some(existing) = state.get_instance(&file_path) {
        if StateFile::is_process_running(existing.pid) {
            println!(
                "Already serving '{}' at http://localhost:{}",
                file_path.display(),
                existing.port
            );
            println!("PID: {}", existing.pid);
            return;
        }
        // Remove stale entry
        state.remove_instance(&file_path);
    }

    // Save state after cleanup
    if !stale.is_empty() {
        if let Err(e) = state.save() {
            eprintln!("Warning: Could not save state: {}", e);
        }
    }

    // Find available port
    let port = match find_available_port(6914, 100) {
        Some(p) => p,
        None => {
            eprintln!("Error: Could not find an available port");
            std::process::exit(1);
        }
    };

    // Get log file path
    let log_path = match get_log_path(&file_path, port) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error determining log path: {}", e);
            std::process::exit(1);
        }
    };

    // Print info before forking
    let filename = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Markdown");
    let url = format!("http://localhost:{}", port);

    println!("Starting mdview daemon for '{}'", filename);
    println!("URL: {}", url);
    println!("Log: {}", log_path.display());

    // Daemonize
    match daemonize(&log_path) {
        Ok(DaemonizeResult::Parent) => {
            // Parent process - open browser and exit
            if !no_open {
                std::thread::sleep(std::time::Duration::from_millis(200));
                if let Err(e) = open::that(&url) {
                    eprintln!("Warning: Could not open browser: {}", e);
                }
            }
            return;
        }
        Ok(DaemonizeResult::Daemon) => {
            // Daemon process - continue running
        }
        Err(e) => {
            eprintln!("Error daemonizing: {}", e);
            std::process::exit(1);
        }
    }

    // === From here on, we're in the daemon process ===

    // Register in state file
    let instance = Instance {
        pid: get_pid(),
        port,
        file_path: file_path.clone(),
        started_at: Utc::now(),
        log_file: log_path,
    };

    let mut state = StateFile::load().unwrap_or_default();
    state.add_instance(instance);
    if let Err(e) = state.save() {
        eprintln!("Warning: Could not save state: {}", e);
    }

    // Log startup
    println!(
        "[{}] mdview daemon started for '{}'",
        Utc::now().format("%Y-%m-%d %H:%M:%S"),
        file_path.display()
    );
    println!("PID: {}", get_pid());
    println!("Port: {}", port);

    // Set up the server
    let content = match fs::read_to_string(&file_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            cleanup_on_shutdown(&file_path);
            std::process::exit(1);
        }
    };

    let html_content = convert_markdown(&content);
    let initial_html = build_html_page(&html_content, filename);

    let (reload_tx, reload_rx) = unbounded();

    let file_path_arc: Arc<std::path::Path> = Arc::from(file_path.as_path());

    let base_dir = match file_path.parent() {
        Some(dir) => Arc::from(dir),
        None => {
            eprintln!("Error: Could not determine parent directory");
            cleanup_on_shutdown(&file_path);
            std::process::exit(1);
        }
    };

    let server = Arc::new(MarkdownServer::new(
        initial_html,
        reload_rx,
        base_dir,
        file_path_arc.clone(),
    ));

    // Start file watcher
    let watcher_file_path = file_path.clone();
    std::thread::spawn(move || {
        if let Err(e) = watch_file(watcher_file_path, reload_tx) {
            eprintln!("File watcher error: {}", e);
        }
    });

    // Set up signal handlers for graceful shutdown
    let shutdown = Arc::new(AtomicBool::new(false));
    flag::register(SIGINT, Arc::clone(&shutdown)).expect("Failed to register SIGINT handler");
    flag::register(SIGTERM, Arc::clone(&shutdown)).expect("Failed to register SIGTERM handler");

    // Shutdown monitor thread
    let shutdown_clone = Arc::clone(&shutdown);
    let cleanup_path = file_path.clone();
    std::thread::spawn(move || {
        while !shutdown_clone.load(Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        println!(
            "[{}] Received shutdown signal",
            Utc::now().format("%Y-%m-%d %H:%M:%S")
        );
        cleanup_on_shutdown(&cleanup_path);
        std::process::exit(0);
    });

    println!(
        "[{}] Server running on http://localhost:{}",
        Utc::now().format("%Y-%m-%d %H:%M:%S"),
        port
    );

    // Run the server
    if let Err(e) = run_server(port, server) {
        eprintln!("Server error: {}", e);
        cleanup_on_shutdown(&file_path);
        std::process::exit(1);
    }
}

/// Clean up state file on shutdown
fn cleanup_on_shutdown(file_path: &std::path::Path) {
    if let Ok(mut state) = StateFile::load() {
        state.remove_instance(file_path);
        let _ = state.save();
    }
}

/// Stop a running background instance
fn run_stop(file: &PathBuf) {
    let file_path = match validate_file(file) {
        Ok(p) => p,
        Err(e) => {
            // File might have been deleted, try to canonicalize what we can
            match file.canonicalize() {
                Ok(p) => p,
                Err(_) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
    };

    let mut state = match StateFile::load() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error loading state: {}", e);
            std::process::exit(1);
        }
    };

    let instance = match state.get_instance(&file_path) {
        Some(i) => i.clone(),
        None => {
            eprintln!("No running instance found for '{}'", file_path.display());
            std::process::exit(1);
        }
    };

    // Send SIGTERM to the process
    let pid = Pid::from_raw(instance.pid);
    match kill(pid, Signal::SIGTERM) {
        Ok(()) => {
            println!("Sent stop signal to mdview (PID {})", instance.pid);
        }
        Err(nix::errno::Errno::ESRCH) => {
            println!(
                "Process {} not running (stale entry), cleaning up",
                instance.pid
            );
        }
        Err(e) => {
            eprintln!("Failed to stop process {}: {}", instance.pid, e);
        }
    }

    // Remove from state
    state.remove_instance(&file_path);
    if let Err(e) = state.save() {
        eprintln!("Warning: Could not save state: {}", e);
    }

    println!("Stopped serving '{}'", file_path.display());
}

/// List all running instances
fn run_list(json_output: bool) {
    let mut state = match StateFile::load() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error loading state: {}", e);
            std::process::exit(1);
        }
    };

    // Clean up stale instances
    let stale = state.cleanup_stale();
    if !stale.is_empty() {
        if let Err(e) = state.save() {
            eprintln!("Warning: Could not save state: {}", e);
        }
    }

    if json_output {
        // JSON output
        let instances: Vec<_> = state.all_instances().collect();
        match serde_json::to_string_pretty(&instances) {
            Ok(json) => println!("{}", json),
            Err(e) => {
                eprintln!("Error serializing to JSON: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    // Human-readable output
    let instances: Vec<_> = state.all_instances().collect();

    if instances.is_empty() {
        println!("No running mdview instances");
        return;
    }

    println!(
        "{:<6} {:<6} {:<20} {}",
        "PID", "PORT", "STARTED", "FILE"
    );
    println!("{}", "-".repeat(70));

    for inst in instances {
        let started = inst.started_at.format("%Y-%m-%d %H:%M:%S");
        let status = if StateFile::is_process_running(inst.pid) {
            ""
        } else {
            " (stale)"
        };
        println!(
            "{:<6} {:<6} {:<20} {}{}",
            inst.pid,
            inst.port,
            started,
            inst.file_path.display(),
            status
        );
    }
}

fn main() {
    let cli = Cli::parse();

    match (&cli.command, &cli.file) {
        // Subcommand provided (ignore any trailing file argument)
        (Some(Commands::Serve { file, no_open }), _) => {
            run_serve(file, *no_open);
        }
        (Some(Commands::Stop { file }), _) => {
            run_stop(file);
        }
        (Some(Commands::List { json }), _) => {
            run_list(*json);
        }
        // Legacy mode: file provided without subcommand
        (None, Some(file)) => {
            run_foreground(file);
        }
        // No arguments - show help
        (None, None) => {
            // Re-parse with --help to show usage
            let _ = Cli::try_parse_from(["mdview", "--help"]);
        }
    }
}
