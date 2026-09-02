use std::{
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};

use super::domain::{NormalizedRect, OutputSpec, RedactionPlan, SourceIncident, SourcePdf};

#[derive(Clone, Debug, PartialEq)]
pub struct TextWord {
    pub index: usize,
    pub text: String,
    pub bounds: Vec<NormalizedRect>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderedPage {
    pub page: usize,
    pub aspect_ratio: f32,
    pub png_data_url: String,
    pub words: Vec<TextWord>,
}

pub trait SourceInspector: Send + Sync {
    fn inspect(&self, paths: &[PathBuf]) -> Result<SourcePdf, SourceIncident>;
}

pub trait PageRenderer: Send + Sync {
    fn render(&self, source: &Path, page: usize) -> Result<RenderedPage, String>;
}

pub trait PdfRedactionEngine: Send + Sync {
    fn redact(
        &self,
        source: &SourcePdf,
        plan: &RedactionPlan,
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

pub fn apply_redaction(
    reservation: &dyn OutputReservation,
    engine: &dyn PdfRedactionEngine,
    source: &SourcePdf,
    plan: &RedactionPlan,
    output: &OutputSpec,
    cancelled: &AtomicBool,
    report_progress: &mut dyn FnMut(usize, usize),
) -> Result<PathBuf, String> {
    let path = reservation.reserve(output)?;
    if let Err(error) = engine.redact(source, plan, &path, cancelled, report_progress) {
        reservation.remove(&path);
        return Err(error);
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use std::{
        path::PathBuf,
        sync::{atomic::AtomicBool, Mutex},
    };

    use super::*;
    use crate::redaction::domain::NormalizedRect;

    struct FailingEngine;

    impl PdfRedactionEngine for FailingEngine {
        fn redact(
            &self,
            _source: &SourcePdf,
            _plan: &RedactionPlan,
            _output: &Path,
            _cancelled: &AtomicBool,
            _report_progress: &mut dyn FnMut(usize, usize),
        ) -> Result<(), String> {
            Err("cancelled".to_owned())
        }
    }

    struct ReportingEngine;

    impl PdfRedactionEngine for ReportingEngine {
        fn redact(
            &self,
            source: &SourcePdf,
            _plan: &RedactionPlan,
            _output: &Path,
            _cancelled: &AtomicBool,
            report_progress: &mut dyn FnMut(usize, usize),
        ) -> Result<(), String> {
            report_progress(source.page_count, source.page_count);
            Ok(())
        }
    }

    #[derive(Default)]
    struct TrackingReservation {
        removed: Mutex<Vec<PathBuf>>,
    }

    impl OutputReservation for TrackingReservation {
        fn preview(&self, _output: &OutputSpec) -> Result<PathBuf, String> {
            unreachable!("the execution use case reserves output")
        }

        fn reserve(&self, _output: &OutputSpec) -> Result<PathBuf, String> {
            Ok(PathBuf::from("masked.pdf"))
        }

        fn remove(&self, path: &Path) {
            self.removed
                .lock()
                .expect("test mutex is available")
                .push(path.to_path_buf());
        }
    }

    #[test]
    fn removes_the_reserved_output_when_the_engine_fails() {
        let source = SourcePdf {
            path: PathBuf::from("source.pdf"),
            name: "source.pdf".to_owned(),
            page_count: 1,
        };
        let rectangle = NormalizedRect::new(0.1, 0.1, 0.2, 0.2).expect("fixture is valid");
        let plan = RedactionPlan::new(vec![(1, vec![rectangle])], 1).expect("plan is valid");
        let reservation = TrackingReservation::default();
        let output = OutputSpec {
            directory: PathBuf::from("."),
            file_name: "masked.pdf".to_owned(),
        };

        let error = apply_redaction(
            &reservation,
            &FailingEngine,
            &source,
            &plan,
            &output,
            &AtomicBool::new(true),
            &mut |_, _| {},
        )
        .expect_err("the engine fails");

        assert_eq!(error, "cancelled");
        assert_eq!(
            reservation
                .removed
                .lock()
                .expect("test mutex is available")
                .as_slice(),
            [PathBuf::from("masked.pdf")]
        );
    }

    #[test]
    fn forwards_progress_from_the_redaction_engine() {
        let source = SourcePdf {
            path: PathBuf::from("source.pdf"),
            name: "source.pdf".to_owned(),
            page_count: 2,
        };
        let rectangle = NormalizedRect::new(0.1, 0.1, 0.2, 0.2).expect("fixture is valid");
        let plan = RedactionPlan::new(vec![(1, vec![rectangle])], 2).expect("plan is valid");
        let reservation = TrackingReservation::default();
        let output = OutputSpec {
            directory: PathBuf::from("."),
            file_name: "masked.pdf".to_owned(),
        };
        let mut progress = (0, 0);

        apply_redaction(
            &reservation,
            &ReportingEngine,
            &source,
            &plan,
            &output,
            &AtomicBool::new(false),
            &mut |current, total| progress = (current, total),
        )
        .expect("the engine succeeds");

        assert_eq!(progress, (2, 2));
    }
}
