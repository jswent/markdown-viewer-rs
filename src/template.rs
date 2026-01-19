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
        /* Code block container for copy button positioning */
        pre {{
            position: relative;
        }}
        /* Copy button styling */
        .copy-button {{
            position: absolute;
            top: 8px;
            right: 8px;
            padding: 5px;
            padding-top: 6px;
            padding-bottom: 6px;
            background-color: transparent;
            border: none;
            border-radius: 6px;
            color: #848d97;
            cursor: pointer;
            display: flex;
            align-items: center;
            justify-content: center;
        }}
        .copy-button:hover {{
            background-color: #262c36;
            color: #c9d1d9;
        }}
        .copy-button .copy-icon {{
            display: block;
            margin-right: 0.2rem;
        }}
        .copy-button .check-icon {{
            display: none;
            color: #3fb950;
            margin-right: 0.2rem;
        }}
        .copy-button.copied .copy-icon {{
            display: none;
        }}
        .copy-button.copied .check-icon {{
            display: block;
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

        // Copy button functionality for code blocks
        (function() {{
            const copyIcon = '<svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" fill="currentColor" class="copy-icon"><path d="M0 6.75C0 5.784.784 5 1.75 5h1.5a.75.75 0 0 1 0 1.5h-1.5a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-1.5a.75.75 0 0 1 1.5 0v1.5A1.75 1.75 0 0 1 9.25 16h-7.5A1.75 1.75 0 0 1 0 14.25Z"></path><path d="M5 1.75C5 .784 5.784 0 6.75 0h7.5C15.216 0 16 .784 16 1.75v7.5A1.75 1.75 0 0 1 14.25 11h-7.5A1.75 1.75 0 0 1 5 9.25Zm1.75-.25a.25.25 0 0 0-.25.25v7.5c0 .138.112.25.25.25h7.5a.25.25 0 0 0 .25-.25v-7.5a.25.25 0 0 0-.25-.25Z"></path></svg>';
            const checkIcon = '<svg aria-hidden="true" height="16" viewBox="0 0 16 16" version="1.1" width="16" fill="currentColor" class="check-icon"><path d="M13.78 4.22a.75.75 0 0 1 0 1.06l-7.25 7.25a.75.75 0 0 1-1.06 0L2.22 9.28a.751.751 0 0 1 .018-1.042.751.751 0 0 1 1.042-.018L6 10.94l6.72-6.72a.75.75 0 0 1 1.06 0Z"></path></svg>';

            function initCopyButtons() {{
                document.querySelectorAll('pre').forEach(function(pre) {{
                    // Skip if button already exists (for live reload)
                    if (pre.querySelector('.copy-button')) return;

                    const button = document.createElement('button');
                    button.className = 'copy-button';
                    button.setAttribute('aria-label', 'Copy');
                    button.innerHTML = copyIcon + checkIcon;

                    button.addEventListener('click', function() {{
                        const code = pre.querySelector('code');
                        const text = code ? code.innerText : pre.innerText;

                        navigator.clipboard.writeText(text).then(function() {{
                            button.classList.add('copied');
                            setTimeout(function() {{
                                button.classList.remove('copied');
                            }}, 2000);
                        }}).catch(function(err) {{
                            console.error('Failed to copy:', err);
                        }});
                    }});

                    pre.appendChild(button);
                }});
            }}

            // Initialize on DOM ready
            if (document.readyState === 'loading') {{
                document.addEventListener('DOMContentLoaded', initCopyButtons);
            }} else {{
                initCopyButtons();
            }}
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
