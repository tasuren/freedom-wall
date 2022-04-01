//! FreedomWall - Window

use wry::webview::WebView;

pub use super::platform::Window;
use super::{ platform::Rects, data_manager::Wallpaper };


/// 背景ウィンドウの状態を変更したりするための構造体のトレイトです。
pub trait WindowTrait {
    /// コンストラクタ
    fn new(data: Wallpaper, webview: WebView, target: String) -> Self;
    /// ウィンドウに透明度を設定します。
    fn set_transparent(&self, alpha: f64);
    /// ウィンドウの位置とサイズを変更します。
    fn set_rect(&self, width: i32, height: i32, x: i32, y: i32);
    /// Width,Height,x,yが入ったVectorからウィンドウの位置とサイズを変更します。
    fn set_rect_from_vec(&self, rect: &Rects) {
        self.set_rect(rect[0], rect[1], rect[2], rect[3]);
    }
    /// ウィンドウを一番前に一番前に表示し続けるかしないかを設定します。
    fn set_front(&mut self, front: bool);
    /// ウィンドウの順番を指定された順番の前に移動させます。
    /// MacOSの場合はindexにはウィンドウ番号が渡されます。
    /// Windowsの場合は拝啓対象のウィンドウの前にあるウィンドウのHWNDがindexに渡されます。
    fn set_order(&mut self, index: isize);
    /// クリックの貫通の有効/無効を設定します。
    fn set_click_through(&mut self, click_through: bool);
}