use std::{collections::BTreeSet, path::PathBuf};

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageSelection {
    pages: Vec<usize>,
}

impl PageSelection {
    pub fn new(mut pages: Vec<usize>, page_count: usize) -> Result<Self, SplitValidationError> {
        if pages.is_empty() {
            return Err(SplitValidationError::EmptySelection);
        }
        if pages.iter().any(|page| *page == 0 || *page > page_count) {
            return Err(SplitValidationError::PageOutOfBounds);
        }

        pages.sort_unstable();
        pages.dedup();
        Ok(Self { pages })
    }

    pub fn pages(&self) -> &[usize] {
        &self.pages
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SplitPlan {
    EachPage,
    Extract(PageSelection),
    Groups(Vec<PageSelection>),
}

impl SplitPlan {
    pub fn groups(
        groups: Vec<Vec<usize>>,
        page_count: usize,
    ) -> Result<Self, SplitValidationError> {
        if groups.is_empty() {
            return Err(SplitValidationError::EmptyGroups);
        }

        let selections = groups
            .into_iter()
            .map(|pages| PageSelection::new(pages, page_count))
            .collect::<Result<Vec<_>, _>>()?;
        let mut assigned_pages = BTreeSet::new();
        for group in &selections {
            for page in group.pages() {
                if !assigned_pages.insert(*page) {
                    return Err(SplitValidationError::OverlappingGroups);
                }
            }
        }
        Ok(Self::Groups(selections))
    }

    pub fn outputs(&self, page_count: usize) -> Vec<PageSelection> {
        match self {
            Self::EachPage => (1..=page_count)
                .map(|page| PageSelection { pages: vec![page] })
                .collect(),
            Self::Extract(selection) => vec![selection.clone()],
            Self::Groups(groups) => groups.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutputSpec {
    pub directory: PathBuf,
    pub base_name: String,
}

impl OutputSpec {
    pub fn new(directory: PathBuf, name: &str) -> Result<Self, SplitValidationError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(SplitValidationError::EmptyOutputName);
        }
        if !directory.is_dir() {
            return Err(SplitValidationError::DirectoryDoesNotExist);
        }
        let base_name = if name.to_ascii_lowercase().ends_with(".pdf") {
            name.to_owned()
        } else {
            format!("{name}.pdf")
        };
        Ok(Self {
            directory,
            base_name,
        })
    }

    pub fn output_names(&self, output_count: usize, batch_index: usize) -> Vec<String> {
        let stem = self
            .base_name
            .strip_suffix(".pdf")
            .or_else(|| self.base_name.strip_suffix(".PDF"))
            .unwrap_or(&self.base_name);
        if output_count == 1 {
            return vec![if batch_index == 0 {
                self.base_name.clone()
            } else {
                format!("{stem}-{batch_index}.pdf")
            }];
        }

        let batch_suffix = if batch_index == 0 {
            String::new()
        } else {
            format!("-{batch_index}")
        };
        (1..=output_count)
            .map(|ordinal| format!("{stem}{batch_suffix}-{ordinal}.pdf"))
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SplitValidationError {
    EmptySelection,
    PageOutOfBounds,
    EmptyGroups,
    OverlappingGroups,
    EmptyOutputName,
    DirectoryDoesNotExist,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_selected_pages_in_document_order() {
        let pages = PageSelection::new(vec![4, 1, 2], 4).expect("pages are valid");

        assert_eq!(pages.pages(), &[1, 2, 4]);
    }

    #[test]
    fn rejects_a_page_used_by_two_groups() {
        let error = SplitPlan::groups(vec![vec![1, 2], vec![2, 3]], 3).expect_err("groups overlap");

        assert_eq!(error, SplitValidationError::OverlappingGroups);
    }

    #[test]
    fn names_a_multiple_output_batch_consistently_after_a_collision() {
        let output = OutputSpec {
            directory: std::env::temp_dir(),
            base_name: "pages.pdf".to_owned(),
        };

        assert_eq!(output.output_names(2, 0), ["pages-1.pdf", "pages-2.pdf"]);
        assert_eq!(
            output.output_names(2, 1),
            ["pages-1-1.pdf", "pages-1-2.pdf"]
        );
    }
}
