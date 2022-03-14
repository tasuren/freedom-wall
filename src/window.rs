//! FreedomWall - Window

use std::collections::HashMap;

use wry::webview::WebView;

#[cfg(target_os="windows")]
use tao::platform::windows::WindowExtWindows;

pub use super::platform::Window;
use super::data_manager::Wallpaper;


/// 背景ウィンドウの状態を変更したりするための構造体のトレイトです。
pub trait WindowTrait {
    /// コンストラクタ
    #[cfg(target_os="macos")]
    fn new(webview: WebView) -> Self;
    /// ウィンドウに透明度を設定します。
    fn set_transparent(&self, alpha: f64);
    /// ウィンドウの位置とサイズを変更します。
    fn set_rect(&self, x: f64, y: f64, width: f64, height: f64);
    /// Height,Width,x,yが入ったVectorからウィンドウの位置とサイズを変更します。
    fn set_rect_from_vec(&self, rect: Vec<f64>);
    /// クリック貫通の有効/無効を切り替えます。
    fn toggle_click_through(&mut self);
}


/// ウィンドウを管理するための構造体です。
pub struct Manager {
    windows: HashMap<Wallpaper, Window>
}


impl Manager {
    fn new() -> Self {
        Manager { windows: HashMap::new() }
    }

    fn add(&mut self, data: Wallpaper) {
        self.windows.push(Window::new())
    }
}