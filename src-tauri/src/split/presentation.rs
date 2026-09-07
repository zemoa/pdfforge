use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_opener::OpenerExt;

use crate::error::{CommandError, CommandResult, ErrorCode};

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
    pub fn is_active(&self) -> bool {
        self.active.lock().is_ok_and(|active| active.is_some())
    }

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
        error: ErrorCode,
    },
}

#[tauri::command]
pub fn inspect_split_source(paths: Vec<String>) -> CommandResult<SplitSourceDto> {
    let source = LocalSourceInspector
        .inspect(&paths.into_iter().map(PathBuf::from).collect::<Vec<_>>())
        .map_err(source_incident_code)?;
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
) -> CommandResult<Vec<ThumbnailDto>> {
    let source = LocalSourceInspector
        .inspect(&[PathBuf::from(source_path)])
        .map_err(source_incident_code)?;
    runtime
        .pdfium(&app)
        .map_err(|_| CommandError::new(ErrorCode::RendererUnavailable))?
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
        .map_err(|_| CommandError::new(ErrorCode::RendererUnavailable))
}

#[tauri::command]
pub fn preview_split_output(
    source_path: String,
    mode: String,
    pages: Vec<usize>,
    groups: Vec<Vec<usize>>,
    directory: String,
    file_name: String,
) -> CommandResult<OutputPreviewDto> {
    let source = inspect_source_path(&source_path)?;
    let plan = split_plan(&mode, pages, groups, source.page_count)?;
    let output = OutputSpec::new(PathBuf::from(directory), &file_name).map_err(validation_code)?;
    let outputs = plan.outputs(source.page_count);
    let paths = LocalOutputReservation
        .preview(&output, outputs.len())
        .map_err(|error| CommandError::new(output_error_code(&error)))?;
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
) -> CommandResult<()> {
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
    let output = OutputSpec::new(PathBuf::from(directory), &file_name).map_err(validation_code)?;
    let selections = plan.outputs(source.page_count);
    let runtime = runtime.inner().clone();
    let cancelled = runtime
        .begin()
        .map_err(|_| CommandError::new(ErrorCode::OperationInProgress))?;
    let renderer = match runtime.pdfium(&app) {
        Ok(renderer) => renderer,
        Err(_) => {
            runtime.finish();
            return Err(CommandError::new(ErrorCode::RendererUnavailable));
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
            Err(_) => SplitEventDto::Failed {
                error: ErrorCode::SplitFailed,
            },
        };
        let _ = app.emit(SPLIT_EVENT, event);
        runtime.finish();
    });
    Ok(())
}

#[tauri::command]
pub fn cancel_split(runtime: State<'_, SplitRuntime>) -> CommandResult<()> {
    runtime
        .cancel()
        .map_err(|_| CommandError::new(ErrorCode::Unexpected))
}

fn inspect_source_path(path: &str) -> CommandResult<super::domain::SourcePdf> {
    LocalSourceInspector
        .inspect(&[PathBuf::from(path)])
        .map_err(source_incident_code)
}

fn split_plan(
    mode: &str,
    pages: Vec<usize>,
    groups: Vec<Vec<usize>>,
    page_count: usize,
) -> CommandResult<SplitPlan> {
    match mode {
        "eachPage" => Ok(SplitPlan::EachPage),
        "extract" => PageSelection::new(pages, page_count)
            .map(SplitPlan::Extract)
            .map_err(validation_code),
        "groups" => SplitPlan::groups(groups, page_count).map_err(validation_code),
        _ => Err(CommandError::new(ErrorCode::SplitFailed)),
    }
}

fn source_incident_code(incident: SourceIncident) -> CommandError {
    match incident {
        SourceIncident::NotPdf => CommandError::new(ErrorCode::SourceNotPdf),
        SourceIncident::PasswordProtected => CommandError::new(ErrorCode::SourcePasswordProtected),
        SourceIncident::Unreadable => CommandError::new(ErrorCode::SourceUnreadable),
        SourceIncident::Inaccessible => CommandError::new(ErrorCode::SourceInaccessible),
        SourceIncident::MultipleSources => CommandError::new(ErrorCode::SourceMultiple),
        SourceIncident::EmptyDocument => CommandError::new(ErrorCode::SourceEmpty),
    }
}

fn validation_code(error: SplitValidationError) -> CommandError {
    match error {
        SplitValidationError::EmptySelection => CommandError::new(ErrorCode::SelectionRequired),
        SplitValidationError::PageOutOfBounds => CommandError::new(ErrorCode::PageUnavailable),
        SplitValidationError::EmptyGroups => CommandError::new(ErrorCode::GroupsRequired),
        SplitValidationError::OverlappingGroups => CommandError::new(ErrorCode::GroupsOverlap),
        SplitValidationError::EmptyOutputName => CommandError::new(ErrorCode::OutputNameRequired),
        SplitValidationError::DirectoryDoesNotExist => {
            CommandError::new(ErrorCode::DestinationMissing)
        }
    }
}

fn output_error_code(error: &str) -> ErrorCode {
    match error {
        "The destination folder does not exist." => ErrorCode::DestinationMissing,
        "The destination folder is not writable." => ErrorCode::DestinationNotWritable,
        _ => ErrorCode::SplitFailed,
    }
}

fn pdfium_library_path(app: &AppHandle) -> Result<PathBuf, String> {
    crate::pdfium::library_path(app)
}
