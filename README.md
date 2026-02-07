# mdview

A fast, lightweight markdown viewer with live reload. Preview markdown files in your browser with GitHub styling, automatically refreshing as you edit.

## Installation

**Via shell installer (macOS/Linux):**

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/jswent/markdown-viewer-rs/releases/download/v0.0.1/markdown-viewer-installer.sh | sh
```

### From source

```bash
cargo install --path .
```

Or build manually:

```bash
cargo build --release
./target/release/mdview <file>
```

## Usage

**Quick preview** (foreground, Ctrl+C to stop):

```bash
mdview README.md
```

**Background mode** (keeps your terminal free):

```bash
mdview serve README.md
```

**Manage background instances**:

```bash
mdview list              # Show running viewers
mdview stop README.md    # Stop a specific viewer
```

The viewer opens your browser automatically. Edit your markdown file in any editor and watch the browser update on save.

## How It Works

mdview starts a local HTTP server and watches your file for changes. When you save, it sends a reload signal to the browser via Server-Sent Events. The server runs on localhost, starting at port 6914.

Background instances are tracked so you can manage them later. Running `serve` on an already-served file will show you the existing URL instead of starting a duplicate.

Logs for background instances are stored in:

- macOS: `~/Library/Application Support/mdview/logs/`
- Linux: `~/.local/share/mdview/logs/`

## FAQ

### Why does this project exist?

Writing markdown shouldn't require context switching. Most solutions either lock you into an editor or require manual refreshes. mdview runs quietly in the background, giving you live preview in your browser while you use whatever editor you prefer.

- **Editor agnostic** - Use vim, VS Code, or anything else
- **Live reload** - Changes appear instantly as you save
- **GitHub styling** - Familiar rendering with syntax highlighting
- **Background mode** - Doesn't block your terminal
- **It just works** - Only previews markdown like GitHub would. Nothing else.

### Is there a package for this?

Not right now. I only built this because I couldn't find anything out there that did what I wanted, simply. If there is sufficient interest down the road I'll publish a package, likely starting with brew.

### Can I contribute to this?

Absolutely. There is definitely work to be done as this was a quick project to solve a specific need. Feel free to open a PR if you have any enhancements. See the todo section below for more.

## TODO

- [ ] Fix syntax highlighting errors with certain languages (e.g. TypeScript)
- [ ] Fix light mode code block
- [x] Add callout support
- [x] Add heading navigation
- [x] Add copy button to code blocks
- [x] Add image preview
- [x] Add background serving

## License

MIT
