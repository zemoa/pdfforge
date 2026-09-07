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

use crate::error::{CommandError, CommandResult, ErrorCode};

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
    pub fn is_active(&self) -> bool {
        self.active.lock().is_ok_and(|active| active.is_some())
    }

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
        error: ErrorCode,
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
) -> CommandResult<OutputPreviewDto> {
    let output =
        OutputSpec::new(PathBuf::from(directory), &file_name).map_err(output_validation_code)?;
    let path = LocalOutputReservation
        .preview(&output)
        .map_err(|error| CommandError::new(output_error_code(&error)))?;
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
) -> CommandResult<()> {
    let inspection =
        LocalSourceInspector.inspect(&source_paths.iter().map(PathBuf::from).collect::<Vec<_>>());
    if inspection.accepted.len() != source_paths.len() || !inspection.incidents.is_empty() {
        return Err(CommandError::new(ErrorCode::PreparationChanged));
    }
    if inspection.accepted.len() < 2 {
        return Err(CommandError::new(ErrorCode::MergeSourcesRequired));
    }
    let output =
        OutputSpec::new(PathBuf::from(directory), &file_name).map_err(output_validation_code)?;
    let runtime = runtime.inner().clone();
    let cancelled = runtime
        .begin()
        .map_err(|_| CommandError::new(ErrorCode::OperationInProgress))?;
    let output_path = match LocalOutputReservation.reserve(&output) {
        Ok(path) => path,
        Err(error) => {
            runtime.finish();
            return Err(CommandError::new(output_error_code(&error)));
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
            Err(_) => {
                LocalOutputReservation.remove(&output_path);
                MergeEventDto::Failed {
                    error: ErrorCode::MergeFailed,
                }
            }
        };
        let _ = app.emit(MERGE_EVENT, event);
        runtime.finish();
    });
    Ok(())
}

#[tauri::command]
pub fn cancel_merge(runtime: State<'_, MergeRuntime>) -> CommandResult<()> {
    runtime
        .cancel()
        .map_err(|_| CommandError::new(ErrorCode::Unexpected))
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

fn output_validation_code(error: super::domain::OutputValidationError) -> CommandError {
    match error {
        super::domain::OutputValidationError::EmptyName => {
            CommandError::new(ErrorCode::OutputNameRequired)
        }
        super::domain::OutputValidationError::DirectoryDoesNotExist => {
            CommandError::new(ErrorCode::DestinationMissing)
        }
    }
}

fn output_error_code(error: &str) -> ErrorCode {
    match error {
        "The destination folder does not exist." => ErrorCode::DestinationMissing,
        "The destination folder is not writable." => ErrorCode::DestinationNotWritable,
        _ => ErrorCode::MergeFailed,
    }
}

#[cfg(test)]
mod tests {
    use super::{output_error_code, output_validation_code};
    use crate::{error::ErrorCode, merge::domain::OutputValidationError};

    #[test]
    fn maps_output_failures_without_exposing_the_backend_message() {
        assert_eq!(
            output_validation_code(OutputValidationError::DirectoryDoesNotExist).code,
            ErrorCode::DestinationMissing
        );
        assert_eq!(
            output_error_code("unexpected filesystem failure"),
            ErrorCode::MergeFailed
        );
    }
}
