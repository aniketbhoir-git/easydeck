//! Windows-only OS code. The crate is empty on other platforms so the workspace still builds.

#[cfg(windows)]
pub mod stats;
