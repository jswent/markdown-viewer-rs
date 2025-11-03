/// Module for watching markdown files and detecting changes
use crossbeam_channel::Sender;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::error::Error;
use std::path::PathBuf;
use std::sync::mpsc::channel as std_channel;

/// Watches a single markdown file for changes and sends reload signals
///
/// This function sets up a file watcher that monitors the specified file for modifications.
/// When changes are detected, it sends a message through the provided channel to trigger
/// a reload.
///
/// # Arguments
///
/// * `path` - The path to the markdown file to watch
/// * `reload_tx` - Channel sender for sending reload signals
///
/// # Returns
///
/// A Result that is Ok(()) if watching completes successfully, or an Error if something goes wrong
///
/// # Errors
///
/// Returns an error if the file watcher cannot be created or if there are issues watching the file
pub fn watch_file(path: PathBuf, reload_tx: Sender<()>) -> Result<(), Box<dyn Error>> {
    let (tx, rx) = std_channel();

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        Config::default(),
    )?;

    // Watch the file for changes
    watcher.watch(&path, RecursiveMode::NonRecursive)?;

    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("file");

    println!("Watching for changes...");

    // Block and wait for file change events
    loop {
        match rx.recv() {
            Ok(event) => {
                // Only process modify events
                if matches!(event.kind, EventKind::Modify(_)) {
                    match reload_tx.send(()) {
                        Ok(_) => {
                            println!("Refreshed: {}", filename);
                        }
                        Err(e) => {
                            eprintln!("Error sending reload signal: {}", e);
                            // If the receiver is dropped, we should exit
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Watch error: {}", e);
                // Continue watching even if there's an error
            }
        }
    }

    Ok(())
}
