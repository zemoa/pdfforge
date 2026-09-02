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
use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
use lopdf::Document;
use pdfium_render::prelude::*;

use super::{
    application::{
        OutputReservation, PageRenderer, PdfRedactionEngine, RenderedPage, SourceInspector,
        TextWord,
    },
    domain::{display_name, NormalizedRect, OutputSpec, RedactionPlan, SourceIncident, SourcePdf},
};

const PAGE_RENDER_WIDTH: i32 = 1600;
const REDACTION_DPI: f32 = 300.0;
const PDF_POINTS_PER_INCH: f32 = 72.0;

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
            *instance = Some(
                Pdfium::bind_to_library(&self.library_path)
                    .map(Pdfium::new)
                    .map_err(|error| format!("The bundled PDF renderer is unavailable: {error}"))?,
            );
        }
        operation(instance.as_ref().expect("PDFium is initialized above"))
    }
}

impl PageRenderer for PdfiumService {
    fn render(&self, source: &Path, page_number: usize) -> Result<RenderedPage, String> {
        self.with_pdfium(|pdfium| {
            let document = pdfium
                .load_pdf_from_file(source, None)
                .map_err(|error| format!("The PDF preview could not be created: {error}"))?;
            let page = document
                .pages()
                .get(page_number.saturating_sub(1) as PdfPageIndex)
                .map_err(|_| "The requested PDF page no longer exists.".to_owned())?;
            let page_width = page.width().value;
            let page_height = page.height().value;
            if page_width <= 0.0 || page_height <= 0.0 {
                return Err("The PDF page has invalid dimensions.".to_owned());
            }

            let image = page
                .render_with_config(&PdfRenderConfig::new().set_target_width(PAGE_RENDER_WIDTH))
                .and_then(|bitmap| bitmap.as_image())
                .map_err(|error| format!("The PDF preview could not be created: {error}"))?;
            let mut png = Cursor::new(Vec::new());
            image
                .write_to(&mut png, ImageFormat::Png)
                .map_err(|error| format!("The PDF preview could not be encoded: {error}"))?;

            Ok(RenderedPage {
                page: page_number,
                aspect_ratio: page_width / page_height,
                png_data_url: format!(
                    "data:image/png;base64,{}",
                    STANDARD.encode(png.into_inner())
                ),
                words: words_for_page(&page, page_width, page_height)?,
            })
        })
    }
}

impl PdfRedactionEngine for PdfiumService {
    fn redact(
        &self,
        source: &SourcePdf,
        plan: &RedactionPlan,
        output: &Path,
        cancelled: &AtomicBool,
        report_progress: &mut dyn FnMut(usize, usize),
    ) -> Result<(), String> {
        self.with_pdfium(|pdfium| {
            let source_document = pdfium
                .load_pdf_from_file(&source.path, None)
                .map_err(|_| format!("{} can no longer be read.", source.name))?;
            let mut output_document = pdfium
                .create_new_pdf()
                .map_err(|error| format!("The masked PDF could not be created: {error}"))?;

            for page_number in 1..=source.page_count {
                ensure_not_cancelled(cancelled)?;
                let source_page = source_document
                    .pages()
                    .get(page_number.saturating_sub(1) as PdfPageIndex)
                    .map_err(|_| "The requested PDF page no longer exists.".to_owned())?;
                let width = source_page.width();
                let height = source_page.height();
                if width.value <= 0.0 || height.value <= 0.0 {
                    return Err("The PDF page has invalid dimensions.".to_owned());
                }
                let target_width = ((width.value * REDACTION_DPI / PDF_POINTS_PER_INCH).ceil()
                    as i64)
                    .clamp(1, i32::MAX as i64) as i32;
                let image = source_page
                    .render_with_config(&PdfRenderConfig::new().set_target_width(target_width))
                    .and_then(|bitmap| bitmap.as_image())
                    .map_err(|error| {
                        format!("The masked PDF page could not be rendered: {error}")
                    })?;
                let mut pixels = image.to_rgba8();
                paint_redactions(&mut pixels, plan.rectangles_for_page(page_number));
                drop(source_page);

                let mut output_page = output_document
                    .pages_mut()
                    .create_page_at_end(PdfPagePaperSize::from_points(width, height))
                    .map_err(|error| {
                        format!("The masked PDF page could not be created: {error}")
                    })?;
                output_page
                    .objects_mut()
                    .create_image_object(
                        PdfPoints::ZERO,
                        PdfPoints::ZERO,
                        &DynamicImage::ImageRgba8(pixels),
                        Some(width),
                        Some(height),
                    )
                    .map_err(|error| {
                        format!("The masked PDF page could not be written: {error}")
                    })?;
                drop(output_page);

                report_progress(page_number, source.page_count);
                ensure_not_cancelled(cancelled)?;
            }
            ensure_not_cancelled(cancelled)?;
            output_document
                .save_to_file(output)
                .map_err(|error| format!("The masked PDF could not be saved: {error}"))?;
            ensure_not_cancelled(cancelled)
        })
    }
}

pub struct LocalOutputReservation;

impl OutputReservation for LocalOutputReservation {
    fn preview(&self, output: &OutputSpec) -> Result<PathBuf, String> {
        ensure_directory_writable(&output.directory)?;
        for suffix in 0usize.. {
            let path = output_path(output, suffix);
            if !path.exists() {
                return Ok(path);
            }
        }
        unreachable!("an incrementing suffix always has a next value")
    }

