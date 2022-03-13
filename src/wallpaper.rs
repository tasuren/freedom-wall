//! FreedomWall - WindowManager

use wry::webview::WebView;

#[cfg(target_os="windows")]
use tao::platform::windows::WindowExtWindows;

pub use super::platform::Wallpaper;


/// 背景ウィンドウの状態を変更したりするためのstructのTraitです。
pub trait WallpaperTrait {
    /// コンストラクタ
    #[cfg(target_os="macos")]
    fn new(webview: WebView) -> Self;
    /// ウィンドウに透明度を設定します。
    fn set_transparent(&self, alpha: f64);
    /// ウィンドウの位置とサイズを変更します。
    fn set_rect(&self, x: f64, y: f64, width: f64, height: f64);
    /// ウィンドウの位置をターゲットウィンドウに移動します。
    fn process_position(&self);
    /// クリック貫通の有効/無効を切り替えます。
    fn toggle_click_through(&mut self);
}