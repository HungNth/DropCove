use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

mod drag;
mod intake;
mod item_actions;
mod managed_lifecycle;
mod pinning;
mod recovery;
mod removal;
mod sizing;
mod snapshot;
mod store;

static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(0);

pub(super) struct TestDirectory(PathBuf);

impl TestDirectory {
    pub(super) fn new(name: &str) -> Self {
        let sequence = NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir()
            .join(format!("dropcove-{name}-{}-{sequence}", std::process::id()));
        fs::create_dir_all(&path).expect("test directory should be created");
        Self(path)
    }

    pub(super) fn path(&self) -> &Path {
        &self.0
    }

    pub(super) fn source_file(&self) -> PathBuf {
        let path = self.0.join("source.txt");
        fs::write(&path, b"dropcove").expect("test source file should be written");
        path
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