    fn reserve(&self, output: &OutputSpec) -> Result<PathBuf, String> {
        ensure_directory_writable(&output.directory)?;
        for suffix in 0usize.. {
            let path = output_path(output, suffix);
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(_) => return Ok(path),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(_) => return Err("The destination folder is not writable.".to_owned()),
            }
        }
        unreachable!("an incrementing suffix always has a next value")
    }

    fn remove(&self, path: &Path) {
        let _ = fs::remove_file(path);
    }
}

fn output_path(output: &OutputSpec, suffix: usize) -> PathBuf {
    if suffix == 0 {
        return output.directory.join(&output.file_name);
    }
    let stem = output
        .file_name
        .strip_suffix(".pdf")
        .or_else(|| output.file_name.strip_suffix(".PDF"))
        .unwrap_or(&output.file_name);
    output.directory.join(format!("{stem}-{suffix}.pdf"))
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
        Err("The redaction was cancelled.".to_owned())
    } else {
        Ok(())
    }
}

fn paint_redactions(image: &mut RgbaImage, rectangles: &[NormalizedRect]) {
    let width = image.width();
    let height = image.height();
    for rectangle in rectangles {
        // Expand by one pixel in every direction so anti-aliased glyph edges cannot survive.
        let left = ((rectangle.left * width as f32).floor() as i64 - 1).max(0) as u32;
        let top = ((rectangle.top * height as f32).floor() as i64 - 1).max(0) as u32;
        let right = ((rectangle.left + rectangle.width) * width as f32)
            .ceil()
            .min(width as f32) as u32;
        let bottom = ((rectangle.top + rectangle.height) * height as f32)
            .ceil()
            .min(height as f32) as u32;
        for y in top..bottom {
            for x in left..right {
                image.put_pixel(x, y, Rgba([0, 0, 0, 255]));
            }
        }
    }
}

fn words_for_page(
    page: &PdfPage,
    page_width: f32,
    page_height: f32,
) -> Result<Vec<TextWord>, String> {
    let characters = page
        .text()
        .map_err(|error| format!("The PDF text could not be read: {error}"))?
        .chars()
        .iter()
        .filter_map(|character| {
            let value = character.unicode_char()?;
            let bounds = character
                .loose_bounds()
                .ok()
                .and_then(|bounds| normalize_bounds(bounds, page_width, page_height));
            Some(ExtractedCharacter { value, bounds })
        })
        .collect::<Vec<_>>();

    Ok(group_characters_into_words(characters))
}

#[derive(Clone, Debug, PartialEq)]
struct ExtractedCharacter {
    value: char,
    bounds: Option<NormalizedRect>,
}

fn group_characters_into_words(characters: Vec<ExtractedCharacter>) -> Vec<TextWord> {
    let mut words = Vec::new();
    let mut current = Vec::new();

    for character in characters {
        if character.value.is_whitespace() {
            push_word(&mut words, &mut current);
        } else if character.bounds.is_some() {
            current.push(character);
        }
    }
    push_word(&mut words, &mut current);
    words
}

fn push_word(words: &mut Vec<TextWord>, characters: &mut Vec<ExtractedCharacter>) {
    if characters.is_empty() {
        return;
    }
    let text = characters.iter().map(|character| character.value).collect();
    let bounds = characters
        .iter()
        .filter_map(|character| character.bounds)
        .collect::<Vec<_>>();
    words.push(TextWord {
        index: words.len(),
        text,
        bounds,
    });
    characters.clear();
}

fn normalize_bounds(bounds: PdfRect, page_width: f32, page_height: f32) -> Option<NormalizedRect> {
    let left = bounds.left().value.max(0.0).min(page_width);
    let right = bounds.right().value.max(0.0).min(page_width);
    let bottom = bounds.bottom().value.max(0.0).min(page_height);
    let top = bounds.top().value.max(0.0).min(page_height);
    NormalizedRect::new(
        left / page_width,
        (page_height - top) / page_height,
        (right - left) / page_width,
        (top - bottom) / page_height,
    )
}

fn is_pdf_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("pdf"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds() -> NormalizedRect {
        NormalizedRect::new(0.1, 0.1, 0.1, 0.1).expect("the fixture is valid")
    }

    #[test]
    fn groups_non_whitespace_characters_into_ordered_words() {
        let words = group_characters_into_words(vec![
            ExtractedCharacter {
                value: 'A',
                bounds: Some(bounds()),
            },
            ExtractedCharacter {
                value: 'l',
                bounds: Some(bounds()),
            },
            ExtractedCharacter {
                value: 'l',
                bounds: Some(bounds()),
            },
            ExtractedCharacter {
                value: 'ô',
                bounds: Some(bounds()),
            },
            ExtractedCharacter {
                value: ' ',
                bounds: None,
            },
            ExtractedCharacter {
                value: '!',
                bounds: Some(bounds()),
            },
        ]);

        assert_eq!(words.len(), 2);
        assert_eq!(words[0].index, 0);
        assert_eq!(words[0].text, "Allô");
        assert_eq!(words[1].index, 1);
        assert_eq!(words[1].text, "!");
    }

    #[test]
    fn paints_every_selected_pixel_black() {
        let mut image = RgbaImage::from_pixel(10, 10, Rgba([255, 255, 255, 255]));
        let rectangle = NormalizedRect::new(0.4, 0.4, 0.2, 0.2).expect("fixture is valid");

        paint_redactions(&mut image, &[rectangle]);

        assert_eq!(*image.get_pixel(5, 5), Rgba([0, 0, 0, 255]));
        assert_eq!(*image.get_pixel(0, 0), Rgba([255, 255, 255, 255]));
    }
}
