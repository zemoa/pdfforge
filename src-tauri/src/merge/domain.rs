use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourcePdf {
    pub path: PathBuf,
    pub name: String,
    pub page_count: usize,
    pub warnings: Vec<InteractiveWarning>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceIncident {
    PasswordProtected { path: PathBuf, name: String },
    Unreadable { path: PathBuf, name: String },
    Inaccessible { path: PathBuf, name: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum InteractiveWarning {
    Bookmarks,
    Forms,
    TaggedStructure,
    NamedDestinations,
}

#[derive(Clone, Debug, Default)]
pub struct SourceInspection {
    pub accepted: Vec<SourcePdf>,
    pub ignored_non_pdfs: Vec<PathBuf>,
    pub incidents: Vec<SourceIncident>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutputSpec {
    pub directory: PathBuf,
    pub file_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OutputValidationError {
    EmptyName,
    DirectoryDoesNotExist,
}

impl OutputSpec {
    pub fn new(directory: PathBuf, name: &str) -> Result<Self, OutputValidationError> {
        let stem = name.trim();
        if stem.is_empty() {
            return Err(OutputValidationError::EmptyName);
        }

        let file_name = if stem.to_ascii_lowercase().ends_with(".pdf") {
            stem.to_owned()
        } else {
            format!("{stem}.pdf")
        };

        if !directory.is_dir() {
            return Err(OutputValidationError::DirectoryDoesNotExist);
        }

        Ok(Self {
            directory,
            file_name,
        })
    }

    pub fn candidate_path(&self) -> PathBuf {
        self.directory.join(&self.file_name)
    }
}

pub fn display_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| path.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_a_missing_pdf_extension() {
        let directory = std::env::temp_dir();
        let output = OutputSpec::new(directory, "fusion").expect("output is valid");

        assert_eq!(output.file_name, "fusion.pdf");
    }

    #[test]
    fn preserves_an_existing_pdf_extension_case_insensitively() {
        let directory = std::env::temp_dir();
        let output = OutputSpec::new(directory, "fusion.PDF").expect("output is valid");

        assert_eq!(output.file_name, "fusion.PDF");
    }
}
