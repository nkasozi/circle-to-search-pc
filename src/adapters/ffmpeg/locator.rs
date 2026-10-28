use std::path::{Path, PathBuf};

const FFMPEG_EXECUTABLE_NAME: &str = "ffmpeg";
const BUNDLED_LIBRARY_RELATIVE_DIRECTORY: &str = "../lib/circle-to-search-pc";

pub const BUNDLED_ENCODER_MISSING_MESSAGE: &str =
    "The built-in recording component is missing. Reinstall Circle to Search to restore it.";

pub fn build_candidate_directories(executable_directory: Option<&Path>) -> Vec<PathBuf> {
    executable_directory
        .map(|directory| {
            vec![
                directory.to_path_buf(),
                directory.join(BUNDLED_LIBRARY_RELATIVE_DIRECTORY),
            ]
        })
        .unwrap_or_default()
}

pub fn resolve_executable(
    candidate_directories: &[PathBuf],
    is_executable_file: impl Fn(&Path) -> bool,
) -> Option<PathBuf> {
    candidate_directories
        .iter()
        .map(|directory| directory.join(FFMPEG_EXECUTABLE_NAME))
        .find(|candidate| is_executable_file(candidate))
}

pub fn locate_bundled_ffmpeg() -> Option<PathBuf> {
    let executable_directory = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf));
    let candidates = build_candidate_directories(executable_directory.as_deref());
    resolve_executable(&candidates, |candidate| candidate.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_only_bundled_locations_are_searched() {
        let candidates = build_candidate_directories(Some(Path::new("/opt/app/usr/bin")));
        assert_eq!(
            candidates,
            vec![
                PathBuf::from("/opt/app/usr/bin"),
                PathBuf::from("/opt/app/usr/bin").join(BUNDLED_LIBRARY_RELATIVE_DIRECTORY)
            ]
        );
    }

    #[test]
    fn test_resolve_executable_returns_first_existing_candidate() {
        let candidates = vec![PathBuf::from("/a"), PathBuf::from("/b")];
        let resolved = resolve_executable(&candidates, |candidate| candidate.starts_with("/b"));
        assert_eq!(
            resolved,
            Some(PathBuf::from("/b").join(FFMPEG_EXECUTABLE_NAME))
        );
    }

    #[test]
    fn test_no_executable_directory_means_no_candidates() {
        assert!(build_candidate_directories(None).is_empty());
        assert_eq!(resolve_executable(&[], |_| true), None);
    }
}
