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
    application::{execute_split, OutputReservation, SourceInspector, ThumbnailRenderer},
    domain::{OutputSpec, PageSelection, SourceIncident, SplitPlan, SplitValidationError},
    infrastructure::{LocalOutputReservation, LocalSourceInspector, PdfiumService},
};

const SPLIT_EVENT: &str = "split-event";

#[derive(Clone, Default)]
pub struct SplitRuntime {
    active: Arc<Mutex<Option<Arc<AtomicBool>>>>,
    pdfium: Arc<Mutex<Option<Arc<PdfiumService>>>>,
}

impl SplitRuntime {
    fn begin(&self) -> Result<Arc<AtomicBool>, String> {
        let mut active = self
            .active
            .lock()
            .map_err(|_| "The split state is unavailable.".to_owned())?;
        if active.is_some() {
            return Err("A split is already in progress.".to_owned());
        }
        let cancelled = Arc::new(AtomicBool::new(false));
        *active = Some(cancelled.clone());
        Ok(cancelled)
    }

    fn cancel(&self) -> Result<(), String> {
        let active = self
            .active
            .lock()
            .map_err(|_| "The split state is unavailable.".to_owned())?;
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
pub struct SplitSourceDto {
    path: String,
    name: String,
    page_count: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailDto {
    page: usize,
    png_data_url: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputPreviewDto {
    output_paths: Vec<String>,
    normalized_name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SplitRequest {
    source_path: String,
    mode: String,
    pages: Vec<usize>,
    groups: Vec<Vec<usize>>,
    directory: String,
    file_name: String,
}

#[derive(Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum SplitEventDto {
    Progress {
        current: usize,
        total: usize,
        percent: u8,
    },
    Succeeded {
        output_paths: Vec<String>,
        opened: bool,
    },
    Cancelled,
    Failed {
        message: String,
    },
}

#[tauri::command]
pub fn inspect_split_source(paths: Vec<String>) -> Result<SplitSourceDto, String> {
    let source = LocalSourceInspector
        .inspect(&paths.into_iter().map(PathBuf::from).collect::<Vec<_>>())
        .map_err(source_incident_message)?;
    Ok(SplitSourceDto {
        path: source.path.display().to_string(),
        name: source.name,
        page_count: source.page_count,
    })
}

#[tauri::command]
pub fn render_split_thumbnails(
    app: AppHandle,
    runtime: State<'_, SplitRuntime>,
    source_path: String,
    pages: Vec<usize>,
) -> Result<Vec<ThumbnailDto>, String> {
    let source = LocalSourceInspector
        .inspect(&[PathBuf::from(source_path)])
        .map_err(source_incident_message)?;
    runtime
        .pdfium(&app)?
        .render(&source.path, &pages)
        .map(|thumbnails| {
            thumbnails
                .into_iter()
                .map(|thumbnail| ThumbnailDto {
                    page: thumbnail.page,
                    png_data_url: thumbnail.png_data_url,
                })
                .collect()
        })
}

#[tauri::command]
pub fn preview_split_output(
    source_path: String,
    mode: String,
    pages: Vec<usize>,
    groups: Vec<Vec<usize>>,
    directory: String,
    file_name: String,
) -> Result<OutputPreviewDto, String> {
    let source = inspect_source_path(&source_path)?;
    let plan = split_plan(&mode, pages, groups, source.page_count)?;
    let output =
        OutputSpec::new(PathBuf::from(directory), &file_name).map_err(validation_message)?;
    let outputs = plan.outputs(source.page_count);
    let paths = LocalOutputReservation.preview(&output, outputs.len())?;
    Ok(OutputPreviewDto {
        output_paths: paths
            .into_iter()
            .map(|path| path.display().to_string())
            .collect(),
        normalized_name: output.base_name,
    })
}

#[tauri::command]
pub fn start_split(
    app: AppHandle,
    runtime: State<'_, SplitRuntime>,
    request: SplitRequest,
) -> Result<(), String> {
    let SplitRequest {
        source_path,
        mode,
        pages,
        groups,
        directory,
        file_name,
    } = request;
    let source = inspect_source_path(&source_path)?;
    let plan = split_plan(&mode, pages, groups, source.page_count)?;
    let output =
        OutputSpec::new(PathBuf::from(directory), &file_name).map_err(validation_message)?;
    let selections = plan.outputs(source.page_count);
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
                SPLIT_EVENT,
                SplitEventDto::Progress {
                    current,
                    total,
                    percent,
                },
            );
        };
        let result = execute_split(
            &LocalOutputReservation,
            renderer.as_ref(),
            &source,
            &output,
            selections,
            cancelled.as_ref(),
            &mut report_progress,
        );
        let event = match result {
            Ok(output_paths) => {
                let open_path = if output_paths.len() == 1 {
                    output_paths[0].clone()
                } else {
                    output.directory.clone()
                };
                let opened = app
                    .opener()
                    .open_path(open_path.display().to_string(), None::<&str>)
                    .is_ok();
                SplitEventDto::Succeeded {
                    output_paths: output_paths
                        .iter()
                        .map(|path| path.display().to_string())
                        .collect(),
                    opened,
                }
            }
            Err(_) if cancelled.load(Ordering::Relaxed) => SplitEventDto::Cancelled,
            Err(message) => SplitEventDto::Failed { message },
        };
        let _ = app.emit(SPLIT_EVENT, event);
        runtime.finish();
    });
    Ok(())
}

#[tauri::command]
pub fn cancel_split(runtime: State<'_, SplitRuntime>) -> Result<(), String> {
    runtime.cancel()
}

fn inspect_source_path(path: &str) -> Result<super::domain::SourcePdf, String> {
    LocalSourceInspector
        .inspect(&[PathBuf::from(path)])
        .map_err(source_incident_message)
}

fn split_plan(
    mode: &str,
    pages: Vec<usize>,
    groups: Vec<Vec<usize>>,
    page_count: usize,
) -> Result<SplitPlan, String> {
    match mode {
        "eachPage" => Ok(SplitPlan::EachPage),
        "extract" => PageSelection::new(pages, page_count)
            .map(SplitPlan::Extract)
            .map_err(validation_message),
        "groups" => SplitPlan::groups(groups, page_count).map_err(validation_message),
        _ => Err("The split mode is invalid.".to_owned()),
    }
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

fn validation_message(error: SplitValidationError) -> String {
    match error {
        SplitValidationError::EmptySelection => "Select at least one page.".to_owned(),
        SplitValidationError::PageOutOfBounds => "A selected page does not exist.".to_owned(),
        SplitValidationError::EmptyGroups => "Create at least one page group.".to_owned(),
        SplitValidationError::OverlappingGroups => {
            "A page cannot belong to more than one group.".to_owned()
        }
        SplitValidationError::EmptyOutputName => "The output name is required.".to_owned(),
        SplitValidationError::DirectoryDoesNotExist => {
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
