//! FreedomWall - Impl for MacOS

use std::{
    ffi::{ CString, c_void },
    ptr::null,
    mem::transmute
};

use wry::{
    webview::WebView,
    application::platform::macos::WindowExtMacOS
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
    number::{
        CFNumberRef, CFNumberGetValue, kCFNumberFloat64Type,
        kCFNumberIntType
    },
    string::{
        CFStringRef, CFStringCreateWithCString,
        CFStringGetLength, UniChar, kCFStringEncodingUTF8
    },
    base::CFIndex
};

use super::super::{
    data_manager::Wallpaper, window::WindowTrait,
    platform::{ set_front, Titles, ExtendedRects }
};


pub struct Window {
    pub webview: WebView,
    ns_window: *const Object,
    pub wallpaper: Wallpaper,
    pub target: String,
    before_front: bool
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


/// 渡されたCFNumberのポインタからf64の値を取り出します。
fn get_cfnumber_as_f64(number: *const c_void) -> Option<f64> {
    let mut value: f64 = 0.0;
    unsafe {
        if CFNumberGetValue(
            number as CFNumberRef, kCFNumberFloat64Type,
            &mut value as *mut f64 as *mut c_void
        ) { Some(value) } else { None }
    }
}


/// 渡されたCFNumberのポインタから整数を取り出します。
fn get_cfnumber(number: *const c_void) -> Option<isize> {
    let mut value: i32 = 0;
    unsafe {
        if CFNumberGetValue(
            number as CFNumberRef, kCFNumberIntType,
            &mut value as *mut i32 as *mut c_void
        ) { Some(value) } else { None }
    }
}


extern {
    /// 渡された文字列から指定された位置にある文字を取り出します。
    fn CFStringGetCharacterAtIndex(theString: CFStringRef, idx: CFIndex) -> UniChar;
}


/// CFStringをStringにします。
fn cfstring2string(text: CFStringRef) -> String {
    let mut result: Vec<u16> = Vec::new();
    for i in 0..unsafe { CFStringGetLength(text) } {
        result.push(unsafe {
            CFStringGetCharacterAtIndex(text, i)
        });
    };
    String::from_utf16(&result).unwrap()
}


/// 存在する全てのウィンドウのタイトルや位置そしてサイズ等を取得します。
/// 二番目のVectorの三番目のisizeの値はウィンドウ番号です。(背景ウィンドウの順序変更に使用する)
pub fn get_windows() -> (Titles, ExtendedRects) {
    let (mut windows_name, mut windows_rects) = (Vec::new(), Vec::new());

    let windows = unsafe {
        CGWindowListCopyWindowInfo(
            kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements,
            kCGNullWindowID
        )
    };
    let mut next_main = false;
    for index in 0..unsafe { CFArrayGetCount(windows) } {
        let data = unsafe { CFArrayGetValueAtIndex(windows, index) as CFDictionaryRef };

        let title = cfstring2string(
            match get_cfdictionary_value_from_str(
                data, "kCGWindowOwnerName"
            ) { Some(value) => value as CFStringRef, _ => continue }
        );
        let layer = get_cfnumber(
            get_cfdictionary_value_from_str(data, "kCGWindowLayer")
                .expect("CFDictionaryからkCGWindowLayerの値の取り出しに失敗しました。")
        ).expect("CFNumberの値の取り出しに失敗しました。");
        if &title == "Dock" && layer != 0 { next_main = true; continue; };
        if layer != 0 { continue; }; // 一般の人がウィンドウだと思うウィンドウ以外は除外する。(タスクトレイアイコン等)

        if title.is_empty() {
            if next_main { next_main = false; };
        } else {
            if index == 0 { continue; };
            let before_index = if windows_name.is_empty() { 0 } else { windows_name.len() - 1 };
            let same_before = !windows_name.is_empty() && windows_name[before_index] == title;

            // サイズを取得する。
            let rect = match get_cfdictionary_value_from_str(
                data, "kCGWindowBounds"
            ) { Some(value) => value as CFDictionaryRef, _ => continue };
            let mut tentative: Vec<i32> = Vec::new();

            let mut update = false;
            for (i, key) in ["Width", "Height", "X", "Y"].iter().enumerate() {
                tentative.push(
                    get_cfnumber(
                        get_cfdictionary_value_from_str(rect, key)
                            .expect("CFDictionaryからkCGWindowBoundsの値を取り出すのに失敗しました。")
                    ).expect("CFNumberの値の取り出しに失敗しました。")
                );
                if !windows_rect.is_empty() || !same_before
                        || windows_rect[before_index].0[i] < tentative[i] {
                    // 一番サイズのでかいウィンドウが対象になるように前取得したやつをチェックする。
                    update = true;
                };
            };

            if update {
                if same_before { windows_name.pop(); windows_rect.pop(); };
                windows_rect.push((tentative, next_main, get_cfnumber(
                    get_cfdictionary_value_from_str(
                        data, "kCGWindowNumber"
                    ).expect("CFDictionaryからkCGWindowNumberの値を取り出すのに失敗しました。")
                ).expect("CFNumberの値の取り出しに失敗しました。")));
                if next_main && !title.contains("FreedomWall") { next_main = false; };
                windows_name.push(title);
            };
        };
    };
    (windows_name, windows_rect)
}


impl WindowTrait for Window {
    fn new(wallpaper: Wallpaper, webview: WebView, alpha: f64, target: String) -> Self {
        let ns_window = webview.window().ns_window() as *const Object;
        let window = Self {
            webview: webview, ns_window: ns_window,
            wallpaper: wallpaper, target: target, before_front: false
        };
        window.set_transparent(alpha);
        window
    }

    fn set_transparent(&self, alpha: f64) {
        unsafe {
            let _: () = msg_send![self.ns_window, setAlphaValue: alpha];
        };
    }

    fn set_front(&mut self, front: bool) {
        // 最前列のウィンドウが切り替わった際のみ動作を行う。
        if front != self.before_front {
            self.before_front = front;
            println!("Front changed [{}]: {}", self.target, front);
            self.webview.window().set_always_on_top(front);
        };
    }

    fn set_order(&mut self, target: isize) {
        // 最前列にオーバーレイ表示されている背景の場合はする必要がないのでifでパスする。
        if !self.before_front {
            unsafe {
                let _: () = msg_send![self.ns_window, orderWindow: 1 as isize relativeTo: target];
            };
        };
    }

    fn set_click_through(&mut self, toggle: bool) {
        unsafe {
            let _: () = msg_send![
                self.ns_window, setIgnoresMouseEvents: if toggle { YES } else { NO }
            ];
        };
    }

    fn get_webview(&self) -> &WebView { &self.webview }
}