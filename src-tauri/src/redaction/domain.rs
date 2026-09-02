use std::path::{Path, PathBuf};

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
}
