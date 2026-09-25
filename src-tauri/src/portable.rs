use std::path::{Path, PathBuf};

pub use localstream_portable_payload::{
    activate, create_portable_executable, PortableBuildSummary, PortableError, PortableResource,
};

pub fn activate_from_current_executable(app_data: &Path) -> Result<Option<PathBuf>, PortableError> {
    let executable = std::env::current_exe().map_err(|source| PortableError::Io {
        context: "resolving the current executable",
        source,
    })?;
    activate(&executable, app_data)
}
