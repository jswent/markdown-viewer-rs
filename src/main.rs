mod markdown;
mod server;
mod template;
mod watcher;

use clap::Parser;
use crossbeam_channel::unbounded;
use markdown::convert_markdown;
use server::{run_server, MarkdownServer};
use std::fs;
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::Arc;
use template::build_html_page;
use watcher::watch_file;

/// Command-line markdown viewer with live reload
#[derive(Parser, Debug)]
#[command(
    name = "mdview",
    version = "0.1.0",
    about = "A markdown viewer with live reload and GitHub styling",
    long_about = None
)]
struct Args {
    /// Path to the markdown file to view
    #[arg(value_name = "FILE")]
    file: PathBuf,
}

/// Finds an available port starting from the specified port
///
/// # Arguments
///
/// * `start_port` - The port number to start searching from
/// * `max_attempts` - Maximum number of ports to try
///
/// # Returns
///
/// Some(port) if an available port is found, None otherwise
fn find_available_port(start_port: u16, max_attempts: u16) -> Option<u16> {
    (start_port..start_port + max_attempts)
        .find(|port| TcpListener::bind(("127.0.0.1", *port)).is_ok())
}

fn main() {
    let args = Args::parse();

    // Validate that the file exists
    if !args.file.exists() {
        eprintln!("Error: File '{}' not found", args.file.display());
        std::process::exit(1);
    }

    if !args.file.is_file() {
        eprintln!("Error: '{}' is not a file", args.file.display());
        std::process::exit(1);
    }

    // Find an available port
    let port = match find_available_port(6914, 100) {
        Some(p) => p,
        None => {
            eprintln!("Error: Could not find an available port");
            std::process::exit(1);
        }
    };

    // Read and convert the initial markdown content
    let content = match fs::read_to_string(&args.file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            std::process::exit(1);
        }
    };

    let html_content = convert_markdown(&content);
    let filename = args
        .file
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Markdown");
    let initial_html = build_html_page(&html_content, filename);

    // Create channel for reload signals
    let (reload_tx, reload_rx) = unbounded();

    // Create the server
    let server = Arc::new(MarkdownServer::new(initial_html, reload_rx));

    // Get absolute path for the file
    let file_path = match args.file.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error resolving file path: {}", e);
            std::process::exit(1);
        }
    };

    let file_path_arc = Arc::from(file_path.as_path());

    // Print serving information
    println!("Serving '{}' at http://localhost:{}", filename, port);

    // Open browser
    let url = format!("http://localhost:{}", port);
    if let Err(e) = open::that(&url) {
        eprintln!("Warning: Could not open browser: {}", e);
        eprintln!("Please open {} manually", url);
    }

    // Start file watcher in a separate thread
    let watcher_file_path = file_path.clone();
    let watcher_handle = std::thread::spawn(move || {
        if let Err(e) = watch_file(watcher_file_path, reload_tx) {
            eprintln!("File watcher error: {}", e);
        }
    });

    // Set up Ctrl+C handler
    ctrlc::set_handler(move || {
        println!("\nShutting down server...");
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    println!("Press Ctrl+C to stop the server");

    // Run the server (blocks here)
    if let Err(e) = run_server(port, server, file_path_arc) {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }

    // Wait for watcher thread (though we shouldn't reach here normally)
    let _ = watcher_handle.join();
}
