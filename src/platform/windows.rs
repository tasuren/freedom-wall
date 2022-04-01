//! FreedomWall - Impl for Windows

use std::mem::size_of;

use wry::{
    webview::WebView,
    application::{
        platform::windows::WindowExtWindows
    }
};

use windows_sys::Win32::{
    Foundation::{ HWND, LPARAM, BOOL, RECT },
    UI::WindowsAndMessaging::{
        EnumWindows, SetLayeredWindowAttributes, SetWindowPos,
        SetWindowLongA, GetWindowTextW, GetWindowRect,
        GetForegroundWindow, MoveWindow,
        AdjustWindowRectEx, GetWindowLongW
    },
    Graphics::Dwm::{
        DwmGetWindowAttribute,
        DWMWA_EXTENDED_FRAME_BOUNDS
    }
};

use super::super::{
    data_manager::Wallpaper, window::WindowTrait,
    platform::{ Titles, ExtendedRects }
};


static mut DATA: (Titles, ExtendedRects) = (Vec::new(), Vec::new());
static mut BEFORE: HWND = 0;


/// `get_windows`内の`EnumWindows`に渡す関数です。
unsafe extern "system" fn lpenumfunc(hwnd: HWND, _: LPARAM) -> BOOL {
    // ウィンドウのタイトルを取得する。
    let mut raw: [u16; 512] = [0; 512];
    let length = GetWindowTextW(hwnd, &mut raw as _, 512);
    DATA.0.push(String::from_utf16_lossy(&raw[..length as usize]).to_string());
    // ウィンドウのサイズ等を取得する。
    let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    DwmGetWindowAttribute(
        hwnd, DWMWA_EXTENDED_FRAME_BOUNDS,
        &mut rect as *mut RECT as *mut _,
        size_of::<RECT>() as u32
    );

    DATA.1.push((
        vec![rect.left, rect.top, rect.right, rect.bottom],
        GetForegroundWindow() == hwnd, BEFORE
    ));
    if hwnd != 0 { BEFORE = hwnd; };

    true.into()
}


/// 全てのウィンドウのタイトルやサイズ等を取得します。
/// 二番目のVectorの三番目のisizeの値は、そのウィンドウの前にあるウィンドウのHWNDです。
/// 一番前のウィンドウの場合は前がいないので代わりに0となります。(背景ウィンドウの順序変更に使う)
pub fn get_windows() -> (Titles, ExtendedRects) {
    unsafe {
        DATA = (Vec::new(), Vec::new());
        assert_eq!(EnumWindows(Some(lpenumfunc), 0), 1);
        BEFORE = 0;
        DATA.clone()
    }
}


pub struct Window {
    pub webview: WebView,
    pub wallpaper: Wallpaper,
    pub target: String,
    front: bool,
    hwnd: HWND,
    first: bool
}


/// SetWindowPosでウィンドウのZ順の位置を変更します。
fn set_order(hwnd: HWND, target: isize, more: u32) {
    unsafe {
        SetWindowPos(
            hwnd, target, 0, 0, 0, 0,
            0x0010 | 0x0001 | 0x0002 | more
        );
    };
}


impl WindowTrait for Window {
    fn new(data: Wallpaper, webview: WebView, target: String) -> Self {
        let mut window = Self {
            hwnd: webview.window().hwnd() as _, webview: webview,
            wallpaper: data, target: target, front: false, first: true
        };
        window.webview.window().set_skip_taskbar(true);
        window
    }

    fn set_transparent(&self, alpha: f64) {
        assert_eq!(unsafe {
            SetLayeredWindowAttributes(self.hwnd, 0, (255.0 * alpha) as u8, 0x00000002)
        }, 1);
    }

    fn set_rect(&self, left: i32, top: i32, right: i32, bottom: i32) {
        unsafe {
            // Rectを調整する。
            let rect = RECT {
                left: left, top: top,
                right: right, bottom: bottom
            };
            // ウィンドウの位置等を更新する。
            MoveWindow(
                self.hwnd, rect.left, rect.top,
                (rect.right - rect.left).abs(),
                (rect.bottom - rect.top).abs(),
                1
            );
        };
        self.webview.resize().unwrap();
    }

    fn set_front(&mut self, front: bool) {
        if front != self.front {
            self.front = front;
            println!("Front changed [{}]: {}", self.target, front);
            if self.first && front {
                self.webview.window().set_focus();
                self.first = false;
            };
            set_order(self.hwnd, if front { -1 } else { -2 }, 0);
        };
    }

    fn set_order(&mut self, target: isize) {
        if !self.front && target != 0 {
            set_order(self.hwnd, target, 0);
        };
    }

    fn set_click_through(&mut self, toggle: bool) {
        assert!(unsafe {
            SetWindowLongA(
                self.hwnd, -20,
                if toggle {
                    0x00080000 | 0x00000020
                } else { 0 }
            )
        } != 0);
    }
}