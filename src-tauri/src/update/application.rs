use semver::Version;

use super::domain::{CheckResult, Platform, Release, ReleaseNotes};

pub fn select_update(
    installed: &Version,
    platform: Platform,
    releases: impl IntoIterator<Item = Release>,
) -> CheckResult {
    let mut missing: Vec<_> = releases
        .into_iter()
        .filter(|release| release.version > *installed)
        .collect();
    missing.sort_by(|left, right| right.version.cmp(&left.version));
    let releases = missing
        .iter()
        .map(|release| ReleaseNotes {
            version: release.version.clone(),
            notes_fr: release.notes_fr.clone(),
            notes_en: release.notes_en.clone(),
        })
        .collect();

    match missing.into_iter().next() {
        None => CheckResult::UpToDate,
        Some(release) if platform.asset(&release).is_some() => CheckResult::Available {
            release: Box::new(release),
            releases,
        },
        Some(release) => CheckResult::Unsupported {
            version: release.version,
            releases,
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

    fn release(minor: u64, linux: bool) -> Release {
        Release {
            version: Version::new(1, minor, 0),
            notes_fr: format!("Notes françaises 1.{minor}.0"),
            notes_en: format!("English notes 1.{minor}.0"),
            linux: linux.then(asset),
            windows: Some(asset()),
        }
    }

    #[test]
    fn includes_every_missing_release_in_semantic_descending_order() {
        let result = select_update(
            &Version::new(1, 1, 0),
            Platform::LinuxX64,
            [
                release(2, true),
                release(0, true),
                release(10, true),
                release(1, true),
                release(3, false),
            ],
        );
        let CheckResult::Available { release, releases } = result else {
            panic!("the latest release should be available");
        };
        assert_eq!(release.version, Version::new(1, 10, 0));
        assert_eq!(release.linux, Some(asset()));
        assert_eq!(
            releases,
            [10, 3, 2].map(|minor| ReleaseNotes {
                version: Version::new(1, minor, 0),
                notes_fr: format!("Notes françaises 1.{minor}.0"),
                notes_en: format!("English notes 1.{minor}.0"),
            })
        );
    }

    #[test]
    fn keeps_missing_notes_when_the_latest_release_is_unsupported() {
        let result = select_update(
            &Version::new(1, 0, 0),
            Platform::LinuxX64,
            [release(1, true), release(2, false)],
        );
        let CheckResult::Unsupported { version, releases } = result else {
            panic!("an older supported release must not replace the latest target");
        };
        assert_eq!(version, Version::new(1, 2, 0));
        assert_eq!(
            releases
                .iter()
                .map(|release| &release.version)
                .collect::<Vec<_>>(),
            [&Version::new(1, 2, 0), &Version::new(1, 1, 0)]
        );
    }

    #[test]
    fn reports_up_to_date_without_newer_releases() {
        for releases in [vec![], vec![release(0, true), release(1, true)]] {
            assert_eq!(
                select_update(&Version::new(1, 1, 0), Platform::LinuxX64, releases),
                CheckResult::UpToDate
            );
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
            matches!(update, CheckResult::Available { release, .. } if release.version == Version::new(1, 2, 0))
        );
    }
}
