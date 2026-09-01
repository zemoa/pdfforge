//! PDFForge application entry point.
//!
//! Business capabilities are added as Rust domain modules and exposed through
//! narrowly-scoped Tauri command adapters. See ARCHITECTURE.md before adding one.

mod merge;
mod split;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(merge::presentation::MergeRuntime::default())
        .manage(split::presentation::SplitRuntime::default())
        .invoke_handler(tauri::generate_handler![
            merge::presentation::inspect_merge_sources,
            merge::presentation::preview_merge_output,
            merge::presentation::start_merge,
            merge::presentation::cancel_merge,
            split::presentation::inspect_split_source,
            split::presentation::render_split_thumbnails,
            split::presentation::preview_split_output,
            split::presentation::start_split,
            split::presentation::cancel_split,
        ])
        .run(tauri::generate_context!())
        .expect("error while running PDFForge");
}
