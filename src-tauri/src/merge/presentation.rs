use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_opener::OpenerExt;

use super::{
    application::{collect_warnings, OutputReservation, PdfMergeEngine, SourceInspector},
    domain::{InteractiveWarning, OutputSpec, SourceIncident, SourcePdf},
    infrastructure::{LocalOutputReservation, LocalSourceInspector, LopdfMergeEngine},
};

const MERGE_EVENT: &str = "merge-event";

#[derive(Clone, Default)]
pub struct MergeRuntime {
    active: Arc<Mutex<Option<Arc<AtomicBool>>>>,
}

impl MergeRuntime {
    fn begin(&self) -> Result<Arc<AtomicBool>, String> {
        let mut active = self
            .active
            .lock()
            .map_err(|_| "The merge state is unavailable.".to_owned())?;
        if active.is_some() {
            return Err("A merge is already in progress.".to_owned());
        }
        let cancelled = Arc::new(AtomicBool::new(false));
        *active = Some(cancelled.clone());
        Ok(cancelled)
    }

    fn cancel(&self) -> Result<(), String> {
        let active = self
            .active
            .lock()
            .map_err(|_| "The merge state is unavailable.".to_owned())?;
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
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourcePdfDto {
    path: String,
    name: String,
    page_count: usize,
    warnings: Vec<InteractiveWarningDto>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectionDto {
    accepted: Vec<SourcePdfDto>,
    ignored_non_pdfs: Vec<String>,
    incidents: Vec<IncidentDto>,
    warnings: Vec<InteractiveWarningDto>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncidentDto {
    kind: &'static str,
    path: String,
    name: String,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum InteractiveWarningDto {
    Bookmarks,
    Forms,
    TaggedStructure,
    NamedDestinations,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputPreviewDto {
    output_path: String,
    normalized_name: String,
}

#[derive(Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum MergeEventDto {
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
pub fn inspect_merge_sources(paths: Vec<String>) -> InspectionDto {
    let inspection =
        LocalSourceInspector.inspect(&paths.into_iter().map(PathBuf::from).collect::<Vec<_>>());

    InspectionDto {
        accepted: inspection.accepted.iter().map(source_dto).collect(),
        ignored_non_pdfs: inspection
            .ignored_non_pdfs
            .iter()
            .map(|path| path.display().to_string())
            .collect(),
        incidents: inspection.incidents.iter().map(incident_dto).collect(),
        warnings: collect_warnings(&inspection.accepted)
            .into_iter()
            .map(warning_dto)
            .collect(),
    }
}

#[tauri::command]
pub fn preview_merge_output(
    directory: String,
    file_name: String,
) -> Result<OutputPreviewDto, String> {
    let output =
        OutputSpec::new(PathBuf::from(directory), &file_name).map_err(output_validation_message)?;
    let path = LocalOutputReservation.preview(&output)?;
    Ok(OutputPreviewDto {
        output_path: path.display().to_string(),
        normalized_name: output.file_name,
    })
}

#[tauri::command]
pub fn start_merge(
    app: AppHandle,
    runtime: State<'_, MergeRuntime>,
    source_paths: Vec<String>,
    directory: String,
    file_name: String,
) -> Result<(), String> {
    let inspection =
        LocalSourceInspector.inspect(&source_paths.iter().map(PathBuf::from).collect::<Vec<_>>());
    if inspection.accepted.len() != source_paths.len() || !inspection.incidents.is_empty() {
        return Err(
            "A source PDF is no longer available. Please review the preparation.".to_owned(),
        );
    }
    if inspection.accepted.len() < 2 {
        return Err("At least two valid PDF files are required.".to_owned());
    }
    let output =
        OutputSpec::new(PathBuf::from(directory), &file_name).map_err(output_validation_message)?;
    let runtime = runtime.inner().clone();
    let cancelled = runtime.begin()?;
    let output_path = match LocalOutputReservation.reserve(&output) {
        Ok(path) => path,
        Err(error) => {
            runtime.finish();
            return Err(error);
        }
    };
    tauri::async_runtime::spawn_blocking(move || {
        let engine = LopdfMergeEngine;
        let mut report_progress = |current: usize, total: usize| {
            let percent = current
                .saturating_mul(100)
                .checked_div(total)
                .unwrap_or(0)
                .min(100) as u8;
            let _ = app.emit(
                MERGE_EVENT,
                MergeEventDto::Progress {
                    current,
                    total,
                    percent,
                },
            );
        };
        let result = engine.merge(
            &inspection.accepted,
            &output_path,
            cancelled.as_ref(),
            &mut report_progress,
        );
        let event = match result {
            Ok(()) => {
                let opened = app
                    .opener()
                    .open_path(output_path.display().to_string(), None::<&str>)
                    .is_ok();
                MergeEventDto::Succeeded {
                    output_path: output_path.display().to_string(),
                    opened,
                }
            }
            Err(_) if cancelled.load(Ordering::Relaxed) => {
                LocalOutputReservation.remove(&output_path);
                MergeEventDto::Cancelled
            }
            Err(message) => {
                LocalOutputReservation.remove(&output_path);
                MergeEventDto::Failed { message }
            }
        };
        let _ = app.emit(MERGE_EVENT, event);
        runtime.finish();
    });
    Ok(())
}

#[tauri::command]
pub fn cancel_merge(runtime: State<'_, MergeRuntime>) -> Result<(), String> {
    runtime.cancel()
}

fn source_dto(source: &SourcePdf) -> SourcePdfDto {
    SourcePdfDto {
        path: source.path.display().to_string(),
        name: source.name.clone(),
        page_count: source.page_count,
        warnings: source.warnings.iter().copied().map(warning_dto).collect(),
    }
}

fn incident_dto(incident: &SourceIncident) -> IncidentDto {
    match incident {
        SourceIncident::PasswordProtected { path, name } => IncidentDto {
            kind: "passwordProtected",
            path: path.display().to_string(),
            name: name.clone(),
        },
        SourceIncident::Unreadable { path, name } => IncidentDto {
            kind: "unreadable",
            path: path.display().to_string(),
            name: name.clone(),
        },
        SourceIncident::Inaccessible { path, name } => IncidentDto {
            kind: "inaccessible",
            path: path.display().to_string(),
            name: name.clone(),
        },
    }
}

fn warning_dto(warning: InteractiveWarning) -> InteractiveWarningDto {
    match warning {
        InteractiveWarning::Bookmarks => InteractiveWarningDto::Bookmarks,
        InteractiveWarning::Forms => InteractiveWarningDto::Forms,
        InteractiveWarning::TaggedStructure => InteractiveWarningDto::TaggedStructure,
        InteractiveWarning::NamedDestinations => InteractiveWarningDto::NamedDestinations,
    }
}

fn output_validation_message(error: super::domain::OutputValidationError) -> String {
    match error {
        super::domain::OutputValidationError::EmptyName => {
            "The output name is required.".to_owned()
        }
        super::domain::OutputValidationError::DirectoryDoesNotExist => {
            "The destination folder does not exist.".to_owned()
        }
    }
}
