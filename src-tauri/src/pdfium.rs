use std::path::Path;

use pdfium_render::prelude::*;

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
