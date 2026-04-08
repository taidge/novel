//! Filesystem removal helpers with short retry loops.
//!
//! Motivation: on Windows, anti-virus, the Search Indexer, or an IDE file
//! watcher may briefly hold a handle on a file or directory that Novel is
//! trying to delete, causing `os error 32` ("file in use"). A short, bounded
//! retry loop makes `novel build` and `novel dev` much more reliable for end
//! users without changing semantics on success.
//!
//! These helpers also sidestep the top-level output-directory watcher
//! problem on Windows: instead of recursively deleting the output dir
//! itself (which requires exclusive access and fails under an active
//! `ReadDirectoryChangesW` watch), [`clean_dir_contents`] walks one level
//! deep and deletes each entry individually — an operation that does not
//! need the parent handle.
//!
//! Extracted from `lib.rs` in T-CODE-1.

use std::path::Path;

/// Remove every entry inside `path` without removing `path` itself.
///
/// Why not just `remove_dir_all` the whole output directory? On Windows, any
/// process holding the top-level directory handle open via
/// `ReadDirectoryChangesW` (VS Code, other IDEs, search indexers, file
/// explorers) will make `remove_dir_all` or `rename` fail with `os error 32`
/// for as long as that watcher lives, because Windows requires exclusive
/// access to delete or rename a directory. Deleting the *contents* of the
/// directory does not require ownership of the directory handle, so it works
/// even under an active watcher — which is the common case during local
/// development. Semantically this is equivalent: `write_to` re-creates every
/// expected child afterwards.
pub(crate) fn clean_dir_contents(path: &Path) -> std::io::Result<()> {
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let child = entry.path();
        let ft = entry.file_type()?;
        if ft.is_dir() {
            remove_dir_all_retry(&child)?;
        } else {
            remove_file_retry(&child)?;
        }
    }
    Ok(())
}

/// Retry wrapper around [`std::fs::remove_dir_all`] for *nested*
/// subdirectories inside the output dir. Those subdirectories are not the
/// ones held by a user-level file watcher, but they may still be touched
/// briefly by antivirus / the Search Indexer during a rebuild, which can
/// flake `remove_dir_all` with `os error 32`. A short retry loop makes the
/// operation much more reliable for end users without changing semantics on
/// success.
pub(crate) fn remove_dir_all_retry(path: &Path) -> std::io::Result<()> {
    retry_io(|| std::fs::remove_dir_all(path), path)
}

/// Retry wrapper around [`std::fs::remove_file`] — same motivation as
/// [`remove_dir_all_retry`].
pub(crate) fn remove_file_retry(path: &Path) -> std::io::Result<()> {
    retry_io(|| std::fs::remove_file(path), path)
}

fn retry_io<F: FnMut() -> std::io::Result<()>>(mut op: F, path: &Path) -> std::io::Result<()> {
    // Delays in milliseconds: 0, 25, 50, 100, 200, 400, 800 — ~1.6s total.
    const DELAYS_MS: &[u64] = &[0, 25, 50, 100, 200, 400, 800];

    let mut last_err = None;
    for (attempt, delay) in DELAYS_MS.iter().enumerate() {
        if *delay > 0 {
            std::thread::sleep(std::time::Duration::from_millis(*delay));
        }
        match op() {
            Ok(()) => return Ok(()),
            // Already gone — treat as success.
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => {
                if attempt + 1 < DELAYS_MS.len() {
                    tracing::debug!(
                        "fs op on {} attempt {} failed: {e}; retrying",
                        path.display(),
                        attempt + 1
                    );
                }
                last_err = Some(e);
            }
        }
    }
    Err(last_err.expect("at least one attempt runs"))
}
