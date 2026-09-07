use std::path::PathBuf;

/// Returns the one existing directory copied from Windows Explorer, if any.
///
/// On non-Windows platforms this deliberately does not inspect the clipboard.
#[tauri::command]
pub fn resolve_pasted_destination_folder() -> Option<String> {
    copied_paths()
        .as_deref()
        .and_then(single_existing_directory)
        .map(|path| path.display().to_string())
}

fn single_existing_directory(paths: &[PathBuf]) -> Option<&PathBuf> {
    match paths {
        [path] if path.is_dir() => Some(path),
        _ => None,
    }
}

#[cfg(target_os = "windows")]
fn copied_paths() -> Option<Vec<PathBuf>> {
    use clipboard_win::{formats, Clipboard, Getter};

    let _clipboard = Clipboard::new_attempts(10).ok()?;
    let mut paths: Vec<PathBuf> = Vec::new();
    formats::FileList.read_clipboard(&mut paths).ok()?;
    Some(paths)
}

#[cfg(not(target_os = "windows"))]
fn copied_paths() -> Option<Vec<PathBuf>> {
    None
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::single_existing_directory;

    #[test]
    fn accepts_one_existing_directory() {
        let directory = std::env::temp_dir();

        assert_eq!(
            single_existing_directory(&[directory.clone()]),
            Some(&directory)
        );
    }

    #[test]
    fn rejects_an_empty_or_multiple_clipboard_selection() {
        let directory = std::env::temp_dir();

        assert_eq!(single_existing_directory(&[]), None);
        assert_eq!(
            single_existing_directory(&[directory.clone(), directory]),
            None
        );
    }

    #[test]
    fn rejects_a_file_or_missing_directory() {
        let file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        let missing = std::env::temp_dir().join("pdfforge-missing-destination-folder");

        assert_eq!(single_existing_directory(&[file]), None);
        assert_eq!(single_existing_directory(&[missing]), None);
    }
}
