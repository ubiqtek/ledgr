//! The inbox is a directory ledgr watches for downloaded statement files.
//! Files that have been imported are moved into a `processed` subdirectory
//! inside it, so the next scan doesn't pick them up again.

use std::fs;
use std::path::{Path, PathBuf};

pub struct Inbox {
    root: PathBuf,
}

impl Inbox {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn processed_dir(&self) -> PathBuf {
        self.root.join("processed")
    }

    /// Creates the inbox and its `processed` subdirectory if they don't
    /// exist yet.
    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        fs::create_dir_all(self.processed_dir())
    }

    /// Files sitting directly in the inbox root, ready to be imported.
    /// Excludes the `processed` subdirectory, dotfiles (e.g. `.DS_Store`,
    /// which Finder litters into synced folders), and anything else that
    /// isn't a plain file.
    pub fn pending_files(&self) -> std::io::Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        for entry in fs::read_dir(&self.root)? {
            let path = entry?.path();
            let is_dotfile = path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with('.'));
            if path.is_file() && !is_dotfile {
                files.push(path);
            }
        }
        files.sort();
        Ok(files)
    }

    /// Moves an imported file into `processed/`, so it isn't picked up
    /// again on the next scan.
    pub fn mark_processed(&self, path: &Path) -> std::io::Result<PathBuf> {
        let file_name = path.file_name().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "path has no file name")
        })?;
        let dest = self.processed_dir().join(file_name);
        fs::rename(path, &dest)?;
        Ok(dest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn ensure_dirs_creates_root_and_processed() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path().join("ledgr-inbox");
        let inbox = Inbox::new(root.clone());

        inbox.ensure_dirs().expect("ensure_dirs");

        assert!(root.is_dir());
        assert!(inbox.processed_dir().is_dir());
    }

    #[test]
    fn pending_files_lists_root_files_but_not_processed_subdir() {
        let dir = tempfile::tempdir().expect("tempdir");
        let inbox = Inbox::new(dir.path().to_path_buf());
        inbox.ensure_dirs().expect("ensure_dirs");

        File::create(dir.path().join("statement.ofx")).expect("create file");
        File::create(inbox.processed_dir().join("old.ofx")).expect("create file");

        let pending = inbox.pending_files().expect("pending_files");
        assert_eq!(pending, vec![dir.path().join("statement.ofx")]);
    }

    #[test]
    fn pending_files_ignores_dotfiles() {
        let dir = tempfile::tempdir().expect("tempdir");
        let inbox = Inbox::new(dir.path().to_path_buf());
        inbox.ensure_dirs().expect("ensure_dirs");

        File::create(dir.path().join("statement.ofx")).expect("create file");
        File::create(dir.path().join(".DS_Store")).expect("create file");

        let pending = inbox.pending_files().expect("pending_files");
        assert_eq!(pending, vec![dir.path().join("statement.ofx")]);
    }

    #[test]
    fn mark_processed_moves_the_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let inbox = Inbox::new(dir.path().to_path_buf());
        inbox.ensure_dirs().expect("ensure_dirs");

        let path = dir.path().join("statement.ofx");
        File::create(&path).expect("create file");

        let dest = inbox.mark_processed(&path).expect("mark_processed");

        assert!(!path.exists());
        assert_eq!(dest, inbox.processed_dir().join("statement.ofx"));
        assert!(dest.exists());
    }
}
