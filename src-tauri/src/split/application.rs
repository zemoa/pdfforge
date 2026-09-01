use std::{
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};

use super::domain::{OutputSpec, PageSelection, SourceIncident, SourcePdf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Thumbnail {
    pub page: usize,
    pub png_data_url: String,
}

pub trait SourceInspector: Send + Sync {
    fn inspect(&self, paths: &[PathBuf]) -> Result<SourcePdf, SourceIncident>;
}

pub trait ThumbnailRenderer: Send + Sync {
    fn render(&self, source: &Path, pages: &[usize]) -> Result<Vec<Thumbnail>, String>;
}

pub trait PdfSplitEngine: Send + Sync {
    fn split(
        &self,
        source: &SourcePdf,
        outputs: &[(PathBuf, PageSelection)],
        cancelled: &AtomicBool,
        report_progress: &mut dyn FnMut(usize, usize),
    ) -> Result<(), String>;
}

pub trait OutputReservation: Send + Sync {
    fn preview(&self, output: &OutputSpec, output_count: usize) -> Result<Vec<PathBuf>, String>;
    fn reserve(&self, output: &OutputSpec, output_count: usize) -> Result<Vec<PathBuf>, String>;
    fn remove_all(&self, paths: &[PathBuf]);
}

pub fn execute_split(
    reservation: &dyn OutputReservation,
    engine: &dyn PdfSplitEngine,
    source: &SourcePdf,
    output: &OutputSpec,
    selections: Vec<PageSelection>,
    cancelled: &AtomicBool,
    report_progress: &mut dyn FnMut(usize, usize),
) -> Result<Vec<PathBuf>, String> {
    let paths = reservation.reserve(output, selections.len())?;
    let outputs = paths.iter().cloned().zip(selections).collect::<Vec<_>>();
    if let Err(error) = engine.split(source, &outputs, cancelled, report_progress) {
        reservation.remove_all(&paths);
        return Err(error);
    }
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use std::{
        path::PathBuf,
        sync::{atomic::AtomicBool, Mutex},
    };

    use super::*;
    use crate::split::domain::PageSelection;

    struct ReportingEngine;

    impl PdfSplitEngine for ReportingEngine {
        fn split(
            &self,
            _source: &SourcePdf,
            outputs: &[(PathBuf, PageSelection)],
            _cancelled: &AtomicBool,
            report_progress: &mut dyn FnMut(usize, usize),
        ) -> Result<(), String> {
            let total = outputs.iter().map(|(_, pages)| pages.pages().len()).sum();
            report_progress(total, total);
            Ok(())
        }
    }

    struct FailingEngine;

    impl PdfSplitEngine for FailingEngine {
        fn split(
            &self,
            _source: &SourcePdf,
            _outputs: &[(PathBuf, PageSelection)],
            _cancelled: &AtomicBool,
            _report_progress: &mut dyn FnMut(usize, usize),
        ) -> Result<(), String> {
            Err("cancelled".to_owned())
        }
    }

    #[derive(Default)]
    struct TrackingReservation {
        removed: Mutex<Vec<PathBuf>>,
    }

    impl OutputReservation for TrackingReservation {
        fn preview(
            &self,
            _output: &OutputSpec,
            _output_count: usize,
        ) -> Result<Vec<PathBuf>, String> {
            unreachable!("the execution use case reserves outputs")
        }

        fn reserve(
            &self,
            _output: &OutputSpec,
            _output_count: usize,
        ) -> Result<Vec<PathBuf>, String> {
            Ok(vec![PathBuf::from("output.pdf")])
        }

        fn remove_all(&self, paths: &[PathBuf]) {
            self.removed
                .lock()
                .expect("test mutex is available")
                .extend(paths.iter().cloned());
        }
    }

    #[test]
    fn reports_progress_for_the_total_number_of_exported_pages() {
        let source = SourcePdf {
            path: PathBuf::from("source.pdf"),
            name: "source.pdf".to_owned(),
            page_count: 3,
        };
        let outputs = vec![
            (
                PathBuf::from("first.pdf"),
                PageSelection::new(vec![1, 3], 3).expect("pages are valid"),
            ),
            (
                PathBuf::from("second.pdf"),
                PageSelection::new(vec![2], 3).expect("pages are valid"),
            ),
        ];
        let mut progress = (0, 0);

        ReportingEngine
            .split(
                &source,
                &outputs,
                &AtomicBool::new(false),
                &mut |current, total| {
                    progress = (current, total);
                },
            )
            .expect("split succeeds");

        assert_eq!(progress, (3, 3));
    }

    #[test]
    fn removes_every_reserved_output_when_the_engine_is_cancelled() {
        let source = SourcePdf {
            path: PathBuf::from("source.pdf"),
            name: "source.pdf".to_owned(),
            page_count: 1,
        };
        let reservation = TrackingReservation::default();
        let output = OutputSpec {
            directory: PathBuf::from("."),
            base_name: "output.pdf".to_owned(),
        };

        let error = execute_split(
            &reservation,
            &FailingEngine,
            &source,
            &output,
            vec![PageSelection::new(vec![1], 1).expect("page is valid")],
            &AtomicBool::new(true),
            &mut |_, _| {},
        )
        .expect_err("the engine cancels");

        assert_eq!(error, "cancelled");
        assert_eq!(
            reservation
                .removed
                .lock()
                .expect("test mutex is available")
                .as_slice(),
            [PathBuf::from("output.pdf")]
        );
    }
}
