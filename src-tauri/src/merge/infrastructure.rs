use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

use lopdf::{Document, Object, ObjectId};

use super::{
    application::{OutputReservation, PdfMergeEngine, SourceInspector},
    domain::{
        display_name, InteractiveWarning, OutputSpec, SourceIncident, SourceInspection, SourcePdf,
    },
};

pub struct LocalSourceInspector;

impl SourceInspector for LocalSourceInspector {
    fn inspect(&self, selected_paths: &[PathBuf]) -> SourceInspection {
        let mut inspection = SourceInspection::default();

        for path in selected_paths {
            if path.is_dir() {
                let mut children = match fs::read_dir(path) {
                    Ok(entries) => entries
                        .filter_map(Result::ok)
                        .map(|entry| entry.path())
                        .filter(|entry| entry.is_file())
                        .collect::<Vec<_>>(),
                    Err(_) => {
                        inspection.incidents.push(SourceIncident::Inaccessible {
                            path: path.clone(),
                            name: display_name(path),
                        });
                        continue;
                    }
                };
                children.sort_by_key(|child| display_name(child).to_lowercase());
                for child in children {
                    inspect_file(&child, &mut inspection);
                }
            } else {
                inspect_file(path, &mut inspection);
            }
        }

        inspection
    }
}

fn inspect_file(path: &Path, inspection: &mut SourceInspection) {
    if !is_pdf_path(path) {
        inspection.ignored_non_pdfs.push(path.to_path_buf());
        return;
    }

    if let Err(_) = fs::metadata(path) {
        inspection.incidents.push(SourceIncident::Inaccessible {
            path: path.to_path_buf(),
            name: display_name(path),
        });
        return;
    }

    match Document::load(path) {
        Ok(document) if document.is_encrypted() || document.was_encrypted() => {
            inspection
                .incidents
                .push(SourceIncident::PasswordProtected {
                    path: path.to_path_buf(),
                    name: display_name(path),
                });
        }
        Ok(document) => inspection.accepted.push(SourcePdf {
            path: path.to_path_buf(),
            name: display_name(path),
            page_count: document.get_pages().len(),
            warnings: detect_interactive_warnings(&document),
        }),
        Err(_) => inspection.incidents.push(SourceIncident::Unreadable {
            path: path.to_path_buf(),
            name: display_name(path),
        }),
    }
}

fn is_pdf_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("pdf"))
}

fn detect_interactive_warnings(document: &Document) -> Vec<InteractiveWarning> {
    let mut warnings = Vec::new();
    for object in document.objects.values() {
        let Ok(dictionary) = object.as_dict() else {
            continue;
        };
        if dictionary.get(b"Outlines").is_ok() {
            warnings.push(InteractiveWarning::Bookmarks);
        }
        if dictionary.get(b"AcroForm").is_ok() {
            warnings.push(InteractiveWarning::Forms);
        }
        if dictionary.get(b"StructTreeRoot").is_ok() {
            warnings.push(InteractiveWarning::TaggedStructure);
        }
        if dictionary.get(b"Names").is_ok() {
            warnings.push(InteractiveWarning::NamedDestinations);
        }
    }
    warnings.sort();
    warnings.dedup();
    warnings
}

pub struct LopdfMergeEngine;

