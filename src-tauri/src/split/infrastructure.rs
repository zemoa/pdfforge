use std::{
    fs::{self, OpenOptions},
    io::Cursor,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
};

use base64::{engine::general_purpose::STANDARD, Engine};
use image::ImageFormat;
use lopdf::Document;
use pdfium_render::prelude::*;

use crate::pdfium::load_or_reuse;

use super::{
    application::{
        OutputReservation, PdfSplitEngine, SourceInspector, Thumbnail, ThumbnailRenderer,
    },
    domain::{OutputSpec, PageSelection, SourceIncident, SourcePdf},
};

pub struct LocalSourceInspector;

impl SourceInspector for LocalSourceInspector {
    fn inspect(&self, paths: &[PathBuf]) -> Result<SourcePdf, SourceIncident> {
        if paths.len() != 1 {
            return Err(SourceIncident::MultipleSources);
        }
        let path = &paths[0];
        if !is_pdf_path(path) {
            return Err(SourceIncident::NotPdf);
        }
        if fs::metadata(path).is_err() {
            return Err(SourceIncident::Inaccessible);
        }

        match Document::load(path) {
            Ok(document) if document.is_encrypted() || document.was_encrypted() => {
                Err(SourceIncident::PasswordProtected)
            }
            Ok(document) => {
                let page_count = document.get_pages().len();
                if page_count == 0 {
                    return Err(SourceIncident::EmptyDocument);
                }
                Ok(SourcePdf {
                    path: path.clone(),
                    name: display_name(path),
                    page_count,
                })
            }
            Err(_) => Err(SourceIncident::Unreadable),
        }
    }
}

pub struct PdfiumService {
    library_path: PathBuf,
    instance: Mutex<Option<Pdfium>>,
}

impl PdfiumService {
    pub fn new(library_path: PathBuf) -> Self {
        Self {
            library_path,
            instance: Mutex::new(None),
        }
    }

    fn with_pdfium<T>(
        &self,
        operation: impl FnOnce(&Pdfium) -> Result<T, String>,
    ) -> Result<T, String> {
        let mut instance = self
            .instance
            .lock()
            .map_err(|_| "The bundled PDF renderer is unavailable.".to_owned())?;
        if instance.is_none() {
            *instance = Some(load_or_reuse(&self.library_path)?);
        }
        operation(instance.as_ref().expect("PDFium is initialized above"))
    }
}

impl ThumbnailRenderer for PdfiumService {
    fn render(&self, source: &Path, pages: &[usize]) -> Result<Vec<Thumbnail>, String> {
        self.with_pdfium(|pdfium| {
            let document = pdfium
                .load_pdf_from_file(source, None)
                .map_err(|error| format!("The PDF preview could not be created: {error}"))?;
            let config = PdfRenderConfig::new()
                .set_target_width(180)
                .set_maximum_height(240);

            pages
                .iter()
                .map(|page_number| {
                    let page = document
                        .pages()
                        .get(page_number.saturating_sub(1) as PdfPageIndex)
                        .map_err(|_| "The requested PDF page no longer exists.".to_owned())?;
                    let image = page
                        .render_with_config(&config)
                        .and_then(|bitmap| bitmap.as_image())
                        .map_err(|error| {
                            format!("The PDF preview could not be created: {error}")
                        })?;
                    let mut png = Cursor::new(Vec::new());
                    image
                        .write_to(&mut png, ImageFormat::Png)
                        .map_err(|error| {
                            format!("The PDF preview could not be encoded: {error}")
                        })?;
                    Ok(Thumbnail {
                        page: *page_number,
                        png_data_url: format!(
                            "data:image/png;base64,{}",
                            STANDARD.encode(png.into_inner())
                        ),
                    })
                })
                .collect()
        })
    }
}

