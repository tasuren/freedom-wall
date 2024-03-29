use std::{
    ffi::{c_void, CString},
    mem::transmute,
    ptr::null,
};

use wry::{
    application::{
        dpi::{LogicalPosition, LogicalSize},
        platform::macos::WindowExtMacOS,
    },
    webview::WebView,
};

use objc::{
    msg_send,
    runtime::{Object, NO, YES},
    sel, sel_impl,
};

use core_foundation::{
    array::{CFArrayGetCount, CFArrayGetValueAtIndex},
    base::CFIndex,
    bundle::CFBundle,
    dictionary::{CFDictionaryGetValueIfPresent, CFDictionaryRef},
    number::{kCFNumberIntType, CFNumberGetValue, CFNumberRef},
    string::{
        kCFStringEncodingUTF8, CFStringCreateWithCString, CFStringGetLength, CFStringRef, UniChar,
    },
};
use core_graphics::display::{
    kCGNullWindowID, kCGWindowListExcludeDesktopElements, kCGWindowListOptionOnScreenOnly,
    CGWindowListCopyWindowInfo,
};

use super::super::{
    data_manager::{Shift, Wallpaper},
    platform::{ExtendedRects, Titles},
    window::WindowTrait,
};

/// Bundleのパスを取得します。
pub fn get_bundle_path() -> String {
    CFBundle::main_bundle()
        .path()
        .unwrap()
        .display()
        .to_string()
}

pub struct Window {
    pub webview: WebView,
    ns_window: *const Object,
    pub wallpaper: Wallpaper,
    pub target: String,
    before_front: bool,
    pub id: usize,
}

/// 渡された&strのCStringを作る。
fn get_cstring(text: &str) -> CString {
    CString::new(text).unwrap()
}

/// 渡されたCStringをCFStringのポインタにする。
fn get_cfstring_pointer(cstring: CString) -> *const c_void {
    unsafe {
        transmute(CFStringCreateWithCString(
            null(),
            cstring.as_ptr(),
            kCFStringEncodingUTF8,
        ))
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
        return None;
    };
    Some(value)
}

/// 渡された&strで渡されたCFDictionaryから値を取り出します。
fn get_cfdictionary_value_from_str(data: CFDictionaryRef, key: &str) -> Option<*const c_void> {
    get_cfdictionary_value(data, get_cfstring_pointer_from_str(key))
}

/// 渡されたCFNumberのポインタから整数を取り出します。
fn get_cfnumber(number: *const c_void) -> Option<i32> {
    let mut value: i32 = 0;
    unsafe {
        if CFNumberGetValue(
            number as CFNumberRef,
            kCFNumberIntType,
            &mut value as *mut i32 as *mut c_void,
        ) {
            Some(value)
        } else {
            None
        }
    }
}

extern "C" {
    /// 渡された文字列から指定された位置にある文字を取り出します。
    fn CFStringGetCharacterAtIndex(theString: CFStringRef, idx: CFIndex) -> UniChar;
}

/// CFStringをStringにします。
fn cfstring2string(text: CFStringRef) -> String {
    let mut result: Vec<u16> = Vec::new();
    for i in 0..unsafe { CFStringGetLength(text) } {
        result.push(unsafe { CFStringGetCharacterAtIndex(text, i) });
    }
    String::from_utf16(&result).unwrap()
}

/// 存在する全てのウィンドウのタイトルや位置そしてサイズ等を取得します。
/// 二番目のVectorの三番目のisizeの値はウィンドウ番号です。(背景ウィンドウの順序変更に使用する)
pub fn get_windows() -> (Titles, ExtendedRects) {
    let (mut windows_name, mut windows_rect): (_, ExtendedRects) = (Vec::new(), Vec::new());

    let windows = unsafe {
        CGWindowListCopyWindowInfo(
            kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements,
            kCGNullWindowID,
        )
    };
    let mut next_main = false;

    for index in 0..unsafe { CFArrayGetCount(windows) } {
        let data = unsafe { CFArrayGetValueAtIndex(windows, index) as CFDictionaryRef };

        let title = cfstring2string(
            match get_cfdictionary_value_from_str(data, "kCGWindowOwnerName") {
                Some(value) => value as CFStringRef,
                _ => continue,
            },
        );
        let layer = get_cfnumber(
            get_cfdictionary_value_from_str(data, "kCGWindowLayer")
                .expect("CFDictionaryからkCGWindowLayerの値の取り出しに失敗しました。"),
        )
        .expect("CFNumberの値の取り出しに失敗しました。");
        if &title == "Dock" && layer != 0 {
            next_main = true;
            continue;
        };
        if layer != 0 {
            continue;
        }; // 一般の人がウィンドウだと思うウィンドウ以外は除外する。(タスクトレイアイコン等)

        if title.is_empty() {
            if next_main {
                next_main = false;
            };
        } else {
            if index == 0 {
                continue;
            };
            let before_index = if windows_name.is_empty() {
                0
            } else {
                windows_name.len() - 1
            };
            let same_before = !windows_name.is_empty() && windows_name[before_index] == title;

            // サイズを取得する。
            let rect = match get_cfdictionary_value_from_str(data, "kCGWindowBounds") {
                Some(value) => value as CFDictionaryRef,
                _ => continue,
            };
            let mut tentative = [0 as i32; 4];

            let mut update = false;
            for (i, key) in ["Width", "Height", "X", "Y"].iter().enumerate() {
                tentative[i] = get_cfnumber(
                    get_cfdictionary_value_from_str(rect, key)
                        .expect("CFDictionaryからkCGWindowBoundsの値を取り出すのに失敗しました。"),
                )
                .expect("CFNumberの値の取り出しに失敗しました。");
                if !windows_rect.is_empty()
                    || !same_before
                    || windows_rect[before_index].0[i] < tentative[i]
                {
                    // 一番サイズのでかいウィンドウが対象になるように前取得したやつをチェックする。
                    update = true;
                };
            }

            if update {
                if same_before {
                    windows_name.pop();
                    windows_rect.pop();
                };
                windows_rect.push((
                    tentative,
                    next_main,
                    get_cfnumber(
                        get_cfdictionary_value_from_str(data, "kCGWindowNumber").expect(
                            "CFDictionaryからkCGWindowNumberの値を取り出すのに失敗しました。",
                        ),
                    )
                    .expect("CFNumberの値の取り出しに失敗しました。") as isize,
                ));
                if next_main {
                    next_main = false;
                };
                windows_name.push(title);
            };
        };
    }
    (windows_name, windows_rect)
}

impl WindowTrait for Window {
    fn new(wallpaper: Wallpaper, webview: WebView, id: usize, target: String) -> Self {
        let ns_window = webview.window().ns_window() as *const Object;
        let window = Self {
            webview: webview,
            ns_window: ns_window,
            id: id,
            wallpaper: wallpaper,
            target: target,
            before_front: false,
        };
        window
    }

    fn set_transparent(&self, alpha: f64) {
        unsafe {
            let _: () = msg_send![self.ns_window, setAlphaValue: alpha];
        };
    }

    fn set_rect(&self, shift: &Shift, width: i32, height: i32, x: i32, y: i32) {
        let window = self.webview.window();
        // 背景ウィンドウのサイズを変える。
        window.set_inner_size::<LogicalSize<i32>>(LogicalSize {
            width: width + shift.right,
            height: height + shift.down,
        });
        // 背景ウィンドウの位置を移動する。
        window.set_outer_position::<LogicalPosition<i32>>(LogicalPosition {
            x: x + shift.left,
            y: y + shift.up,
        });
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
}