impl PdfMergeEngine for LopdfMergeEngine {
    fn merge(
        &self,
        sources: &[SourcePdf],
        output: &Path,
        cancelled: &AtomicBool,
        report_progress: &mut dyn FnMut(usize, usize),
    ) -> Result<(), String> {
        if sources.len() < 2 {
            return Err("At least two PDF files are required.".to_owned());
        }

        let total_pages = sources
            .iter()
            .map(|source| source.page_count)
            .sum::<usize>();
        let mut completed_pages = 0;
        let mut max_id = 1;
        let mut documents_pages = BTreeMap::<ObjectId, Object>::new();
        let mut documents_objects = BTreeMap::<ObjectId, Object>::new();
        let mut merged = Document::with_version("1.5");

        for source in sources {
            ensure_not_cancelled(cancelled)?;
            let mut document = Document::load(&source.path)
                .map_err(|_| format!("{} can no longer be read.", source.name))?;
            if document.is_encrypted() || document.was_encrypted() {
                return Err(format!("{} is password protected.", source.name));
            }

            document.renumber_objects_with(max_id);
            max_id = document.max_id + 1;

            for object_id in document.get_pages().into_values() {
                ensure_not_cancelled(cancelled)?;
                let page = document
                    .get_object(object_id)
                    .map_err(|_| format!("{} contains an invalid page.", source.name))?
                    .to_owned();
                documents_pages.insert(object_id, page);
                completed_pages += 1;
                report_progress(completed_pages, total_pages);
            }
            documents_objects.extend(document.objects);
        }

        let mut catalog_object: Option<(ObjectId, Object)> = None;
        let mut pages_object: Option<(ObjectId, Object)> = None;
        for (object_id, object) in &documents_objects {
            match object.type_name().unwrap_or(b"") {
                b"Catalog" => {
                    if catalog_object.is_none() {
                        catalog_object = Some((*object_id, object.clone()));
                    }
                }
                b"Pages" => {
                    if pages_object.is_none() {
                        pages_object = Some((*object_id, object.clone()));
                    }
                }
                b"Page" | b"Outlines" | b"Outline" => {}
                _ => {
                    merged.objects.insert(*object_id, object.clone());
                }
            }
        }

        let (catalog_id, catalog) =
            catalog_object.ok_or_else(|| "A PDF catalog is missing.".to_owned())?;
        let (pages_id, pages) =
            pages_object.ok_or_else(|| "A PDF page tree is missing.".to_owned())?;

        for (object_id, page) in &documents_pages {
            let mut page = page.clone();
            let dictionary = page
                .as_dict_mut()
                .map_err(|_| "A PDF page is malformed.".to_owned())?;
            dictionary.set("Parent", pages_id);
            merged.objects.insert(*object_id, page);
        }

        let mut pages = pages
            .as_dict()
            .map_err(|_| "The PDF page tree is malformed.".to_owned())?
            .clone();
        pages.set("Count", documents_pages.len() as u32);
        pages.set(
            "Kids",
            documents_pages
                .keys()
                .copied()
                .map(Object::Reference)
                .collect::<Vec<_>>(),
        );
        merged.objects.insert(pages_id, Object::Dictionary(pages));

        let mut catalog = catalog
            .as_dict()
            .map_err(|_| "The PDF catalog is malformed.".to_owned())?
            .clone();
        catalog.set("Pages", pages_id);
        catalog.remove(b"Outlines");
        catalog.remove(b"AcroForm");
        catalog.remove(b"StructTreeRoot");
        merged
            .objects
            .insert(catalog_id, Object::Dictionary(catalog));
        merged.trailer.set("Root", catalog_id);
        merged.max_id = merged.objects.len() as u32;
        merged.renumber_objects();
        ensure_not_cancelled(cancelled)?;
        merged.save(output).map_err(|error| error.to_string())?;
        report_progress(total_pages, total_pages);
        Ok(())
    }
}

fn ensure_not_cancelled(cancelled: &AtomicBool) -> Result<(), String> {
    if cancelled.load(Ordering::Relaxed) {
        Err("The merge was cancelled.".to_owned())
    } else {
        Ok(())
    }
}

pub struct LocalOutputReservation;

impl OutputReservation for LocalOutputReservation {
    fn preview(&self, output: &OutputSpec) -> Result<PathBuf, String> {
        ensure_directory_writable(&output.directory)?;
        Ok(next_available_path(output))
    }

    fn reserve(&self, output: &OutputSpec) -> Result<PathBuf, String> {
        ensure_directory_writable(&output.directory)?;
        for number in 0usize.. {
            let candidate = numbered_path(output, number);
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&candidate)
            {
                Ok(_) => return Ok(candidate),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(_) => return Err("The destination folder is not writable.".to_owned()),
            }
        }
        unreachable!("an incrementing suffix always has a next value")
    }

    fn remove(&self, path: &Path) {
        let _ = fs::remove_file(path);
    }
}

fn ensure_directory_writable(directory: &Path) -> Result<(), String> {
    if !directory.is_dir() {
        return Err("The destination folder does not exist.".to_owned());
    }
    let probe = directory.join(format!(".pdfforge-write-check-{}", std::process::id()));
    let file = OpenOptions::new().write(true).create_new(true).open(&probe);
    match file {
        Ok(_) => {
            let _ = fs::remove_file(probe);
            Ok(())
        }
        Err(_) => Err("The destination folder is not writable.".to_owned()),
    }
}

fn next_available_path(output: &OutputSpec) -> PathBuf {
    (0usize..)
        .map(|number| numbered_path(output, number))
        .find(|candidate| !candidate.exists())
        .expect("an incrementing suffix always has a next value")
}

fn numbered_path(output: &OutputSpec, number: usize) -> PathBuf {
    if number == 0 {
        return output.candidate_path();
    }
    let stem = output
        .file_name
        .strip_suffix(".pdf")
        .or_else(|| output.file_name.strip_suffix(".PDF"))
        .unwrap_or(&output.file_name);
    output.directory.join(format!("{stem}-{number}.pdf"))
}
