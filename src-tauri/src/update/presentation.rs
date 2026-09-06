use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    sync::{atomic::AtomicBool, Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_opener::OpenerExt;

use crate::{
    merge::presentation::MergeRuntime, redaction::presentation::RedactionRuntime,
    split::presentation::SplitRuntime,
};

use super::{
    application::select_update,
    domain::{CheckResult, Platform, Release},
    infrastructure::{download_and_verify, fetch_releases, parse_version},
};

const UPDATE_EVENT: &str = "update-event";
const UPDATE_ROOT: &str = "updates";
const PREVIOUS_FILE: &str = "previous/PDFForge";
const NOTIFICATION_FILE: &str = "updated-to";
const FAILURE_FILE: &str = "install-failed";

#[derive(Clone, Default)]
pub struct UpdateRuntime {
    active: Arc<Mutex<Option<Arc<AtomicBool>>>>,
    available: Arc<Mutex<Option<Release>>>,
}

impl UpdateRuntime {
    fn begin(&self) -> Result<Arc<AtomicBool>, String> {
        let mut active = self
            .active
            .lock()
            .map_err(|_| "The update state is unavailable.".to_owned())?;
        if active.is_some() {
            return Err("An update is already in progress.".to_owned());
        }
        let cancelled = Arc::new(AtomicBool::new(false));
        *active = Some(cancelled.clone());
        Ok(cancelled)
    }

    fn finish(&self) {
        if let Ok(mut active) = self.active.lock() {
            *active = None;
        }
    }

    fn cancel(&self) {
        if let Ok(active) = self.active.lock() {
            if let Some(cancelled) = active.as_ref() {
                cancelled.store(true, std::sync::atomic::Ordering::Relaxed);
            }
        }
    }

    fn set_available(&self, release: Option<Release>) {
        if let Ok(mut available) = self.available.lock() {
            *available = release;
        }
    }

    fn available(&self) -> Result<Release, String> {
        self.available
            .lock()
            .map_err(|_| "The update state is unavailable.".to_owned())?
            .clone()
            .ok_or_else(|| "Search for an update before downloading it.".to_owned())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatusDto {
    installed_version: String,
    rollback_available: bool,
    updated_to: Option<String>,
    installation_error: bool,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum CheckResultDto {
    UpToDate,
    Unsupported { version: String },
    Available { version: String, notes: String },
}

#[derive(Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum UpdateEventDto {
    Progress { downloaded: u64, total: Option<u64> },
    Cancelled,
    ManualDownload { version: String },
    Failed { message: String },
}

#[tauri::command]
pub fn update_status(app: AppHandle) -> Result<UpdateStatusDto, String> {
    let root = update_root(&app)?;
    let notification = root.join(NOTIFICATION_FILE);
    let failure = root.join(FAILURE_FILE);
    let updated_to = fs::read_to_string(&notification)
        .ok()
        .map(|value| value.trim().to_owned());
    if updated_to.is_some() {
        let _ = fs::remove_file(notification);
    }
    let installation_error = failure.is_file();
    if installation_error {
        let _ = fs::remove_file(failure);
    }
    Ok(UpdateStatusDto {
        installed_version: env!("CARGO_PKG_VERSION").to_owned(),
        rollback_available: previous_path(&root).is_file(),
        updated_to,
        installation_error,
    })
}

#[tauri::command]
pub async fn check_for_update(
    runtime: State<'_, UpdateRuntime>,
    locale: String,
) -> Result<CheckResultDto, String> {
    let runtime = runtime.inner().clone();
    let checked = tauri::async_runtime::spawn_blocking(move || {
        let installed = parse_version(env!("CARGO_PKG_VERSION"))?;
        let platform = Platform::current()?;
        let result = select_update(&installed, platform, fetch_releases()?);
        runtime.set_available(match &result {
            CheckResult::Available { release } => Some((**release).clone()),
            _ => None,
        });
        Ok::<_, String>((result, locale))
    })
    .await
    .map_err(|_| "The update search could not be completed.".to_owned())??;
    Ok(check_dto(checked.0, &checked.1))
}

#[tauri::command]
pub fn start_update(
    app: AppHandle,
    runtime: State<'_, UpdateRuntime>,
    merge: State<'_, MergeRuntime>,
    split: State<'_, SplitRuntime>,
    redaction: State<'_, RedactionRuntime>,
) -> Result<(), String> {
    if merge.is_active() || split.is_active() || redaction.is_active() {
        return Err(
            "A PDF operation is in progress. Finish or cancel it before installing an update."
                .to_owned(),
        );
    }
    let release = runtime.available()?;
    let platform = Platform::current()?;
    let asset = platform
        .asset(&release)
        .cloned()
        .ok_or_else(|| "This update is not available for this system.".to_owned())?;
    let runtime = runtime.inner().clone();
    let cancelled = runtime.begin()?;
    tauri::async_runtime::spawn_blocking(move || {
        let result = download_install(&app, &release, &asset, platform, cancelled.as_ref());
        match result {
            Ok(InstallOutcome::ManualDownload) => {
                let _ = app.emit(
                    UPDATE_EVENT,
                    UpdateEventDto::ManualDownload {
                        version: release.version.to_string(),
                    },
                );
            }
            Ok(InstallOutcome::Restarting) => {}
            Err(message) if message == "Update download cancelled." => {
                let _ = app.emit(UPDATE_EVENT, UpdateEventDto::Cancelled);
            }
            Err(message) => {
                let _ = app.emit(UPDATE_EVENT, UpdateEventDto::Failed { message });
            }
        }
        runtime.finish();
    });
    Ok(())
}

#[tauri::command]
pub fn cancel_update(runtime: State<'_, UpdateRuntime>) {
    runtime.cancel();
}

#[tauri::command]
pub fn restore_previous_update(
    app: AppHandle,
    merge: State<'_, MergeRuntime>,
    split: State<'_, SplitRuntime>,
    redaction: State<'_, RedactionRuntime>,
) -> Result<(), String> {
    if merge.is_active() || split.is_active() || redaction.is_active() {
        return Err(
            "A PDF operation is in progress. Finish or cancel it before restoring a version."
                .to_owned(),
        );
    }
    let root = update_root(&app)?;
    let previous = previous_path(&root);
    if !previous.is_file() {
        return Err("There is no previous version to restore.".to_owned());
    }
    let restore = root.join("temporary").join("restore-candidate");
    fs::create_dir_all(
        restore
            .parent()
            .ok_or_else(|| "The restore folder is unavailable.".to_owned())?,
    )
    .map_err(|_| "The restore folder is unavailable.".to_owned())?;
    fs::copy(&previous, &restore)
        .map_err(|_| "The previous version cannot be prepared for restoration.".to_owned())?;
    replace_with(&app, &restore, "", false)
}

enum InstallOutcome {
    ManualDownload,
    Restarting,
}

fn check_dto(result: CheckResult, locale: &str) -> CheckResultDto {
    match result {
        CheckResult::UpToDate => CheckResultDto::UpToDate,
        CheckResult::Unsupported { version } => CheckResultDto::Unsupported {
            version: version.to_string(),
        },
        CheckResult::Available { release } => CheckResultDto::Available {
            version: release.version.to_string(),
            notes: if locale == "fr" {
                release.notes_fr
            } else {
                release.notes_en
            },
        },
    }
}

fn download_install(
    app: &AppHandle,
    release: &Release,
    asset: &super::domain::ReleaseAsset,
    platform: Platform,
    cancelled: &AtomicBool,
) -> Result<InstallOutcome, String> {
    let root = update_root(app)?;
    let staging = root.join("temporary").join(release.version.to_string());
    let _ = fs::remove_dir_all(&staging);
    fs::create_dir_all(&staging).map_err(|_| "The update folder cannot be created.".to_owned())?;
    let candidate = staging.join(&asset.file_name);
    let result = download_and_verify(asset, &candidate, cancelled, |downloaded, total| {
        let _ = app.emit(UPDATE_EVENT, UpdateEventDto::Progress { downloaded, total });
    });
    if let Err(error) = result {
        let _ = fs::remove_dir_all(&staging);
        return Err(error);
    }

    let current = current_artifact_path()?;
    if !is_replaceable(&current) {
        let downloads = app
            .path()
            .download_dir()
            .map_err(|_| "The Downloads folder is unavailable.".to_owned())?;
        fs::create_dir_all(&downloads)
            .map_err(|_| "The Downloads folder is unavailable.".to_owned())?;
        let destination = unique_path(&downloads, &asset.file_name);
        fs::rename(&candidate, &destination)
            .or_else(|_| fs::copy(&candidate, &destination).map(|_| ()))
            .map_err(|_| "The verified update could not be saved to Downloads.".to_owned())?;
        let _ = fs::remove_dir_all(&staging);
        app.opener()
            .open_path(downloads.display().to_string(), None::<&str>)
            .map_err(|_| "The Downloads folder could not be opened.".to_owned())?;
        return Ok(InstallOutcome::ManualDownload);
    }
    replace_with(app, &candidate, &release.version.to_string(), true)?;
    match platform {
        Platform::LinuxX64 | Platform::WindowsX64 => Ok(InstallOutcome::Restarting),
    }
}

fn replace_with(
    app: &AppHandle,
    candidate: &Path,
    updated_to: &str,
    keep_rollback: bool,
) -> Result<(), String> {
    let root = update_root(app)?;
    let current = current_artifact_path()?;
    let previous = previous_path(&root);
    let previous_dir = previous
        .parent()
        .ok_or_else(|| "The rollback folder is unavailable.".to_owned())?;
    let _ = fs::remove_dir_all(previous_dir);
    if keep_rollback {
        fs::create_dir_all(previous_dir)
            .map_err(|_| "The rollback folder cannot be created.".to_owned())?;
    }

    #[cfg(target_os = "windows")]
    {
        return replace_windows(
            app,
            candidate,
            &current,
            &previous,
            updated_to,
            keep_rollback,
        );
    }
    #[cfg(not(target_os = "windows"))]
    {
        if keep_rollback {
            fs::copy(&current, &previous)
                .map_err(|_| "The current application cannot be backed up.".to_owned())?;
        }
        let temporary = current.with_extension("pdfforge-new");
        fs::copy(candidate, &temporary).map_err(|_| "The update cannot be prepared.".to_owned())?;
        copy_permissions(&current, &temporary)?;
        fs::rename(&temporary, &current)
            .map_err(|_| "The application could not be replaced.".to_owned())?;
        if !updated_to.is_empty() {
            fs::write(root.join(NOTIFICATION_FILE), updated_to)
                .map_err(|_| "The update notification could not be prepared.".to_owned())?;
        }
        Command::new(&current)
            .spawn()
            .map_err(|_| "The updated application could not be restarted.".to_owned())?;
        app.exit(0);
        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn replace_windows(
    app: &AppHandle,
    candidate: &Path,
    current: &Path,
    previous: &Path,
    updated_to: &str,
    keep_rollback: bool,
) -> Result<(), String> {
    let root = update_root(app)?;
    let script = root.join("replace-update.cmd");
    let notification = root.join(NOTIFICATION_FILE);
    let quote = |path: &Path| path.display().to_string().replace('"', "''");
    let backup = if keep_rollback {
        format!(
            "copy /Y \"{}\" \"{}\" >nul || goto fail\r\n",
            quote(current),
            quote(previous)
        )
    } else {
        String::new()
    };
    let body = format!(
        "@echo off\r\ntimeout /t 2 /nobreak >nul\r\n{}copy /Y \"{}\" \"{}.new\" >nul || goto fail\r\nmove /Y \"{}.new\" \"{}\" >nul || goto fail\r\n{}\r\nstart \"\" \"{}\"\r\ndel \"{}\"\r\ndel \"%~f0\"\r\nexit /b 0\r\n:fail\r\necho failed>\"{}\"\r\ndel \"{}\"\r\nstart \"\" \"{}\"\r\ndel \"%~f0\"\r\n",
        backup, quote(candidate), quote(current), quote(current), quote(current),
        if updated_to.is_empty() { String::new() } else { format!("echo {}>\"{}\"", updated_to, quote(&notification)) },
        quote(current), quote(candidate), quote(&root.join(FAILURE_FILE)), quote(candidate), quote(current)
    );
    fs::write(&script, body)
        .map_err(|_| "The update installer could not be prepared.".to_owned())?;
    Command::new("cmd")
        .args(["/C", &script.display().to_string()])
        .spawn()
        .map_err(|_| "The update installer could not be started.".to_owned())?;
    app.exit(0);
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn copy_permissions(source: &Path, target: &Path) -> Result<(), String> {
    fs::set_permissions(
        target,
        fs::metadata(source)
            .map_err(|_| "The application permissions are unavailable.".to_owned())?
            .permissions(),
    )
    .map_err(|_| "The updated application permissions could not be set.".to_owned())
}

fn current_artifact_path() -> Result<PathBuf, String> {
    #[cfg(target_os = "linux")]
    if let Some(appimage) = env::var_os("APPIMAGE") {
        return Ok(PathBuf::from(appimage));
    }
    env::current_exe().map_err(|_| "The current application location is unavailable.".to_owned())
}

fn is_replaceable(path: &Path) -> bool {
    let Some(directory) = path.parent() else {
        return false;
    };
    let probe = directory.join(format!(
        ".pdfforge-update-write-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |value| value.as_nanos())
    ));
    FileGuard::create(&probe).is_ok()
}

struct FileGuard(PathBuf);

impl FileGuard {
    fn create(path: &Path) -> std::io::Result<Self> {
        fs::write(path, [])?;
        Ok(Self(path.to_owned()))
    }
}

impl Drop for FileGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn update_root(app: &AppHandle) -> Result<PathBuf, String> {
    let root = app
        .path()
        .app_local_data_dir()
        .map_err(|_| "The local update folder is unavailable.".to_owned())?
        .join(UPDATE_ROOT);
    fs::create_dir_all(&root).map_err(|_| "The local update folder is unavailable.".to_owned())?;
    Ok(root)
}

fn previous_path(root: &Path) -> PathBuf {
    let extension = if cfg!(target_os = "windows") {
        "exe"
    } else {
        "AppImage"
    };
    root.join(PREVIOUS_FILE).with_extension(extension)
}

fn unique_path(directory: &Path, file_name: &str) -> PathBuf {
    let candidate = directory.join(file_name);
    if !candidate.exists() {
        return candidate;
    }
    let path = Path::new(file_name);
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("PDFForge");
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    (1_u32..)
        .map(|number| directory.join(format!("{stem}-{number}.{extension}")))
        .find(|candidate| !candidate.exists())
        .expect("an unused numbered path exists")
}
