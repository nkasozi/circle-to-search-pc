use std::path::Path;

use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileLaunchAction {
    OpenWithDefaultApplication,
    RevealInFileManager,
}

pub trait FileLauncher: Send + Sync {
    fn launch(&self, path: &Path, action: FileLaunchAction) -> Result<FileLaunchAction>;
}
