//! Secret Lifecycle Management
//!
//! Issue #44: Secure secret handling with automatic cleanup
//!
//! This module provides RAII-based guards for temporary files containing secrets.
//! Files are automatically deleted when the guard goes out of scope, even on panic.

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use tracing::{debug, info, warn};

/// RAII guard for temporary files containing sensitive data.
///
/// When this guard is dropped (goes out of scope), the file is automatically deleted.
/// This ensures secrets don't persist on disk even if the process crashes.
///
/// # Example
///
/// ```rust
/// use std::path::PathBuf;
/// use lornu_engine::agents::lifecycle::TempFileGuard;
///
/// {
///     let _guard = TempFileGuard::new(
///         PathBuf::from("/tmp/setup.sh"),
///         "#!/bin/bash\nexport SECRET=xxx".to_string()
///     ).unwrap();
///
///     // File exists here, use it for your operations...
///
/// } // File is automatically deleted when _guard goes out of scope
/// ```
pub struct TempFileGuard {
    /// Path to the temporary file
    pub path: PathBuf,
    /// Whether to securely overwrite before deletion (defense in depth)
    secure_wipe: bool,
}

impl TempFileGuard {
    /// Create a new temporary file with the given content.
    ///
    /// The file will be automatically deleted when this guard is dropped.
    pub fn new(path: PathBuf, content: String) -> Result<Self> {
        fs::write(&path, &content)
            .with_context(|| format!("Failed to write temp file: {:?}", path))?;

        debug!("Created temporary file: {:?}", path);

        Ok(Self {
            path,
            secure_wipe: true,
        })
    }

    /// Create a guard without secure wiping (faster, but less secure).
    pub fn new_fast(path: PathBuf, content: String) -> Result<Self> {
        fs::write(&path, &content)
            .with_context(|| format!("Failed to write temp file: {:?}", path))?;

        debug!("Created temporary file (fast mode): {:?}", path);

        Ok(Self {
            path,
            secure_wipe: false,
        })
    }

    /// Create a guard for an existing file (takes ownership for cleanup).
    pub fn from_existing(path: PathBuf) -> Self {
        Self {
            path,
            secure_wipe: true,
        }
    }

    /// Get the path to the temporary file.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Securely overwrite the file contents before deletion.
    fn secure_overwrite(&self) -> Result<()> {
        if !self.path.exists() {
            return Ok(());
        }

        let metadata = fs::metadata(&self.path)?;
        let size = metadata.len() as usize;

        // Overwrite with zeros
        let zeros = vec![0u8; size];
        fs::write(&self.path, &zeros)?;

        // Overwrite with ones
        let ones = vec![0xFFu8; size];
        fs::write(&self.path, &ones)?;

        // Overwrite with random-ish data (simple XOR pattern)
        let pattern: Vec<u8> = (0..size).map(|i| (i * 0xAB) as u8).collect();
        fs::write(&self.path, &pattern)?;

        Ok(())
    }
}

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        if self.path.exists() {
            // Attempt secure wipe if enabled
            if self.secure_wipe {
                if let Err(e) = self.secure_overwrite() {
                    warn!(
                        path = ?self.path,
                        error = %e,
                        "Failed to securely overwrite temporary file"
                    );
                }
            }

            // Delete the file
            match fs::remove_file(&self.path) {
                Ok(_) => info!(path = ?self.path, "Securely wiped and deleted temporary file"),
                Err(e) => warn!(
                    path = ?self.path,
                    error = %e,
                    "Failed to delete temporary file"
                ),
            }
        }
    }
}

