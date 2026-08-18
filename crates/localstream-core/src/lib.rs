use serde::Serialize;

pub mod media;

pub use media::{LibraryScan, LibraryScanError};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub local_first: bool,
}

#[derive(Debug, Default)]
pub struct LocalStreamCore;

impl LocalStreamCore {
    #[must_use]
    pub const fn app_info(&self) -> AppInfo {
        AppInfo {
            name: "LocalStream",
            version: env!("CARGO_PKG_VERSION"),
            local_first: true,
        }
    }

    pub fn scan_library(
        &self,
        approved_directory: impl AsRef<std::path::Path>,
    ) -> Result<LibraryScan, LibraryScanError> {
        media::scan_approved_directory(approved_directory.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::LocalStreamCore;

    #[test]
    fn exposes_local_first_application_identity() {
        let info = LocalStreamCore.app_info();

        assert_eq!(info.name, "LocalStream");
        assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
        assert!(info.local_first);
    }
}
