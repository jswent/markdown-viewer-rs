/// Module for HTTP server with Server-Sent Events (SSE) support
use crate::markdown::convert_markdown;
use crate::template::build_html_page;
use crossbeam_channel::Receiver;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tiny_http::{Header, Request, Response, Server};

/// HTTP server with markdown rendering and SSE live reload
pub struct MarkdownServer {
    cache: Arc<Mutex<String>>,
    reload_rx: Receiver<()>,
    base_dir: Arc<Path>,
    file_path: Arc<Path>,
}

impl MarkdownServer {
    /// Creates a new MarkdownServer instance
    ///
    /// # Arguments
    ///
    /// * `initial_html` - The initial HTML content to serve
    /// * `reload_rx` - Channel receiver for reload signals from the file watcher
    /// * `base_dir` - Directory containing the markdown file (for serving images)
    /// * `file_path` - Full path to the markdown file
    pub fn new(
        initial_html: String,
        reload_rx: Receiver<()>,
        base_dir: Arc<Path>,
        file_path: Arc<Path>,
    ) -> Self {
        Self {
            cache: Arc::new(Mutex::new(initial_html)),
            reload_rx,
            base_dir,
            file_path,
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
    /// Routes requests to either serve HTML content, handle SSE connections, or serve image files
    ///
    /// # Arguments
    ///
    /// * `request` - The incoming HTTP request
    pub fn handle_request(&self, request: Request) {
        let url = request.url().to_string();

        if url == "/events" {
            self.handle_sse(request);
        } else if Self::is_image_request(&url) {
            self.handle_image(request, &url);
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

    /// Checks if a URL path is requesting an image file
    fn is_image_request(url: &str) -> bool {
        let lower = url.to_lowercase();
        lower.ends_with(".png")
            || lower.ends_with(".jpg")
            || lower.ends_with(".jpeg")
            || lower.ends_with(".gif")
            || lower.ends_with(".svg")
            || lower.ends_with(".webp")
            || lower.ends_with(".bmp")
            || lower.ends_with(".ico")
    }

    /// Maps file extensions to MIME types for image serving
    fn get_content_type(path: &Path) -> &'static str {
        match path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_lowercase())
            .as_deref()
        {
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("gif") => "image/gif",
            Some("svg") => "image/svg+xml",
            Some("webp") => "image/webp",
            Some("bmp") => "image/bmp",
            Some("ico") => "image/x-icon",
            _ => "application/octet-stream",
        }
    }

    /// Safely resolves an image path relative to the base directory
    ///
    /// Returns None if the path is invalid or attempts directory traversal
    fn resolve_image_path(&self, url_path: &str) -> Option<PathBuf> {
        // Remove leading slash
        let path_str = url_path.trim_start_matches('/');

        // Prevent empty paths
        if path_str.is_empty() {
            return None;
        }

        // Construct the full path
        let full_path = self.base_dir.join(path_str);

        // Canonicalize both paths to resolve .. and symlinks
        let canonical_full = match full_path.canonicalize() {
            Ok(p) => p,
            Err(_) => return None, // File doesn't exist or can't be accessed
        };

        let canonical_base = match self.base_dir.canonicalize() {
            Ok(p) => p,
            Err(_) => return None,
        };

        // Ensure the resolved path is within base_dir (prevents traversal)
        if !canonical_full.starts_with(&canonical_base) {
            eprintln!("Security: Blocked path traversal attempt: {}", url_path);
            return None;
        }

        // Verify it's a file (not a directory)
        if !canonical_full.is_file() {
            return None;
        }

        Some(canonical_full)
    }

    /// Handles image file requests
    fn handle_image(&self, request: Request, url_path: &str) {
        // Resolve path safely
        let image_path = match self.resolve_image_path(url_path) {
            Some(path) => path,
            None => {
                // Return 404 for invalid/missing files
                let response = Response::from_string("404 Not Found")
                    .with_status_code(404)
                    .with_header(
                        Header::from_bytes(&b"Content-Type"[..], &b"text/plain"[..]).unwrap(),
                    );
                let _ = request.respond(response);
                return;
            }
        };

        // Read image file as binary data
        let image_data = match fs::read(&image_path) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Error reading image file {}: {}", image_path.display(), e);
                let response = Response::from_string("500 Internal Server Error")
                    .with_status_code(500)
                    .with_header(
                        Header::from_bytes(&b"Content-Type"[..], &b"text/plain"[..]).unwrap(),
                    );
                let _ = request.respond(response);
                return;
            }
        };

        // Send response with appropriate Content-Type
        let content_type = Self::get_content_type(&image_path);
        let response = Response::from_data(image_data)
            .with_header(
                Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()).unwrap(),
            )
            .with_header(
                Header::from_bytes(&b"Cache-Control"[..], &b"max-age=3600"[..]).unwrap(),
            );

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
            match reload_rx.recv_timeout(Duration::from_secs(15)) {
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
                    // Timeout - send keepalive as data message so client can detect it
                    if write!(stream, "data: keepalive\n\n").is_err() {
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
///
/// # Returns
///
/// A Result indicating success or failure
pub fn run_server(
    port: u16,
    server: Arc<MarkdownServer>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let http_server = Server::http(format!("127.0.0.1:{}", port))?;

    for request in http_server.incoming_requests() {
        let server = Arc::clone(&server);

        // Spawn a thread for each request
        std::thread::spawn(move || {
            let url = request.url();
            // Only refresh cache for HTML requests (not SSE or images)
            if url != "/events" && !MarkdownServer::is_image_request(url) {
                server.refresh_cache(&server.file_path);
            }
            server.handle_request(request);
        });
    }

    Ok(())
}
