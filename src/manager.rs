//! FreedomWall - Manager

use std::{
    collections::HashMap, rc::Rc,
    fs::{ canonicalize, read }
};

use wry::{
    application::{
        event_loop::{ EventLoop, EventLoopWindowTarget },
        window::WindowBuilder
    },
    webview::WebViewBuilder, http::ResponseBuilder
};
use url::Url;

use super::{
    window::{ Window, WindowTrait },
    data_manager::{ DataManager, Wallpaper },
    platform::get_windows, utils::error
};


/// ウィンドウ等を管理するための構造体です。
pub struct Manager<'window> {
    pub event_loop: &'window EventLoop<()>,
    pub windows: Vec<Window<'window>>,
    pub data: DataManager
}


/// managerの実装です。
impl<'window> Manager<'window> {
    pub fn new(event_loop: &'window EventLoop<()>) -> Option<Self> {
        match DataManager::new() {
            Ok(data) => Some(
                Self { event_loop: event_loop, windows: Vec::new(), data: data }
            ),
            Err(message) => { error(&message); None }
        }
    }

    /// 背景ウィンドウを追加します。
    pub fn add(&mut self, data: &'window Wallpaper) -> wry::Result<()> {
        let window = WindowBuilder::new()
            .with_title(format!("FreedomWall - {} Wallpaper Window", data.name))
            .with_decorations(false)
            .build(&self.event_loop)?;
        let webview = WebViewBuilder::new(window)?
            .with_custom_protocol("wry".into(), |request| {
                let path = request.uri().replace("wry://", "");
                let test;
                ResponseBuilder::new()
                    .mimetype(
                        match mime_guess::from_path(&path).first() {
                            Some(mime) => {
                                test = format!("{}/{}", mime.type_(), mime.subtype());
                                &test
                            },
                            _ => "text/plain"
                        }
                    )
                    .body(read(canonicalize(&path)?)?)
            })
            .with_url(&Url::parse_with_params(
                &format!("wry://{}", format!("{}/index.html", &data.path)),
                &data.detail.setting
            ).expect("クエリパラメータの処理に失敗しました。").to_string())?
            .build()?;
        self.windows.push(Window::new(data, webview));
        Ok(())
    }

    /// 指定された背景ウィンドウを削除します。
    pub fn remove(&mut self, name: &str) {
        for (index, window) in self.windows.iter().enumerate() {
            if &window.wallpaper.name == name {
                self.windows.remove(index);
                break;
            };
        };
    }

    /// 背景ウィンドウの処理をします。
    /// 設定されている背景ウィンドウの場所とサイズを対象のアプリに合わせます。
    pub fn process_windows(&'window mut self) {
        let (titles, rects) = get_windows();
        let mut main = false;
        let mut done = Vec::new();
        for (title, (rect, layer)) in titles.iter().zip(rects) {
            if title == "Dock" && layer != 0 { main = true; continue; };
            // 背景の対象として設定されているか検索をする。
            for target in self.data.general.wallpapers.iter() {
                if target.targets.iter().any(|x| title.contains(x)) {
                    // 背景の対象のウィンドウを検索する。
                    for window in self.windows.iter_mut()
                            .filter(|x| x.wallpaper.name == target.wallpaper) {
                        // もし対象のウィンドウが見つかったのならそのウィンドウに背景ウィンドウを移動させる。
                        window.set_rect_from_vec(&rect);
                        window.set_click_through(if main { true } else { false });
                        done.push(window.webview.window().id());
                        continue;
                    };
                    
                    self.add(
                        self.data.get_wallpaper(target.wallpaper)
                            .unwrap_or_else(|| {
                                error(&format!("{}に対応する壁紙が見つかりませんでした。", target.wallpaper));
                                panic!("Failed find wallpaper");
                            })
                    );
                    break;
                };
            };
            if main { main = false; };
        };

    }
}