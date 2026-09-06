use std::path::{Path, PathBuf};

#[cfg(target_os = "windows")]
use std::fs;

use pdfium_render::prelude::*;
use tauri::{AppHandle, Manager};

/// Creates a Pdfium handle, or a new handle to the bindings already initialized
/// elsewhere in this process.
///
/// `pdfium-render` stores its bindings globally, so the split and redaction
/// adapters must not both try to initialize them.
pub fn load_or_reuse(path: &Path) -> Result<Pdfium, String> {
    match Pdfium::bind_to_library(path) {
        Ok(bindings) => Ok(Pdfium::new(bindings)),
        Err(PdfiumError::PdfiumLibraryBindingsAlreadyInitialized) => Ok(Pdfium::default()),
        Err(error) => Err(format!("The bundled PDF renderer is unavailable: {error}")),
    }
}

pub fn library_path(app: &AppHandle) -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    {
        ensure_windows_library(app)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let resource_directory = app
            .path()
            .resource_dir()
            .map_err(|error| format!("The PDF renderer path is unavailable: {error}"))?;
        let bundled = resource_directory
            .join("pdfium")
            .join("linux-x86_64")
            .join("lib")
            .join("libpdfium.so");
        if bundled.is_file() {
            return Ok(bundled);
        }
        let development = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("pdfium")
            .join("linux-x86_64")
            .join("lib")
            .join("libpdfium.so");
        if development.is_file() {
            return Ok(development);
        }
        Err("The bundled PDF renderer is missing.".to_owned())
    }
}

#[cfg(target_os = "windows")]
fn ensure_windows_library(app: &AppHandle) -> Result<PathBuf, String> {
    const PDFIUM: &[u8] = include_bytes!("../resources/pdfium/windows-x86_64/bin/pdfium.dll");
    let directory = app
        .path()
        .app_local_data_dir()
        .map_err(|_| "The local PDF renderer folder is unavailable.".to_owned())?
        .join("pdfium")
        .join(env!("CARGO_PKG_VERSION"));
    fs::create_dir_all(&directory)
        .map_err(|_| "The local PDF renderer folder is unavailable.".to_owned())?;
    let library = directory.join("pdfium.dll");
    if fs::read(&library).ok().as_deref() != Some(PDFIUM) {
        let temporary = directory.join("pdfium.dll.new");
        fs::write(&temporary, PDFIUM)
            .map_err(|_| "The PDF renderer could not be restored.".to_owned())?;
        fs::rename(&temporary, &library)
            .map_err(|_| "The PDF renderer could not be restored.".to_owned())?;
    }
    Ok(library)
}
