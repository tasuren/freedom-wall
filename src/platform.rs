//! FreedomWall - Platform

#[cfg(target_os="macos")]
pub mod macos;
#[cfg(target_os="macos")]
pub use macos::{ Window, get_windows };