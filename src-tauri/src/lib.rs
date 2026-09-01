//! PDFForge application entry point.
//!
//! Business capabilities are added as Rust domain modules and exposed through
//! narrowly-scoped Tauri command adapters. See ARCHITECTURE.md before adding one.

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running PDFForge");
}
