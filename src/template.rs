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
        const eventSource = new EventSource('/events');
        eventSource.onmessage = (event) => {{
            if (event.data === 'reload') {{
                location.reload();
            }}
        }};
        eventSource.onerror = () => {{
            console.log('SSE connection error, will retry...');
        }};
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
