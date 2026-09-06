use semver::Version;

use super::domain::{CheckResult, Platform, Release};

pub fn select_update(
    installed: &Version,
    platform: Platform,
    releases: impl IntoIterator<Item = Release>,
) -> CheckResult {
    let newest = releases
        .into_iter()
        .filter(|release| release.version > *installed)
        .max_by(|left, right| left.version.cmp(&right.version));

    match newest {
        None => CheckResult::UpToDate,
        Some(release) if platform.asset(&release).is_some() => CheckResult::Available {
            release: Box::new(release),
        },
        Some(release) => CheckResult::Unsupported {
            version: release.version,
        },
    }
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use super::*;
    use crate::update::domain::{Release, ReleaseAsset};

    fn asset() -> ReleaseAsset {
        ReleaseAsset {
            url: "https://example.test/file".into(),
            signature_url: "https://example.test/file.sig".into(),
            checksum_url: "https://example.test/file.sha256".into(),
            file_name: "file".into(),
        }
    }

    #[test]
    fn selects_the_newest_supported_release() {
        let update = select_update(
            &Version::new(1, 0, 0),
            Platform::LinuxX64,
            [
                Release {
                    version: Version::new(1, 1, 0),
                    notes_fr: String::new(),
                    notes_en: String::new(),
                    linux: Some(asset()),
                    windows: None,
                },
                Release {
                    version: Version::new(1, 2, 0),
                    notes_fr: String::new(),
                    notes_en: String::new(),
                    linux: Some(asset()),
                    windows: None,
                },
            ],
        );
        assert!(
            matches!(update, CheckResult::Available { release } if release.version == Version::new(1, 2, 0))
        );
    }
}
