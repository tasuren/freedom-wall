//! FreedomWall - Impl for MacOS

use std::{
    ffi::{ CString, c_void },
    ptr::null,
    mem::transmute
};

use wry::{
    webview::WebView,
    application::{
        platform::macos::WindowExtMacOS,
        dpi::{ LogicalPosition, LogicalSize }
    }
};

use objc::{
    msg_send, sel, sel_impl,
    runtime::{ Object, YES, NO }
};

use core_graphics::display::{
    CGWindowListCopyWindowInfo, kCGWindowListOptionOnScreenOnly,
    kCGWindowListExcludeDesktopElements, kCGNullWindowID
};
use core_foundation::{
    dictionary::{ CFDictionaryRef, CFDictionaryGetValueIfPresent },
    array::{ CFArrayGetCount, CFArrayGetValueAtIndex },
    number::{ CFNumberRef, CFNumberGetValue, kCFNumberFloat32Type },
    string::{
        CFStringRef, CFStringCreateWithCString,
        CFStringGetLength, UniChar, kCFStringEncodingUTF8
    },
    base::CFIndex
};

use super::super::WallpaperTrait;


pub struct Wallpaper {
    pub webview: WebView,
    pub ns_window: *const Object,
    pub now_click_through: bool
}


/// 渡された&strのCStringを作る。
fn get_cstring(text: &str) -> CString {
    CString::new(text).unwrap()
}


/// 渡されたCStringをCFStringのポインタにする。
fn get_cfstring_pointer(cstring: CString) -> *const c_void {
    unsafe {
        transmute(
            CFStringCreateWithCString(
                null(), cstring.as_ptr(), kCFStringEncodingUTF8
            )
        )
    }
}


/// 渡された&strをCFStringのポインタにする。
fn get_cfstring_pointer_from_str(text: &str) -> *const c_void {
    get_cfstring_pointer(get_cstring(text))
}


/// 渡されたキーに対応する値を渡されたCFDictionaryから取り出します。
fn get_cfdictionary_value(data: CFDictionaryRef, key: *const c_void) -> Option<*const c_void> {
    let mut value: *const c_void = null();
    if unsafe { CFDictionaryGetValueIfPresent(data, key, &mut value) == 0 } {
        return None
    };
    Some(value)
}


/// 渡された&strで渡されたCFDictionaryから値を取り出します。
fn get_cfdictionary_value_from_str(data: CFDictionaryRef, key: &str) -> Option<*const c_void> {
    get_cfdictionary_value(data, get_cfstring_pointer_from_str(key))
}


/// 渡されたCFNumberのポインタから値を取り出します。
fn get_cfnumber(number: *const c_void) -> Option<f32> {
    let mut value: f32 = 0.0;
    unsafe {
        if CFNumberGetValue(
            number as CFNumberRef, kCFNumberFloat32Type,
            &mut value as *mut f32 as *mut c_void
        ) { Some(value) } else { None }
    }
}


extern {
    /// 渡された文字列から指定された位置にある文字を取り出します。
    fn CFStringGetCharacterAtIndex(theString: CFStringRef, idx: CFIndex) -> UniChar;
}


/// CFStringをStringにします。
fn cfstring2str(text: CFStringRef) -> String {
    let mut result: Vec<u16> = Vec::new();
    for i in 0..unsafe { CFStringGetLength(text) } {
        result.push(unsafe {
            CFStringGetCharacterAtIndex(text, i)
        });
    };
    String::from_utf16(&result).unwrap()
}


/// 渡された文字列が名前に含まれるウィンドウのサイズを取得します。
fn get_window(name: String) -> Vec<f64> {
    let mut result: Vec<f64> = Vec::new();
    let windows = unsafe {
        CGWindowListCopyWindowInfo(
            kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements,
            kCGNullWindowID
        )
    };
    for i in 0..unsafe { CFArrayGetCount(windows) } {
        let data = unsafe { CFArrayGetValueAtIndex(windows, i) as CFDictionaryRef };

        let title = cfstring2str(
            match get_cfdictionary_value_from_str(
                data, "kCGWindowOwnerName"
            ) { Some(value) => value as CFStringRef, _ => continue }
        );

        if !title.is_empty() && title.contains(&name) {
            // もしタイトルに指定された名前が含まれているのなら。
            let rect = match get_cfdictionary_value_from_str(
                data, "kCGWindowBounds"
            ) { Some(value) => value as CFDictionaryRef, _ => continue };
            let mut tentative: Vec<f64> = Vec::new();
            let mut update = false;
            for (i, key) in ["Height", "Width", "X", "Y"].iter().enumerate() {
                match get_cfnumber(
                    match get_cfdictionary_value_from_str(rect, key) {
                        Some(value) => value, _ => { update = false; break; }
                    }
                ) {
                    Some(value) => {
                        tentative.push(value as f64);
                        if result.is_empty() || result[i] < tentative[i] {
                            update = true;
                        };
                    },
                    _ => ()
                }
            };
            if update {
                result = tentative;
            };
        };
    };
    result
}


impl WallpaperTrait for Wallpaper {
    fn new(webview: WebView) -> Self {
        let ns_window = webview.window().ns_window() as *const Object;
        let mut wallpaper = Wallpaper {
            webview: webview, ns_window: ns_window,
            now_click_through: false
        };
        wallpaper.set_transparent(0.2);
        wallpaper.toggle_click_through();
        wallpaper
    }

    fn set_transparent(&self, alpha: f64) {
        unsafe {
            let _: () = msg_send![self.ns_window, setAlphaValue: alpha];
        };
    }

    fn set_rect(&self, x: f64, y: f64, width: f64, height: f64) {
        let window = self.webview.window();
        // 背景ウィンドウのサイズを変える。
        window.set_inner_size::<LogicalSize<f64>>(
            (LogicalSize {width: width, height: height}).into()
        );
        // 背景ウィンドウの位置を移動する。
        window.set_outer_position::<LogicalPosition<f64>>(
            (LogicalPosition { x: x, y: y }).into()
        );
    }

    fn process_position(&self) {
        // 対象のウィンドウのサイズと座標を取得する。
        let rect = get_window("Discord".to_string());
        if rect.len() >= 4 {
            // ウィンドウの位置を移動する。
            self.set_rect(rect[2], rect[3], rect[1], rect[0]);
            // println!("{:?}", rect);
        };
    }

    fn toggle_click_through(&mut self) {
        self.now_click_through = !self.now_click_through;
        unsafe {
            // クリックイベントがウィンドウに来ないようにする。
            let _: () = msg_send![
                self.ns_window,
                setIgnoresMouseEvents: if self.now_click_through { YES } else { NO }
            ];
            // ウィンドウレベルをオーバーレイに設定する。
            let _: () = msg_send![self.ns_window, setLevel: 15];
        };
    }
}