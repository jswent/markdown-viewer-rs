use chrono::{DateTime, Utc};
use directories::ProjectDirs;
use fs2::FileExt;
use nix::sys::signal::kill;
use nix::unistd::Pid;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum StateError {
    NoProjectDirs,
    Io(std::io::Error),
    Json(serde_json::Error),
    LockFailed,
}

impl std::fmt::Display for StateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StateError::NoProjectDirs => write!(f, "Could not determine data directory"),
            StateError::Io(e) => write!(f, "IO error: {}", e),
            StateError::Json(e) => write!(f, "JSON error: {}", e),
            StateError::LockFailed => write!(f, "Failed to acquire lock on state file"),
        }
    }
}

impl std::error::Error for StateError {}

impl From<std::io::Error> for StateError {
    fn from(e: std::io::Error) -> Self {
        StateError::Io(e)
    }
}

impl From<serde_json::Error> for StateError {
    fn from(e: serde_json::Error) -> Self {
        StateError::Json(e)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub pid: i32,
    pub port: u16,
    pub file_path: PathBuf,
    pub started_at: DateTime<Utc>,
    pub log_file: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StateFile {
    pub version: u32,
    #[serde(default)]
    pub instances: HashMap<PathBuf, Instance>,
}

impl Default for StateFile {
    fn default() -> Self {
        Self {
            version: 1,
            instances: HashMap::new(),
        }
    }
}

impl StateFile {
    /// Get the data directory for mdview
    pub fn get_data_dir() -> Result<PathBuf, StateError> {
        ProjectDirs::from("", "", "mdview")
            .map(|dirs| dirs.data_dir().to_path_buf())
            .ok_or(StateError::NoProjectDirs)
    }

    /// Get the logs directory
    pub fn get_logs_dir() -> Result<PathBuf, StateError> {
        let data_dir = Self::get_data_dir()?;
        Ok(data_dir.join("logs"))
    }

    /// Get the state file path
    pub fn get_state_file_path() -> Result<PathBuf, StateError> {
        let data_dir = Self::get_data_dir()?;
        Ok(data_dir.join("instances.json"))
    }

    /// Load the state file, creating directories if needed
    pub fn load() -> Result<Self, StateError> {
        let state_path = Self::get_state_file_path()?;

        // Create directory if it doesn't exist
        if let Some(parent) = state_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Create logs directory
        let logs_dir = Self::get_logs_dir()?;
        fs::create_dir_all(&logs_dir)?;

        // If file doesn't exist, return default
        if !state_path.exists() {
            return Ok(Self::default());
        }

        // Open and lock file for reading
        let file = File::open(&state_path)?;
        FileExt::lock_shared(&file).map_err(|_| StateError::LockFailed)?;

        let mut contents = String::new();
        let mut reader = std::io::BufReader::new(&file);
        reader.read_to_string(&mut contents)?;

        FileExt::unlock(&file).map_err(|_| StateError::LockFailed)?;

        // Parse JSON, falling back to default if corrupted
        match serde_json::from_str(&contents) {
            Ok(state) => Ok(state),
            Err(_) => {
                // Backup corrupted file
                let backup_path = state_path.with_extension("json.bak");
                let _ = fs::rename(&state_path, &backup_path);
                eprintln!(
                    "Warning: State file was corrupted. Backed up to {:?}",
                    backup_path
                );
                Ok(Self::default())
            }
        }
    }

    /// Save the state file with exclusive locking
    pub fn save(&self) -> Result<(), StateError> {
        let state_path = Self::get_state_file_path()?;

        // Create directory if it doesn't exist
        if let Some(parent) = state_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Open file with exclusive lock
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&state_path)?;

        FileExt::lock_exclusive(&file).map_err(|_| StateError::LockFailed)?;

        let contents = serde_json::to_string_pretty(self)?;
        let mut writer = std::io::BufWriter::new(&file);
        writer.write_all(contents.as_bytes())?;
        writer.flush()?;

        FileExt::unlock(&file).map_err(|_| StateError::LockFailed)?;

        Ok(())
    }

    /// Add an instance to the state
    pub fn add_instance(&mut self, instance: Instance) {
        self.instances.insert(instance.file_path.clone(), instance);
    }

    /// Remove an instance by file path
    pub fn remove_instance(&mut self, file_path: &Path) -> Option<Instance> {
        self.instances.remove(file_path)
    }

    /// Get an instance by file path
    pub fn get_instance(&self, file_path: &Path) -> Option<&Instance> {
        self.instances.get(file_path)
    }

    /// Check if a process is still running
    pub fn is_process_running(pid: i32) -> bool {
        match kill(Pid::from_raw(pid), None) {
            Ok(()) => true,
            Err(nix::errno::Errno::ESRCH) => false, // No such process
            Err(nix::errno::Errno::EPERM) => true,  // Process exists but no permission
            Err(_) => false,
        }
    }

    /// Clean up stale instances (processes that are no longer running)
    /// Returns the list of removed stale instances
    pub fn cleanup_stale(&mut self) -> Vec<Instance> {
        let stale_paths: Vec<PathBuf> = self
            .instances
            .iter()
            .filter(|(_, inst)| !Self::is_process_running(inst.pid))
            .map(|(path, _)| path.clone())
            .collect();

        let mut removed = Vec::new();
        for path in stale_paths {
            if let Some(inst) = self.instances.remove(&path) {
                removed.push(inst);
            }
        }
        removed
    }

    /// Get all instances
    pub fn all_instances(&self) -> impl Iterator<Item = &Instance> {
        self.instances.values()
    }
}

/// Generate a sanitized log filename from the markdown file path
pub fn generate_log_filename(file_path: &Path, port: u16) -> String {
    let stem = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    // Sanitize: keep only alphanumeric, dash, underscore
    let sanitized: String = stem
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .take(50) // Limit length
        .collect();

    format!("{}-{}.log", sanitized, port)
}

/// Get the log file path for a given markdown file
pub fn get_log_path(file_path: &Path, port: u16) -> Result<PathBuf, StateError> {
    let logs_dir = StateFile::get_logs_dir()?;
    let filename = generate_log_filename(file_path, port);
    Ok(logs_dir.join(filename))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_log_filename() {
        let path = PathBuf::from("/some/path/README.md");
        let filename = generate_log_filename(&path, 6914);
        assert_eq!(filename, "README-6914.log");
    }

    #[test]
    fn test_generate_log_filename_special_chars() {
        let path = PathBuf::from("/some/path/my file (1).md");
        let filename = generate_log_filename(&path, 6915);
        assert_eq!(filename, "my_file__1_-6915.log");
    }

    #[test]
    fn test_state_file_default() {
        let state = StateFile::default();
        assert_eq!(state.version, 1);
        assert!(state.instances.is_empty());
    }

    #[test]
    fn test_is_process_running_self() {
        // Our own process should be running
        let pid = std::process::id() as i32;
        assert!(StateFile::is_process_running(pid));
    }

    #[test]
    fn test_is_process_not_running() {
        // PID 0 should not be a valid user process
        // Use a very high PID that's unlikely to exist
        assert!(!StateFile::is_process_running(999999999));
    }
}
