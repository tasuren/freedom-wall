//! FreedomWall - Platform

#[cfg(target_os="macos")]
pub mod macos;
#[cfg(target_os="macos")]
pub use macos::{ Window, get_windows };

#[cfg(target_os="windows")]
pub mod windows;
#[cfg(target_os="windows")]
pub use windows::{ Window, get_windows };

pub mod all;
pub use all::{ Titles, Rects, ExtendedRects };