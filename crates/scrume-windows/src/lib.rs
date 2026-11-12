//! Narrow Win32 adapter; unsafe FFI is isolated in this crate's Windows modules.
#[cfg(windows)]
#[allow(unsafe_code)]
pub mod dpapi;
#[cfg(windows)]
#[allow(unsafe_code)]
pub mod pipe;
