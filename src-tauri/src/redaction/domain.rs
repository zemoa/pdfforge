use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourcePdf {
    pub path: PathBuf,
    pub name: String,
    pub page_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceIncident {
    NotPdf,
    PasswordProtected,
    Unreadable,
    Inaccessible,
    MultipleSources,
    EmptyDocument,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NormalizedRect {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

impl NormalizedRect {
    pub fn new(left: f32, top: f32, width: f32, height: f32) -> Option<Self> {
        if ![left, top, width, height]
            .iter()
            .all(|value| value.is_finite())
            || left < 0.0
            || top < 0.0
            || width <= 0.0
            || height <= 0.0
            || left + width > 1.0
            || top + height > 1.0
        {
            return None;
        }
        Some(Self {
            left,
            top,
            width,
            height,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RedactionPlan {
    selections: BTreeMap<usize, Vec<NormalizedRect>>,
}

impl RedactionPlan {
    pub fn new(
        selections: Vec<(usize, Vec<NormalizedRect>)>,
        page_count: usize,
    ) -> Result<Self, RedactionValidationError> {
        let mut by_page = BTreeMap::new();
        for (page, rectangles) in selections {
            if page == 0 || page > page_count {
                return Err(RedactionValidationError::PageOutOfBounds);
            }
            if rectangles.is_empty() {
                continue;
            }
            by_page
                .entry(page)
                .or_insert_with(Vec::new)
                .extend(rectangles);
        }
        if by_page.is_empty() {
            return Err(RedactionValidationError::EmptySelection);
        }
        Ok(Self {
            selections: by_page,
        })
    }

    pub fn rectangles_for_page(&self, page: usize) -> &[NormalizedRect] {
        self.selections
            .get(&page)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutputSpec {
    pub directory: PathBuf,
    pub file_name: String,
}

impl OutputSpec {
    pub fn new(directory: PathBuf, name: &str) -> Result<Self, RedactionValidationError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(RedactionValidationError::EmptyOutputName);
        }
        if !directory.is_dir() {
            return Err(RedactionValidationError::DirectoryDoesNotExist);
        }
        Ok(Self {
            directory,
            file_name: if name.to_ascii_lowercase().ends_with(".pdf") {
                name.to_owned()
            } else {
                format!("{name}.pdf")
            },
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RedactionValidationError {
    EmptySelection,
    PageOutOfBounds,
    EmptyOutputName,
    DirectoryDoesNotExist,
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
    fn accepts_a_rectangle_inside_the_page() {
        assert_eq!(
            NormalizedRect::new(0.1, 0.2, 0.3, 0.4),
            Some(NormalizedRect {
                left: 0.1,
                top: 0.2,
                width: 0.3,
                height: 0.4,
            })
        );
    }

    #[test]
    fn rejects_a_rectangle_outside_the_page() {
        assert_eq!(NormalizedRect::new(0.8, 0.1, 0.3, 0.2), None);
    }

    #[test]
    fn rejects_a_plan_without_a_redaction() {
        assert_eq!(
            RedactionPlan::new(vec![(1, vec![])], 1),
            Err(RedactionValidationError::EmptySelection)
        );
    }

    #[test]
    fn rejects_a_plan_that_references_a_missing_page() {
        let rectangle = NormalizedRect::new(0.1, 0.1, 0.2, 0.2).expect("fixture is valid");

        assert_eq!(
            RedactionPlan::new(vec![(2, vec![rectangle])], 1),
            Err(RedactionValidationError::PageOutOfBounds)
        );
    }

    #[test]
    fn normalizes_the_output_extension() {
        let output = OutputSpec::new(std::env::temp_dir(), "masked").expect("output is valid");

        assert_eq!(output.file_name, "masked.pdf");
    }
}
