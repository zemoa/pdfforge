fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "inspect_merge_sources",
            "preview_merge_output",
            "start_merge",
            "cancel_merge",
            "inspect_split_source",
            "render_split_thumbnails",
            "preview_split_output",
            "start_split",
            "cancel_split",
        ]),
    ))
    .expect("failed to build Tauri application");
}
