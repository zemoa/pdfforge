use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use reqwest::blocking::Client;
use semver::Version;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::domain::{stable_version, Release, ReleaseAsset};

const RELEASES_URL: &str = "https://api.github.com/repos/zemoa/pdfforge/releases";
const USER_AGENT: &str = "PDFForge";

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    draft: bool,
    prerelease: bool,
    published_at: Option<String>,
    body: Option<String>,
    assets: Vec<GithubAsset>,
}

#[derive(Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

pub fn fetch_releases() -> Result<Vec<Release>, String> {
    let client = client()?;
    let mut result = Vec::new();
    for page in 1.. {
        let url = format!("{RELEASES_URL}?per_page=100&page={page}");
        let response = client
            .get(url)
            .send()
            .map_err(|_| {
                "Unable to search for updates. Check your connection and try again.".to_owned()
            })?
            .error_for_status()
            .map_err(|_| {
                "Unable to search for updates. Check your connection and try again.".to_owned()
            })?;
        let releases: Vec<GithubRelease> = serde_json::from_str(
            &response
                .text()
                .map_err(|_| "The update service returned an invalid response.".to_owned())?,
        )
        .map_err(|_| "The update service returned an invalid response.".to_owned())?;
        let finished = releases.len() < 100;
        result.extend(releases.into_iter().filter_map(release_from_github));
        if finished {
            break;
        }
    }
    Ok(result)
}

fn release_from_github(release: GithubRelease) -> Option<Release> {
    if release.draft || release.prerelease || release.published_at.is_none() {
        return None;
    }
    let version = stable_version(&release.tag_name)?;
    let stem = format!("PDFForge-{version}");
    let linux_name = format!("{stem}-linux-x86_64.AppImage");
    let windows_name = format!("{stem}-windows-x86_64.exe");
    let notes = release.body.unwrap_or_default();
    let (notes_fr, notes_en) = release_notes(&notes);
    Some(Release {
        version,
        notes_fr,
        notes_en,
        linux: find_asset(&release.assets, &linux_name),
        windows: find_asset(&release.assets, &windows_name),
    })
}

fn find_asset(assets: &[GithubAsset], name: &str) -> Option<ReleaseAsset> {
    let url = assets
        .iter()
        .find(|asset| asset.name == name)?
        .browser_download_url
        .clone();
    let signature_url = assets
        .iter()
        .find(|asset| asset.name == format!("{name}.sig"))?
        .browser_download_url
        .clone();
    let checksum_url = assets
        .iter()
        .find(|asset| asset.name == format!("{name}.sha256"))?
        .browser_download_url
        .clone();
    Some(ReleaseAsset {
        url,
        signature_url,
        checksum_url,
        file_name: name.to_owned(),
    })
}

fn release_notes(body: &str) -> (String, String) {
    const FR_START: &str = "<!-- pdfforge-notes:fr:start -->";
    const FR_END: &str = "<!-- pdfforge-notes:fr:end -->";
    const EN_START: &str = "<!-- pdfforge-notes:en:start -->";
    const EN_END: &str = "<!-- pdfforge-notes:en:end -->";
    let section = |start: &str, end: &str| {
        body.split_once(start)
            .and_then(|(_, tail)| {
                tail.split_once(end)
                    .map(|(notes, _)| notes.trim().to_owned())
            })
            .unwrap_or_default()
    };
    (section(FR_START, FR_END), section(EN_START, EN_END))
}

pub fn download_and_verify(
    asset: &ReleaseAsset,
    destination: &Path,
    cancelled: &AtomicBool,
    mut progress: impl FnMut(u64, Option<u64>),
) -> Result<(), String> {
    let client = client()?;
    let signature = fetch_text(&client, &asset.signature_url)?;
    let checksum = fetch_text(&client, &asset.checksum_url)?;
    let expected_checksum = expected_checksum(&checksum)?;
    let mut response = client
        .get(&asset.url)
        .send()
        .map_err(|_| {
            "Unable to download the update. Check your connection and try again.".to_owned()
        })?
        .error_for_status()
        .map_err(|_| {
            "Unable to download the update. Check your connection and try again.".to_owned()
        })?;
    let total = response.content_length();
    let mut file = File::create(destination)
        .map_err(|_| "There is not enough space to download the update.".to_owned())?;
    let mut hash = Sha256::new();
    let mut bytes = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        if cancelled.load(Ordering::Relaxed) {
            let _ = fs::remove_file(destination);
            return Err("Update download cancelled.".to_owned());
        }
        let read = response
            .read(&mut buffer)
            .map_err(|_| "The update download failed.".to_owned())?;
        if read == 0 {
            break;
        }
        file.write_all(&buffer[..read])
            .map_err(|_| "There is not enough space to download the update.".to_owned())?;
        hash.update(&buffer[..read]);
        bytes += read as u64;
        progress(bytes, total);
    }
    file.sync_all()
        .map_err(|_| "The update download could not be saved.".to_owned())?;
    let actual = hex::encode(hash.finalize());
    if !actual.eq_ignore_ascii_case(expected_checksum) {
        let _ = fs::remove_file(destination);
        return Err("The update checksum could not be verified.".to_owned());
    }
    verify_signature(destination, signature.trim()).inspect_err(|_| {
        let _ = fs::remove_file(destination);
    })
}