impl PdfSplitEngine for PdfiumService {
    fn split(
        &self,
        source: &SourcePdf,
        outputs: &[(PathBuf, PageSelection)],
        cancelled: &AtomicBool,
        report_progress: &mut dyn FnMut(usize, usize),
    ) -> Result<(), String> {
        self.with_pdfium(|pdfium| {
            let source_document = pdfium
                .load_pdf_from_file(&source.path, None)
                .map_err(|_| format!("{} can no longer be read.", source.name))?;
            let total = outputs.iter().map(|(_, pages)| pages.pages().len()).sum();
            let mut completed = 0;

            for (output_path, selection) in outputs {
                ensure_not_cancelled(cancelled)?;
                let mut output_document = pdfium
                    .create_new_pdf()
                    .map_err(|error| format!("The output PDF could not be created: {error}"))?;
                for (index, page_number) in selection.pages().iter().enumerate() {
                    ensure_not_cancelled(cancelled)?;
                    output_document
                        .pages_mut()
                        .copy_page_from_document(
                            &source_document,
                            page_number.saturating_sub(1) as PdfPageIndex,
                            index as PdfPageIndex,
                        )
                        .map_err(|error| {
                            format!("Page {page_number} could not be copied: {error}")
                        })?;
                    completed += 1;
                    report_progress(completed, total);
                }
                ensure_not_cancelled(cancelled)?;
                output_document
                    .save_to_file(output_path)
                    .map_err(|error| format!("The output PDF could not be saved: {error}"))?;
            }
            Ok(())
        })
    }
}

pub struct LocalOutputReservation;

impl OutputReservation for LocalOutputReservation {
    fn preview(&self, output: &OutputSpec, output_count: usize) -> Result<Vec<PathBuf>, String> {
        ensure_directory_writable(&output.directory)?;
        for batch_index in 0usize.. {
            let paths = output_paths(output, output_count, batch_index);
            if paths.iter().all(|path| !path.exists()) {
                return Ok(paths);
            }
        }
        unreachable!("an incrementing batch index always has a next value")
    }

    fn reserve(&self, output: &OutputSpec, output_count: usize) -> Result<Vec<PathBuf>, String> {
        ensure_directory_writable(&output.directory)?;
        for batch_index in 0usize.. {
            let paths = output_paths(output, output_count, batch_index);
            let mut reserved = Vec::new();
            let mut collision = false;
            for path in &paths {
                match OpenOptions::new().write(true).create_new(true).open(path) {
                    Ok(_) => reserved.push(path.clone()),
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                        collision = true;
                        break;
                    }
                    Err(_) => {
                        self.remove_all(&reserved);
                        return Err("The destination folder is not writable.".to_owned());
                    }
                }
            }
            if collision {
                self.remove_all(&reserved);
                continue;
            }
            return Ok(reserved);
        }
        unreachable!("an incrementing batch index always has a next value")
    }

    fn remove_all(&self, paths: &[PathBuf]) {
        for path in paths {
            let _ = fs::remove_file(path);
        }
    }
}

fn output_paths(output: &OutputSpec, output_count: usize, batch_index: usize) -> Vec<PathBuf> {
    output
        .output_names(output_count, batch_index)
        .into_iter()
        .map(|name| output.directory.join(name))
        .collect()
}

fn ensure_directory_writable(directory: &Path) -> Result<(), String> {
    if !directory.is_dir() {
        return Err("The destination folder does not exist.".to_owned());
    }
    let probe = directory.join(format!(".pdfforge-write-check-{}", std::process::id()));
    match OpenOptions::new().write(true).create_new(true).open(&probe) {
        Ok(_) => {
            let _ = fs::remove_file(probe);
            Ok(())
        }
        Err(_) => Err("The destination folder is not writable.".to_owned()),
    }
}

fn ensure_not_cancelled(cancelled: &AtomicBool) -> Result<(), String> {
    if cancelled.load(Ordering::Relaxed) {
        Err("The split was cancelled.".to_owned())
    } else {
        Ok(())
    }
}

fn display_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| path.display().to_string())
}

fn is_pdf_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("pdf"))
}
