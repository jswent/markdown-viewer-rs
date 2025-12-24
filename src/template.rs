/// Module for generating HTML templates with GitHub-style markdown rendering

/// Builds a complete HTML page with GitHub markdown styling and auto-reload functionality
///
/// # Arguments
///
/// * `markdown_html` - The rendered markdown content as HTML
/// * `title` - The page title (typically the filename)
///
/// # Returns
///
/// A complete HTML document as a String
pub fn build_html_page(markdown_html: &str, title: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="color-scheme" content="light dark">
    <title>{title}</title>
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/github-markdown-css/5.5.1/github-markdown.min.css">
    <style>
        html {{
            colors-cheme: light dark;
        }}
        .markdown-body {{
            box-sizing: border-box;
            min-width: 200px;
            max-width: 980px;
            margin: 0 auto;
            padding: 45px;
        }}
    </style>
</head>
<body>
    <div class="markdown-body">
        {content}
    </div>
    <script>
        (function() {{
            let eventSource = null;
            let reconnectAttempts = 0;
            let lastMessageTime = Date.now();
            let connectionCheckInterval = null;
            const MAX_RECONNECT_DELAY = 30000; // 30 seconds max delay
            const CONNECTION_TIMEOUT = 30000; // 30s without message = dead connection
            const KEEPALIVE_CHECK_INTERVAL = 5000; // Check every 5 seconds

            function connect() {{
                // Close existing connection if any
                if (eventSource) {{
                    eventSource.close();
                    eventSource = null;
                }}

                console.log('Connecting to SSE...');
                eventSource = new EventSource('/events');

                eventSource.onopen = function() {{
                    console.log('SSE connected');
                    reconnectAttempts = 0;
                    lastMessageTime = Date.now();
                }};

                eventSource.onmessage = function(event) {{
                    lastMessageTime = Date.now();
                    if (event.data === 'reload') {{
                        console.log('Reload signal received');
                        location.reload();
                    }} else if (event.data === 'keepalive') {{
                        // Keepalive received - connection is healthy
                        console.log('Keepalive received');
                    }}
                }};

                eventSource.onerror = function(error) {{
                    console.log('SSE error, connection state:', eventSource.readyState);

                    // readyState: 0 = CONNECTING, 1 = OPEN, 2 = CLOSED
                    if (eventSource.readyState === EventSource.CLOSED) {{
                        reconnect();
                    }}
                }};
            }}

            function reconnect() {{
                if (eventSource) {{
                    eventSource.close();
                    eventSource = null;
                }}

                // Exponential backoff with max delay
                const delay = Math.min(1000 * Math.pow(2, reconnectAttempts), MAX_RECONNECT_DELAY);
                reconnectAttempts++;

                console.log('Reconnecting in ' + delay + 'ms (attempt ' + reconnectAttempts + ')...');
                setTimeout(connect, delay);
            }}

            function checkConnectionHealth() {{
                const timeSinceLastMessage = Date.now() - lastMessageTime;

                // If we haven't received ANY message (keepalive or reload) in 30s, connection is dead
                if (timeSinceLastMessage > CONNECTION_TIMEOUT) {{
                    console.log('Connection appears dead (no messages for ' +
                                Math.round(timeSinceLastMessage / 1000) + 's), forcing reconnection...');
                    reconnect();
                }}
            }}

            // Start connection
            connect();

            // Periodically check connection health
            connectionCheckInterval = setInterval(checkConnectionHealth, KEEPALIVE_CHECK_INTERVAL);

            // Cleanup on page unload
            window.addEventListener('beforeunload', function() {{
                if (connectionCheckInterval) {{
                    clearInterval(connectionCheckInterval);
                }}
                if (eventSource) {{
                    eventSource.close();
                }}
            }});
        }})();
    </script>
</body>
</html>"#,
        title = title,
        content = markdown_html
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_html_page() {
        let html = build_html_page("<h1>Test</h1>", "Test Page");
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<h1>Test</h1>"));
        assert!(html.contains("Test Page"));
        assert!(html.contains("EventSource('/events')"));
        assert!(html.contains("github-markdown.min.css"));
    }
}
