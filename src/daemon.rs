use nix::sys::stat::{umask, Mode};
use nix::unistd::{close, dup2, fork, setsid, ForkResult};
use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::path::Path;

#[derive(Debug)]
pub enum DaemonError {
    Fork(nix::Error),
    Setsid(nix::Error),
    Io(std::io::Error),
    Dup(nix::Error),
    Close(nix::Error),
}

impl std::fmt::Display for DaemonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DaemonError::Fork(e) => write!(f, "Fork failed: {}", e),
            DaemonError::Setsid(e) => write!(f, "Setsid failed: {}", e),
            DaemonError::Io(e) => write!(f, "IO error: {}", e),
            DaemonError::Dup(e) => write!(f, "Dup2 failed: {}", e),
            DaemonError::Close(e) => write!(f, "Close failed: {}", e),
        }
    }
}

impl std::error::Error for DaemonError {}

impl From<std::io::Error> for DaemonError {
    fn from(e: std::io::Error) -> Self {
        DaemonError::Io(e)
    }
}

/// Result of the daemonize operation
pub enum DaemonizeResult {
    /// We are in the parent process - should exit
    Parent,
    /// We are in the daemon child - should continue running
    Daemon,
}

/// Daemonize the current process using the double-fork pattern.
///
/// This function:
/// 1. First fork - parent exits, child continues
/// 2. Create new session (setsid) - become session leader
/// 3. Second fork - session leader exits, grandchild continues
/// 4. Set umask
/// 5. Redirect stdout/stderr to log file
/// 6. Close stdin
///
/// Returns `DaemonizeResult::Parent` if this is the parent (should exit),
/// or `DaemonizeResult::Daemon` if this is the daemon child (should continue).
pub fn daemonize(log_path: &Path) -> Result<DaemonizeResult, DaemonError> {
    // First fork
    match unsafe { fork() } {
        Ok(ForkResult::Parent { .. }) => {
            return Ok(DaemonizeResult::Parent);
        }
        Ok(ForkResult::Child) => {
            // Continue in child
        }
        Err(e) => return Err(DaemonError::Fork(e)),
    }

    // Create new session - become session leader
    setsid().map_err(DaemonError::Setsid)?;

    // Second fork - prevent acquiring a controlling terminal
    match unsafe { fork() } {
        Ok(ForkResult::Parent { .. }) => {
            // Intermediate process exits
            std::process::exit(0);
        }
        Ok(ForkResult::Child) => {
            // Continue in grandchild (the actual daemon)
        }
        Err(e) => return Err(DaemonError::Fork(e)),
    }

    // Set umask for file creation
    umask(Mode::from_bits_truncate(0o027));

    // Open log file for stdout/stderr redirection
    let log_file = File::create(log_path)?;
    let log_fd = log_file.as_raw_fd();

    // Redirect stdout to log file
    dup2(log_fd, 1).map_err(DaemonError::Dup)?;

    // Redirect stderr to log file
    dup2(log_fd, 2).map_err(DaemonError::Dup)?;

    // Close stdin
    close(0).map_err(DaemonError::Close)?;

    // Note: log_fd is intentionally kept open (stdout/stderr point to it)
    // The file will be closed when the process exits

    Ok(DaemonizeResult::Daemon)
}

/// Get the current process ID
pub fn get_pid() -> i32 {
    std::process::id() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_pid() {
        let pid = get_pid();
        assert!(pid > 0);
    }
}
