use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_opener::OpenerExt;

use super::{
    application::{apply_redaction, OutputReservation, PageRenderer, SourceInspector},
    domain::{NormalizedRect, OutputSpec, RedactionPlan, RedactionValidationError, SourceIncident},
    infrastructure::{LocalOutputReservation, LocalSourceInspector, PdfiumService},
};

const REDACTION_EVENT: &str = "redaction-event";

#[derive(Clone, Default)]
pub struct RedactionRuntime {
    active: Arc<Mutex<Option<Arc<AtomicBool>>>>,
    pdfium: Arc<Mutex<Option<Arc<PdfiumService>>>>,
}

impl RedactionRuntime {
    fn begin(&self) -> Result<Arc<AtomicBool>, String> {
        let mut active = self
            .active
            .lock()
            .map_err(|_| "The redaction state is unavailable.".to_owned())?;
        if active.is_some() {
            return Err("A redaction is already in progress.".to_owned());
        }
        let cancelled = Arc::new(AtomicBool::new(false));
        *active = Some(cancelled.clone());
        Ok(cancelled)
    }

    fn cancel(&self) -> Result<(), String> {
        let active = self
            .active
            .lock()
            .map_err(|_| "The redaction state is unavailable.".to_owned())?;
        if let Some(cancelled) = active.as_ref() {
            cancelled.store(true, Ordering::Relaxed);
        }
        Ok(())
    }

    fn finish(&self) {
        if let Ok(mut active) = self.active.lock() {
            *active = None;
        }
    }

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

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedRectDto {
    left: f32,
    top: f32,
    width: f32,
    height: f32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageRedactionsDto {
    page: usize,
    rectangles: Vec<NormalizedRectDto>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RedactionRequest {
    source_path: String,
    selections: Vec<PageRedactionsDto>,
    directory: String,
    file_name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputPreviewDto {
    output_path: String,
    normalized_name: String,
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

#[derive(Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum RedactionEventDto {
    Progress {
        current: usize,
        total: usize,
        percent: u8,
    },
    Succeeded {
        output_path: String,
        opened: bool,
    },
    Cancelled,
    Failed {
        message: String,
    },
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

#[tauri::command]
pub fn preview_redaction_output(
    source_path: String,
    selections: Vec<PageRedactionsDto>,
    directory: String,
    file_name: String,
) -> Result<OutputPreviewDto, String> {
    let source = inspect_source_path(&source_path)?;
    redaction_plan(selections, source.page_count)?;
    let output =
        OutputSpec::new(PathBuf::from(directory), &file_name).map_err(validation_message)?;
    let path = LocalOutputReservation.preview(&output)?;
    Ok(OutputPreviewDto {
        output_path: path.display().to_string(),
        normalized_name: output.file_name,
    })
}

#[tauri::command]
pub fn start_redaction(
    app: AppHandle,
    runtime: State<'_, RedactionRuntime>,
    request: RedactionRequest,
) -> Result<(), String> {
    let RedactionRequest {
        source_path,
        selections,
        directory,
        file_name,
    } = request;
    let source = inspect_source_path(&source_path)?;
    let plan = redaction_plan(selections, source.page_count)?;
    let output =
        OutputSpec::new(PathBuf::from(directory), &file_name).map_err(validation_message)?;
    let runtime = runtime.inner().clone();
    let cancelled = runtime.begin()?;
    let renderer = match runtime.pdfium(&app) {
        Ok(renderer) => renderer,
        Err(error) => {
            runtime.finish();
            return Err(error);
        }
    };

    tauri::async_runtime::spawn_blocking(move || {
        let mut report_progress = |current: usize, total: usize| {
            let percent = current
                .saturating_mul(100)
                .checked_div(total)
                .unwrap_or(0)
                .min(100) as u8;
            let _ = app.emit(
                REDACTION_EVENT,
                RedactionEventDto::Progress {
                    current,
                    total,
                    percent,
                },
            );
        };
        let result = apply_redaction(
            &LocalOutputReservation,
            renderer.as_ref(),
            &source,
            &plan,
            &output,
            cancelled.as_ref(),
            &mut report_progress,
        );
        let event = match result {
            Ok(output_path) => {
                let opened = app
                    .opener()
                    .open_path(output_path.display().to_string(), None::<&str>)
                    .is_ok();
                RedactionEventDto::Succeeded {
                    output_path: output_path.display().to_string(),
                    opened,
                }
            }
            Err(_) if cancelled.load(Ordering::Relaxed) => RedactionEventDto::Cancelled,
            Err(message) => RedactionEventDto::Failed { message },
        };
        let _ = app.emit(REDACTION_EVENT, event);
        runtime.finish();
    });
    Ok(())
}

#[tauri::command]
pub fn cancel_redaction(runtime: State<'_, RedactionRuntime>) -> Result<(), String> {
    runtime.cancel()
}

fn inspect_source_path(path: &str) -> Result<super::domain::SourcePdf, String> {
    LocalSourceInspector
        .inspect(&[PathBuf::from(path)])
        .map_err(source_incident_message)
}

fn redaction_plan(
    selections: Vec<PageRedactionsDto>,
    page_count: usize,
) -> Result<RedactionPlan, String> {
    let selections = selections
        .into_iter()
        .map(|selection| {
            let rectangles = selection
                .rectangles
                .into_iter()
                .map(|rectangle| {
                    NormalizedRect::new(
                        rectangle.left,
                        rectangle.top,
                        rectangle.width,
                        rectangle.height,
                    )
                    .ok_or_else(|| "A redaction rectangle is invalid.".to_owned())
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok((selection.page, rectangles))
        })
        .collect::<Result<Vec<_>, String>>()?;
    RedactionPlan::new(selections, page_count).map_err(validation_message)
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

fn validation_message(error: RedactionValidationError) -> String {
    match error {
        RedactionValidationError::EmptySelection => {
            "Select at least one word or zone to redact.".to_owned()
        }
        RedactionValidationError::PageOutOfBounds => "A selected page no longer exists.".to_owned(),
        RedactionValidationError::EmptyOutputName => "The output name is required.".to_owned(),
        RedactionValidationError::DirectoryDoesNotExist => {
            "The destination folder does not exist.".to_owned()
        }
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