fn client() -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent(USER_AGENT)
        .build()
        .map_err(|_| "The update service is unavailable.".to_owned())
}

fn fetch_text(client: &Client, url: &str) -> Result<String, String> {
    client
        .get(url)
        .send()
        .map_err(|_| "Unable to download the update verification data.".to_owned())?
        .error_for_status()
        .map_err(|_| "Unable to download the update verification data.".to_owned())?
        .text()
        .map_err(|_| "Unable to read the update verification data.".to_owned())
}

fn verify_signature(path: &Path, encoded_signature: &str) -> Result<(), String> {
    let public_key = option_env!("UPDATE_ED25519_PUBLIC_KEY")
        .filter(|key| !key.is_empty())
        .ok_or_else(|| "Update verification is not configured for this build.".to_owned())?;
    let public_key = BASE64
        .decode(public_key.trim())
        .map_err(|_| "The update verification key is invalid.".to_owned())?;
    let public_key: [u8; 32] = public_key
        .try_into()
        .map_err(|_| "The update verification key is invalid.".to_owned())?;
    let bytes = fs::read(path).map_err(|_| "The downloaded update cannot be read.".to_owned())?;
    verify_bytes(&public_key, &bytes, encoded_signature)
}

fn expected_checksum(checksum: &str) -> Result<&str, String> {
    checksum
        .split_whitespace()
        .next()
        .filter(|value| value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .ok_or_else(|| "The update checksum is invalid.".to_owned())
}

fn verify_bytes(
    public_key: &[u8; 32],
    bytes: &[u8],
    encoded_signature: &str,
) -> Result<(), String> {
    let signature = BASE64
        .decode(encoded_signature)
        .map_err(|_| "The update signature is invalid.".to_owned())?;
    let signature = Signature::from_slice(&signature)
        .map_err(|_| "The update signature is invalid.".to_owned())?;
    VerifyingKey::from_bytes(public_key)
        .map_err(|_| "The update verification key is invalid.".to_owned())?
        .verify(bytes, &signature)
        .map_err(|_| "The update signature could not be verified.".to_owned())
}

pub fn parse_version(value: &str) -> Result<Version, String> {
    Version::parse(value).map_err(|_| "The installed application version is invalid.".to_owned())
}

#[cfg(test)]
mod tests {
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

    use super::{expected_checksum, release_notes, verify_bytes};

    #[test]
    fn extracts_bilingual_release_notes() {
        let body = "<!-- pdfforge-notes:fr:start -->\nBonjour\n<!-- pdfforge-notes:fr:end -->\n<!-- pdfforge-notes:en:start -->\nHello\n<!-- pdfforge-notes:en:end -->";
        assert_eq!(release_notes(body), ("Bonjour".into(), "Hello".into()));
    }

    #[test]
    fn accepts_a_valid_checksum_sidecar() {
        assert_eq!(
            expected_checksum(
                "d2a4c9d5a98e6cb69bf2f99f3c8b434a040f12a0f7a514e1f7d5f4d1a42c8f94  PDFForge.exe"
            )
            .unwrap(),
            "d2a4c9d5a98e6cb69bf2f99f3c8b434a040f12a0f7a514e1f7d5f4d1a42c8f94"
        );
    }

    #[test]
    fn verifies_an_ed25519_signature() {
        let public_key =
            hex::decode("d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a")
                .unwrap()
                .try_into()
                .unwrap();
        let signature = BASE64.encode(
            hex::decode(concat!(
                "e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e06522490155",
                "5fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b"
            ))
            .unwrap(),
        );
        assert!(verify_bytes(&public_key, b"", &signature).is_ok());
    }
}
