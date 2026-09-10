use semver::Version;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReleaseAsset {
    pub url: String,
    pub signature_url: String,
    pub checksum_url: String,
    pub file_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Release {
    pub version: Version,
    pub notes_fr: String,
    pub notes_en: String,
    pub linux: Option<ReleaseAsset>,
    pub windows: Option<ReleaseAsset>,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Platform {
    LinuxX64,
    WindowsX64,
}

impl Platform {
    pub fn current() -> Result<Self, String> {
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        {
            return Ok(Self::LinuxX64);
        }
        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        {
            return Ok(Self::WindowsX64);
        }
        #[allow(unreachable_code)]
        Err("Updates are not available on this system.".to_owned())
    }

    pub fn asset<'a>(&self, release: &'a Release) -> Option<&'a ReleaseAsset> {
        match self {
            Self::LinuxX64 => release.linux.as_ref(),
            Self::WindowsX64 => release.windows.as_ref(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReleaseNotes {
    pub version: Version,
    pub notes_fr: String,
    pub notes_en: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckResult {
    UpToDate,
    Unsupported {
        version: Version,
        releases: Vec<ReleaseNotes>,
    },
    Available {
        release: Box<Release>,
        releases: Vec<ReleaseNotes>,
    },
}

pub fn stable_version(tag: &str) -> Option<Version> {
    let value = tag.strip_prefix('v')?;
    let version = Version::parse(value).ok()?;
    (version.pre.is_empty()
        && version.build.is_empty()
        && tag == format!("v{}.{}.{}", version.major, version.minor, version.patch))
    .then_some(version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_plain_stable_v_tags() {
        assert_eq!(stable_version("v1.2.3"), Some(Version::new(1, 2, 3)));
        assert_eq!(stable_version("1.2.3"), None);
        assert_eq!(stable_version("v1.2.3-beta.1"), None);
        assert_eq!(stable_version("v1.2"), None);
    }
}
