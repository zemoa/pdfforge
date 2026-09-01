use std::{
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};

use super::domain::{InteractiveWarning, OutputSpec, SourceInspection, SourcePdf};

pub trait SourceInspector: Send + Sync {
    fn inspect(&self, selected_paths: &[PathBuf]) -> SourceInspection;
}

pub trait PdfMergeEngine: Send + Sync {
    fn merge(
        &self,
        sources: &[SourcePdf],
        output: &Path,
        cancelled: &AtomicBool,
        report_progress: &mut dyn FnMut(usize, usize),
    ) -> Result<(), String>;
}

pub trait OutputReservation: Send + Sync {
    fn preview(&self, output: &OutputSpec) -> Result<PathBuf, String>;
    fn reserve(&self, output: &OutputSpec) -> Result<PathBuf, String>;
    fn remove(&self, path: &Path);
}

pub fn collect_warnings(sources: &[SourcePdf]) -> Vec<InteractiveWarning> {
    let mut warnings = sources
        .iter()
        .flat_map(|source| source.warnings.iter().copied())
        .collect::<Vec<_>>();
    warnings.sort();
    warnings.dedup();
    warnings
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::merge::domain::{InteractiveWarning, SourcePdf};

    #[test]
    fn deduplicates_preflight_warnings() {
        let source = |warnings| SourcePdf {
            path: PathBuf::from("source.pdf"),
            name: "source.pdf".to_owned(),
            page_count: 1,
            warnings,
        };

        assert_eq!(
            collect_warnings(&[
                source(vec![InteractiveWarning::Bookmarks]),
                source(vec![
                    InteractiveWarning::Bookmarks,
                    InteractiveWarning::Forms
                ]),
            ]),
            vec![InteractiveWarning::Bookmarks, InteractiveWarning::Forms]
        );
    }
}
