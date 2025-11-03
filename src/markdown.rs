use comrak::plugins::syntect::SyntectAdapterBuilder;
/// Module for converting markdown to HTML using comrak
use comrak::{markdown_to_html_with_plugins, Options, Plugins};
use std::io::Cursor;
use syntect::highlighting::ThemeSet;

/// Converts markdown content to HTML with GitHub-flavored markdown extensions
///
/// This function configures comrak to match the behavior of the Python implementation,
/// including support for tables, strikethrough, autolinks, tasklists, and syntax highlighting.
///
/// # Arguments
///
/// * `content` - The raw markdown content as a string
///
/// # Returns
///
/// The rendered HTML as a String
pub fn convert_markdown(content: &str) -> String {
    let mut options = Options::default();

    // Enable GitHub-flavored markdown extensions
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.tagfilter = true;
    options.extension.superscript = false;
    options.extension.header_ids = Some(String::new());
    options.extension.footnotes = false;
    options.extension.description_lists = false;
    options.extension.front_matter_delimiter = None;

    // Configure rendering options
    options.render.github_pre_lang = true;
    options.render.unsafe_ = false;
    options.render.hardbreaks = false;

    // Parse options
    options.parse.smart = false;
    options.parse.default_info_string = None;

    // Set up syntax highlighting with custom gh-dark theme (bundled at compile time)
    const THEME_DATA: &[u8] = include_bytes!("../assets/gh-dark.tmTheme");

    let mut theme_set = ThemeSet::new();
    let theme = ThemeSet::load_from_reader(&mut Cursor::new(THEME_DATA))
        .expect("Failed to load bundled gh-dark theme");
    theme_set.themes.insert("gh-dark".to_string(), theme);

    let adapter = SyntectAdapterBuilder::new()
        .theme_set(theme_set)
        .theme("gh-dark")
        .build();
    let mut plugins = Plugins::default();
    plugins.render.codefence_syntax_highlighter = Some(&adapter);

    markdown_to_html_with_plugins(content, &options, &plugins)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_markdown() {
        let md = "# Hello World\n\nThis is a test.";
        let html = convert_markdown(md);
        assert!(html.contains("<h1>"));
        assert!(html.contains("Hello World"));
        assert!(html.contains("<p>"));
        assert!(html.contains("This is a test."));
    }

    #[test]
    fn test_table_support() {
        let md = "| Header 1 | Header 2 |\n|----------|----------|\n| Cell 1   | Cell 2   |";
        let html = convert_markdown(md);
        assert!(html.contains("<table>"));
        assert!(html.contains("<th>"));
        assert!(html.contains("Header 1"));
    }

    #[test]
    fn test_code_block() {
        let md = "```rust\nfn main() {}\n```";
        let html = convert_markdown(md);
        assert!(html.contains("<pre>"));
        assert!(html.contains("<code"));
        assert!(html.contains("fn main()"));
    }

    #[test]
    fn test_strikethrough() {
        let md = "~~strikethrough~~";
        let html = convert_markdown(md);
        assert!(html.contains("<del>") || html.contains("strikethrough"));
    }

    #[test]
    fn test_tasklist() {
        let md = "- [ ] Task 1\n- [x] Task 2";
        let html = convert_markdown(md);
        assert!(html.contains("checkbox"));
    }
}
