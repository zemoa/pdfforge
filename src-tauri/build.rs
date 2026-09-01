fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "inspect_merge_sources",
            "preview_merge_output",
            "start_merge",
            "cancel_merge",
        ]),
    ))
    .expect("failed to build Tauri application");
}
