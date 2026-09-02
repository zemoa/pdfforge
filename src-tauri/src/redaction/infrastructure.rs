use std::{
    fs,
    io::Cursor,
    path::{Path, PathBuf},
    sync::Mutex,
};

use base64::{engine::general_purpose::STANDARD, Engine};
use image::ImageFormat;
use lopdf::Document;
use pdfium_render::prelude::*;

use super::{
    application::{PageRenderer, RenderedPage, SourceInspector, TextWord},
    domain::{display_name, NormalizedRect, SourceIncident, SourcePdf},
};

const PAGE_RENDER_WIDTH: i32 = 1600;

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
}
