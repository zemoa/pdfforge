use serde::Serialize;

/// Stable, user-facing error identifiers shared by all Tauri commands.
///
/// Technical error details stay in the backend. The renderer translates these
/// identifiers in the active locale before showing an error to the user.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ErrorCode {
    SourceNotPdf,
    SourcePasswordProtected,
    SourceUnreadable,
    SourceInaccessible,
    SourceMultiple,
    SourceEmpty,
    MergeSourcesRequired,
    OutputNameRequired,
    DestinationMissing,
    DestinationNotWritable,
    SelectionRequired,
    PageUnavailable,
    GroupsRequired,
    GroupsOverlap,
    RedactionZoneInvalid,
    RendererUnavailable,
    OperationInProgress,
    PreparationChanged,
    MergeFailed,
    SplitFailed,
    RedactionFailed,
    UpdateCheckFailed,
    UpdateDownloadFailed,
    UpdateSpaceUnavailable,
    UpdateVerificationFailed,
    UpdateInstallFailed,
    UpdateRestoreFailed,
    UpdateUnavailable,
    PreviousVersionUnavailable,
    PdfOperationInProgress,
    Unexpected,
}

#[derive(Clone, Debug, Serialize)]
pub struct CommandError {
    pub code: ErrorCode,
}

impl CommandError {
    pub const fn new(code: ErrorCode) -> Self {
        Self { code }
    }
}

pub type CommandResult<T> = Result<T, CommandError>;

#[cfg(test)]
mod tests {
    use super::{CommandError, ErrorCode};

    #[test]
    fn serializes_only_a_stable_error_code() {
        let value = serde_json::to_value(CommandError::new(ErrorCode::DestinationMissing))
            .expect("error is serializable");

        assert_eq!(value, serde_json::json!({ "code": "destinationMissing" }));
    }
}
