#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
pub use macos::{get_windows, Window};

#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use windows::{get_windows, Window};

pub mod all;
pub use all::{ExtendedRects, Rects, Titles};
