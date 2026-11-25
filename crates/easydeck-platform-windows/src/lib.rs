//! Windows-only OS code. The crate is empty on other platforms so the workspace still builds.

#[cfg(windows)]
pub mod audio;
#[cfg(windows)]
pub mod media;
#[cfg(windows)]
pub mod stats;
