//! FreedomWall - Window

use wry::webview::WebView;

#[cfg(target_os="windows")]
use tao::platform::windows::WindowExtWindows;

pub use super::platform::Window;
use super::data_manager::Wallpaper;


/// 背景ウィンドウの状態を変更したりするための構造体のトレイトです。
pub trait WindowTrait {
    /// コンストラクタ
    fn new(data: Wallpaper, webview: WebView, alpha: f64, target: String) -> Self;
    /// ウィンドウに透明度を設定します。
    fn set_transparent(&self, alpha: f64);
    /// ウィンドウの位置とサイズを変更します。
    fn set_rect(&self, x: f64, y: f64, width: f64, height: f64);
    /// Height,Width,x,yが入ったVectorからウィンドウの位置とサイズを変更します。
    fn set_rect_from_vec(&self, rect: &Vec<f64>);
    /// ウィンドウを一番前に一番前に表示し続けるかしないかを設定します。
    fn set_front(&mut self, front: bool);
    /// ウィンドウの順番を指定された順番の前に移動させます。
    fn set_order(&mut self, index: isize);
    /// クリックの貫通の有効/無効を設定します。
    fn set_click_through(&mut self, click_through: bool);
}