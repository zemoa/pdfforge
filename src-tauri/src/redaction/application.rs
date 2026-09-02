use std::path::{Path, PathBuf};

use super::domain::{NormalizedRect, SourceIncident, SourcePdf};

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
