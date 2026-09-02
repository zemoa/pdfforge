use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use super::{
    application::{PageRenderer, SourceInspector},
    domain::{NormalizedRect, SourceIncident},
    infrastructure::{LocalSourceInspector, PdfiumService},
};

#[derive(Clone, Default)]
pub struct RedactionRuntime {
    pdfium: Arc<Mutex<Option<Arc<PdfiumService>>>>,
}

impl RedactionRuntime {
    fn pdfium(&self, app: &AppHandle) -> Result<Arc<PdfiumService>, String> {
        let mut pdfium = self
            .pdfium
            .lock()
            .map_err(|_| "The PDF renderer state is unavailable.".to_owned())?;
        if pdfium.is_none() {
            *pdfium = Some(Arc::new(PdfiumService::new(pdfium_library_path(app)?)));
        }
        Ok(Arc::clone(
            pdfium.as_ref().expect("PDFium is initialized above"),
        ))
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RedactionSourceDto {
    path: String,
    name: String,
    page_count: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedRectDto {
    left: f32,
    top: f32,
    width: f32,
    height: f32,
}

impl From<NormalizedRect> for NormalizedRectDto {
    fn from(rect: NormalizedRect) -> Self {
        Self {
            left: rect.left,
            top: rect.top,
            width: rect.width,
            height: rect.height,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextWordDto {
    index: usize,
    text: String,
    bounds: Vec<NormalizedRectDto>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RedactionPageDto {
    page: usize,
    aspect_ratio: f32,
    png_data_url: String,
    words: Vec<TextWordDto>,
}

#[tauri::command]
pub fn inspect_redaction_source(paths: Vec<String>) -> Result<RedactionSourceDto, String> {
    let source = LocalSourceInspector
        .inspect(&paths.into_iter().map(PathBuf::from).collect::<Vec<_>>())
        .map_err(source_incident_message)?;
    Ok(RedactionSourceDto {
        path: source.path.display().to_string(),
        name: source.name,
        page_count: source.page_count,
    })
}

#[tauri::command]
pub fn render_redaction_page(
    app: AppHandle,
    runtime: State<'_, RedactionRuntime>,
    source_path: String,
    page: usize,
) -> Result<RedactionPageDto, String> {
    let source = LocalSourceInspector
        .inspect(&[PathBuf::from(source_path)])
        .map_err(source_incident_message)?;
    if page == 0 || page > source.page_count {
        return Err("The requested PDF page no longer exists.".to_owned());
    }
    runtime
        .pdfium(&app)?
        .render(&source.path, page)
        .map(|page| RedactionPageDto {
            page: page.page,
            aspect_ratio: page.aspect_ratio,
            png_data_url: page.png_data_url,
            words: page
                .words
                .into_iter()
                .map(|word| TextWordDto {
                    index: word.index,
                    text: word.text,
                    bounds: word.bounds.into_iter().map(Into::into).collect(),
                })
                .collect(),
        })
}

fn source_incident_message(incident: SourceIncident) -> String {
    match incident {
        SourceIncident::NotPdf => "Choose a PDF file.".to_owned(),
        SourceIncident::PasswordProtected => "This PDF is password protected.".to_owned(),
        SourceIncident::Unreadable => "This PDF cannot be read.".to_owned(),
        SourceIncident::Inaccessible => "This PDF is inaccessible.".to_owned(),
        SourceIncident::MultipleSources => "Choose exactly one PDF file.".to_owned(),
        SourceIncident::EmptyDocument => "This PDF contains no pages.".to_owned(),
    }
}

fn pdfium_library_path(app: &AppHandle) -> Result<PathBuf, String> {
    let resource_directory = app
        .path()
        .resource_dir()
        .map_err(|error| format!("The PDF renderer path is unavailable: {error}"))?;
    let platform_directory = if cfg!(target_os = "windows") {
        "windows-x86_64"
    } else {
        "linux-x86_64"
    };
    let library_name = if cfg!(target_os = "windows") {
        "pdfium.dll"
    } else {
        "libpdfium.so"
    };
    if cfg!(target_os = "windows") {
        if let Ok(executable_directory) = app.path().executable_dir() {
            let portable = executable_directory.join(library_name);
            if portable.is_file() {
                return Ok(portable);
            }
        }
    }
    let bundled = resource_directory
        .join("pdfium")
        .join(platform_directory)
        .join(if cfg!(target_os = "windows") {
            "bin"
        } else {
            "lib"
        })
        .join(library_name);
    if bundled.is_file() {
        return Ok(bundled);
    }

    let development = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join("pdfium")
        .join(platform_directory)
        .join(if cfg!(target_os = "windows") {
            "bin"
        } else {
            "lib"
        })
        .join(library_name);
    if development.is_file() {
        return Ok(development);
    }
    Err("The bundled PDF renderer is missing.".to_owned())
}