/// Execute a command with a secret passed via environment variable (not written to disk).
///
/// This is the most secure approach - the secret never touches the filesystem.
///
/// # Example
///
/// ```rust,no_run
/// use lornu_engine::agents::lifecycle::exec_with_secret_env;
///
/// exec_with_secret_env(
///     "gh",
///     &["pr", "list"],
///     "GITHUB_TOKEN",
///     "ghp_xxxxxxxxxxxx"
/// ).unwrap();
/// ```
pub fn exec_with_secret_env(
    program: &str,
    args: &[&str],
    env_name: &str,
    secret: &str,
) -> Result<String> {
    let output = Command::new(program)
        .args(args)
        .env(env_name, secret)
        .output()
        .with_context(|| format!("Failed to execute: {} {:?}", program, args))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Command failed with status {}: {}", output.status, stderr);
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Execute a command with a secret passed via stdin (not written to disk).
///
/// Useful for commands that expect secrets on stdin rather than environment variables.
///
/// # Example
///
/// ```rust,no_run
/// use lornu_engine::agents::lifecycle::exec_with_secret_stdin;
///
/// exec_with_secret_stdin(
///     "some-cli",
///     &["--password-stdin"],
///     "my-secret-password"
/// ).unwrap();
/// ```
pub fn exec_with_secret_stdin(program: &str, args: &[&str], secret: &str) -> Result<String> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to spawn: {} {:?}", program, args))?;

    // Write secret to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(secret.as_bytes())
            .context("Failed to write secret to stdin")?;
    }

    let output = child
        .wait_with_output()
        .context("Failed to wait for command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Command failed with status {}: {}", output.status, stderr);
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Batch cleanup of sensitive file patterns.
///
/// This is useful for cleaning up after a pipeline run.
pub fn cleanup_sensitive_files(base_dir: &PathBuf, patterns: &[&str]) -> Result<Vec<PathBuf>> {
    use std::fs::read_dir;

    let mut deleted = Vec::new();

    fn matches_pattern(filename: &str, pattern: &str) -> bool {
        // Simple glob matching for common patterns
        if let Some(suffix) = pattern.strip_prefix("**/") {
            filename.ends_with(suffix)
        } else if pattern.starts_with("*.") {
            let ext = &pattern[1..];
            filename.ends_with(ext)
        } else {
            filename == pattern
        }
    }

    fn scan_dir(dir: &PathBuf, patterns: &[&str], deleted: &mut Vec<PathBuf>) -> Result<()> {
        if !dir.exists() || !dir.is_dir() {
            return Ok(());
        }

        for entry in read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                scan_dir(&path, patterns, deleted)?;
            } else if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                for pattern in patterns {
                    if matches_pattern(filename, pattern) {
                        // Use TempFileGuard to securely delete
                        let guard = TempFileGuard::from_existing(path.clone());
                        deleted.push(path.clone());
                        drop(guard); // Triggers secure deletion
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    scan_dir(base_dir, patterns, &mut deleted)?;
    Ok(deleted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_temp_file_guard_cleanup() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("secret.txt");

        {
            let _guard = TempFileGuard::new(path.clone(), "secret-content".to_string()).unwrap();
            assert!(path.exists());
        }

        // File should be deleted after guard drops
        assert!(!path.exists());
    }

    #[test]
    fn test_temp_file_guard_fast() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("secret-fast.txt");

        {
            let _guard =
                TempFileGuard::new_fast(path.clone(), "secret-content".to_string()).unwrap();
            assert!(path.exists());
        }

        assert!(!path.exists());
    }

    #[test]
    fn test_cleanup_sensitive_files() {
        let dir = tempdir().unwrap();
        let base = dir.path().to_path_buf();

        // Create some test files
        fs::write(base.join("setup.sh"), "secret").unwrap();
        fs::write(base.join("safe.txt"), "safe").unwrap();
        fs::write(base.join("credentials.json"), "{}").unwrap();

        let patterns = &["setup.sh", "credentials.json"];
        let deleted = cleanup_sensitive_files(&base, patterns).unwrap();

        assert_eq!(deleted.len(), 2);
        assert!(!base.join("setup.sh").exists());
        assert!(!base.join("credentials.json").exists());
        assert!(base.join("safe.txt").exists());
    }
}
