use wry::webview::WebView;

pub use super::platform::Window;
use super::{
    data_manager::{Shift, Wallpaper},
    platform::Rects,
};

/// 背景ウィンドウの状態を変更したりするための構造体のトレイトです。
pub trait WindowTrait {
    /// コンストラクタ
    fn new(data: Wallpaper, webview: WebView, id: usize, target: String) -> Self;
    /// ウィンドウに透明度を設定します。
    fn set_transparent(&self, alpha: f64);
    /// ウィンドウの位置とサイズを変更します。
    /// Windowsの場合は`set_rect_from_vec`に書いてあるものが順番に渡されます。
    fn set_rect(&self, shift: &Shift, width: i32, height: i32, x: i32, y: i32);
    /// Width,Height,x,yが入ったVectorからウィンドウの位置とサイズを変更します。
    /// Windowsの場合はx(left),y(top),x(right),y(bottom)の左上と右下の位置となります。
    fn set_rect_from_vec(&self, shift: &Shift, rect: &Rects) {
        self.set_rect(shift, rect[0], rect[1], rect[2], rect[3]);
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
