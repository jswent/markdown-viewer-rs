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
        body {{
            background-color: #ffffff;
        }}
        @media (prefers-color-scheme: dark) {{
            body {{
                background-color: #0d1117;
            }}
        }}
        .markdown-body {{
            box-sizing: border-box;
            min-width: 200px;
            max-width: 980px;
            margin: 0 auto;
            padding: 45px;
            border: 1px solid #d1d9e0;
            border-radius: 8px;
        }}
        @media (prefers-color-scheme: dark) {{
            .markdown-body {{
                border-color: #3d444d;
            }}
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
        /* GitHub-style markdown alerts */
        .markdown-body .markdown-alert {{
            padding: 8px 16px;
            margin-bottom: 16px;
            color: inherit;
            border-left: .25em solid var(--color-border-default);
        }}
        .markdown-body .markdown-alert > :first-child {{
            margin-top: 0;
        }}
        .markdown-body .markdown-alert > :last-child {{
            margin-bottom: 0;
        }}
        .markdown-body .markdown-alert-title {{
            display: flex;
            align-items: center;
            gap: 8px;
            font-weight: 500;
        }}
        .markdown-body .markdown-alert-title::before {{
            content: '';
            display: inline-block;
            width: 16px;
            height: 16px;
            flex-shrink: 0;
            background-repeat: no-repeat;
            background-position: center;
            background-size: 16px 16px;
        }}
        /* Note - blue */
        .markdown-body .markdown-alert-note {{
            border-left-color: var(--color-accent-emphasis);
        }}
        .markdown-body .markdown-alert-note .markdown-alert-title {{
            color: var(--color-accent-fg);
        }}
        .markdown-body .markdown-alert-note .markdown-alert-title::before {{
            background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16' width='16' height='16'%3E%3Cpath fill='%232f81f7' d='M0 8a8 8 0 1 1 16 0A8 8 0 0 1 0 8Zm8-6.5a6.5 6.5 0 1 0 0 13 6.5 6.5 0 0 0 0-13ZM6.5 7.75A.75.75 0 0 1 7.25 7h1a.75.75 0 0 1 .75.75v2.75h.25a.75.75 0 0 1 0 1.5h-2a.75.75 0 0 1 0-1.5h.25v-2h-.25a.75.75 0 0 1-.75-.75ZM8 6a1 1 0 1 1 0-2 1 1 0 0 1 0 2Z'/%3E%3C/svg%3E");
        }}
        /* Tip - green */
        .markdown-body .markdown-alert-tip {{
            border-left-color: var(--color-success-emphasis);
        }}
        .markdown-body .markdown-alert-tip .markdown-alert-title {{
            color: var(--color-success-fg);
        }}
        .markdown-body .markdown-alert-tip .markdown-alert-title::before {{
            background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16' width='16' height='16'%3E%3Cpath fill='%233fb950' d='M8 1.5c-2.363 0-4 1.69-4 3.75 0 .984.424 1.625.984 2.304l.214.253c.223.264.47.556.673.848.284.411.537.896.621 1.49a.75.75 0 0 1-1.484.211c-.04-.282-.163-.547-.37-.847a8.456 8.456 0 0 0-.542-.68c-.084-.1-.173-.205-.268-.32C3.201 7.75 2.5 6.766 2.5 5.25 2.5 2.31 4.863 0 8 0s5.5 2.31 5.5 5.25c0 1.516-.701 2.5-1.328 3.259-.095.115-.184.22-.268.319-.207.245-.383.453-.541.681-.208.3-.33.565-.37.847a.751.751 0 0 1-1.485-.212c.084-.593.337-1.078.621-1.489.203-.292.45-.584.673-.848.075-.088.147-.173.213-.253.561-.679.985-1.32.985-2.304 0-2.06-1.637-3.75-4-3.75ZM5.75 12h4.5a.75.75 0 0 1 0 1.5h-4.5a.75.75 0 0 1 0-1.5ZM6 15.25a.75.75 0 0 1 .75-.75h2.5a.75.75 0 0 1 0 1.5h-2.5a.75.75 0 0 1-.75-.75Z'/%3E%3C/svg%3E");
        }}
        /* Important - purple */
        .markdown-body .markdown-alert-important {{
            border-left-color: var(--color-done-emphasis);
        }}
        .markdown-body .markdown-alert-important .markdown-alert-title {{
            color: var(--color-done-fg);
        }}
        .markdown-body .markdown-alert-important .markdown-alert-title::before {{
            background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16' width='16' height='16'%3E%3Cpath fill='%23a371f7' d='M0 1.75C0 .784.784 0 1.75 0h12.5C15.216 0 16 .784 16 1.75v9.5A1.75 1.75 0 0 1 14.25 13H8.06l-2.573 2.573A1.458 1.458 0 0 1 3 14.543V13H1.75A1.75 1.75 0 0 1 0 11.25Zm1.75-.25a.25.25 0 0 0-.25.25v9.5c0 .138.112.25.25.25h2a.75.75 0 0 1 .75.75v2.19l2.72-2.72a.749.749 0 0 1 .53-.22h6.5a.25.25 0 0 0 .25-.25v-9.5a.25.25 0 0 0-.25-.25Zm7 2.25v2.5a.75.75 0 0 1-1.5 0v-2.5a.75.75 0 0 1 1.5 0ZM9 9a1 1 0 1 1-2 0 1 1 0 0 1 2 0Z'/%3E%3C/svg%3E");
        }}
        /* Warning - yellow */
        .markdown-body .markdown-alert-warning {{
            border-left-color: var(--color-attention-emphasis);
        }}
        .markdown-body .markdown-alert-warning .markdown-alert-title {{
            color: var(--color-attention-fg);
        }}
        .markdown-body .markdown-alert-warning .markdown-alert-title::before {{
            background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16' width='16' height='16'%3E%3Cpath fill='%23d29922' d='M6.457 1.047c.659-1.234 2.427-1.234 3.086 0l6.082 11.378A1.75 1.75 0 0 1 14.082 15H1.918a1.75 1.75 0 0 1-1.543-2.575Zm1.763.707a.25.25 0 0 0-.44 0L1.698 13.132a.25.25 0 0 0 .22.368h12.164a.25.25 0 0 0 .22-.368Zm.53 3.996v2.5a.75.75 0 0 1-1.5 0v-2.5a.75.75 0 0 1 1.5 0ZM9 11a1 1 0 1 1-2 0 1 1 0 0 1 2 0Z'/%3E%3C/svg%3E");
        }}
        /* Caution - red */
        .markdown-body .markdown-alert-caution {{
            border-left-color: var(--color-danger-emphasis);
        }}
        .markdown-body .markdown-alert-caution .markdown-alert-title {{
            color: var(--color-danger-fg);
        }}
        .markdown-body .markdown-alert-caution .markdown-alert-title::before {{
            background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16' width='16' height='16'%3E%3Cpath fill='%23f85149' d='M4.47.22A.749.749 0 0 1 5 0h6c.199 0 .389.079.53.22l4.25 4.25c.141.14.22.331.22.53v6a.749.749 0 0 1-.22.53l-4.25 4.25A.749.749 0 0 1 11 16H5a.749.749 0 0 1-.53-.22L.22 11.53A.749.749 0 0 1 0 11V5c0-.199.079-.389.22-.53Zm.84 1.28L1.5 5.31v5.38l3.81 3.81h5.38l3.81-3.81V5.31L10.69 1.5ZM8 4a.75.75 0 0 1 .75.75v3.5a.75.75 0 0 1-1.5 0v-3.5A.75.75 0 0 1 8 4Zm0 8a1 1 0 1 1 0-2 1 1 0 0 1 0 2Z'/%3E%3C/svg%3E");
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
