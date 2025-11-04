/// Module for HTTP server with Server-Sent Events (SSE) support
use crate::markdown::convert_markdown;
use crate::template::build_html_page;
use crossbeam_channel::Receiver;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tiny_http::{Header, Request, Response, Server};

/// HTTP server with markdown rendering and SSE live reload
pub struct MarkdownServer {
    cache: Arc<Mutex<String>>,
    reload_rx: Receiver<()>,
}

impl MarkdownServer {
    /// Creates a new MarkdownServer instance
    ///
    /// # Arguments
    ///
    /// * `initial_html` - The initial HTML content to serve
    /// * `reload_rx` - Channel receiver for reload signals from the file watcher
    pub fn new(initial_html: String, reload_rx: Receiver<()>) -> Self {
        Self {
            cache: Arc::new(Mutex::new(initial_html)),
            reload_rx,
        }
    }

    /// Refreshes the cached HTML content by reading and rendering the markdown file
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the markdown file to read and render
    pub fn refresh_cache(&self, file_path: &Path) {
        match fs::read_to_string(file_path) {
            Ok(content) => {
                let html_content = convert_markdown(&content);
                let filename = file_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Markdown");
                let full_html = build_html_page(&html_content, filename);

                if let Ok(mut cache) = self.cache.lock() {
                    *cache = full_html;
                }
            }
            Err(e) => {
                eprintln!("Error reading file: {}", e);
            }
        }
    }

    /// Handles an HTTP request
    ///
    /// Routes requests to either serve HTML content or handle SSE connections
    ///
    /// # Arguments
    ///
    /// * `request` - The incoming HTTP request
    pub fn handle_request(&self, request: Request) {
        let url = request.url().to_string();

        if url == "/events" {
            self.handle_sse(request);
        } else {
            self.handle_html(request);
        }
    }

    /// Handles regular HTML requests by serving the cached content
    fn handle_html(&self, request: Request) {
        let html = self.cache.lock().unwrap().clone();

        let response = Response::from_string(html)
            .with_header(
                Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap(),
            )
            .with_header(Header::from_bytes(&b"Cache-Control"[..], &b"no-cache"[..]).unwrap());

        let _ = request.respond(response);
    }

    /// Handles Server-Sent Events (SSE) connections for live reload
    ///
    /// This function keeps the connection open and sends reload events when the file changes.
    /// It also sends periodic keepalive messages to prevent connection timeouts.
    fn handle_sse(&self, request: Request) {
        // Clone the receiver for this SSE connection
        let reload_rx = self.reload_rx.clone();

        // Create SSE response headers
        let response = Response::empty(200)
            .with_header(
                Header::from_bytes(&b"Content-Type"[..], &b"text/event-stream"[..]).unwrap(),
            )
            .with_header(Header::from_bytes(&b"Cache-Control"[..], &b"no-cache"[..]).unwrap())
            .with_header(Header::from_bytes(&b"Connection"[..], &b"keep-alive"[..]).unwrap())
            .with_header(
                Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap(),
            );

        // Upgrade to a data stream
        let mut stream = request.upgrade("text/event-stream", response);

        // Keep connection alive and send reload events
        loop {
            match reload_rx.recv_timeout(Duration::from_secs(30)) {
                Ok(_) => {
                    // File changed, send reload event
                    if write!(stream, "data: reload\n\n").is_err() {
                        // Connection closed by client
                        break;
                    }
                    if stream.flush().is_err() {
                        // Connection closed by client
                        break;
                    }
                }
                Err(_) => {
                    // Timeout - send keepalive comment
                    if write!(stream, ": keepalive\n\n").is_err() {
                        // Connection closed
                        break;
                    }
                    if stream.flush().is_err() {
                        break;
                    }
                }
            }
        }
    }
}

/// Runs the HTTP server on the specified port
///
/// This function blocks indefinitely, handling incoming requests in separate threads.
///
/// # Arguments
///
/// * `port` - The port to bind the server to
/// * `server` - The MarkdownServer instance to handle requests
/// * `file_path` - Path to the markdown file being served
///
/// # Returns
///
/// A Result indicating success or failure
pub fn run_server(
    port: u16,
    server: Arc<MarkdownServer>,
    file_path: Arc<Path>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let http_server = Server::http(format!("127.0.0.1:{}", port))?;

    for request in http_server.incoming_requests() {
        let server = Arc::clone(&server);
        let file_path = Arc::clone(&file_path);

        // Spawn a thread for each request
        std::thread::spawn(move || {
            // Check if this is a regular request (not SSE) and refresh cache
            if request.url() != "/events" {
                server.refresh_cache(&file_path);
            }
            server.handle_request(request);
        });
    }

    Ok(())
}
