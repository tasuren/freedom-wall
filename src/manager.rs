//! FreedomWall - Manager

use std::{ path::PathBuf, fs::{ canonicalize, read } };

use wry::{
    application::{
        event_loop::EventLoopWindowTarget,
        window::WindowBuilder
    },
    webview::{ WebViewBuilder, WebView }, http::{ Request, ResponseBuilder, Response },
    Error
};
use url::Url;
use rust_i18n::{ t, set_locale };

use super::{
    window::{ Window, WindowTrait },
    data_manager::{ DataManager, Wallpaper },
    platform::get_windows, APPLICATION_NAME
};


/// ウィンドウ等を管理するための構造体です。
pub struct Manager {
    pub windows: Vec<Window>,
    pub data: DataManager,
    pub setting: WebView
}


/// リクエストを処理します。
fn request_handler(request: &Request) -> Result<Response, Error> {
    if let Ok(url) = Url::parse(&request.uri()) {
        if let Ok(path) = if request.uri().starts_with("wry://") {
            Ok(PathBuf::from(format!("./{}", request.uri().replace("wry://", ""))))
        } else { url.to_file_path() } {
            let test;
            return ResponseBuilder::new()
                .mimetype(
                    match mime_guess::from_path(&path).first() {
                        Some(mime) => {
                            test = format!("{}/{}", mime.type_(), mime.subtype());
                            &test
                        },
                        _ => "text/plain"
                    }
                )
                .body(read(canonicalize(path.to_str().unwrap())?)?);
        };
    };
    ResponseBuilder::new()
        .header("Location", "src/NotFound.html")
        .status(301)
        .body(Vec::new())
}


/// Managerの実装です。
impl Manager {
    pub fn new(event_loop: &EventLoopWindowTarget<()>) -> Result<Self, String> {
        let data = DataManager::new()?;
        // 言語設定を適用させる。
        set_locale(&data.general.language);
        // 設定ウィンドウを作る。
        let window = WindowBuilder::new()
            .with_title(format!("{} Setting", APPLICATION_NAME))
            .build(event_loop).expect("Failed to build the setting window.");
        window.set_visible(true);
        Ok(Self {
            windows: Vec::new(), data: data, setting: WebViewBuilder::new(window).unwrap()
                .with_custom_protocol("wry".into(), request_handler)
                .with_url("wry://src/setting.html").unwrap()
                .build().expect("Failed to build the setting webview.")
        })
    }

    /// 背景ウィンドウを追加します。
    pub fn add(
        &mut self, event_loop: &EventLoopWindowTarget<()>,
        data: Wallpaper, alpha: f64, target: String
    ) -> Result<(), String> {
        let window = WindowBuilder::new()
            .with_title(format!("{} - {} Wallpaper Window", APPLICATION_NAME, data.name))
            .with_decorations(false)
            .build(event_loop).expect("Failed to build the window.");
        match &Url::parse_with_params(
            &format!("wry://{}", format!("{}/index.html", &data.path)),
            &data.detail.setting
        ) {
            Ok(url) => {
                let webview = WebViewBuilder::new(window).unwrap()
                    .with_custom_protocol("wry".into(), request_handler)
                    .with_url(&url.to_string()).unwrap()
                    .with_initialization_script(if data.detail.forceSize { r#"
                        // ウィンドウのサイズに壁紙のサイズを合わせるためのスクリプトを実行する。
                        let resize = function () {
                        for (let element of document.getElementsByClassName("background")) {
                            element.style.width = `${window.innerWidth}px`;
                            element.style.height = `${window.innerHeight}px`;
                        };
                        };
                        window.resize = resize;
        
                        let onload = function () {
                            // 画面全体にHTMLが表示されるようにする。
                            document.getElementsByTagName("head")[0].innerHTML += `
                            <style type="text/css">
                            * {
                                padding: 0;
                                margin: 0;
                            }
                            body {
                                overflow: hidden;
                            }
                                #background {
                                object-fit: fill;
                            }
                            </style>
                            `;
                            resize();
                        };
                        window.onload = onload;"# } else { "" })
                    .with_dev_tool(self.data.general.dev)
                    .build().expect("Failed to build the webview.");
                self.windows.push(Window::new(data, webview, alpha, target));
                Ok(())
            }, _ => Err(t!("core.general.processQueryParameterFailed"))
        }
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
    pub fn process_windows(&mut self, event_loop: &EventLoopWindowTarget<()>) -> Result<(), String>{
        let (titles, rects) = get_windows();
        let mut done = Vec::new();
        // DEBUG: println!("{}", self.windows.len());

        // 背景を設定すべきウィンドウを探す。
        for (title, (rect, main)) in titles.iter().zip(rects) {
            let mut make = None;
            for target in self.data.general.wallpapers.iter() {
                // 背景を設定すべきウィンドウかどうかを調べる。
                if target.targets.iter().any(|target| title.contains(target))
                        && target.exceptions.iter().all(|exception| !title.contains(exception)) {
                    // 既に背景ウィンドウを設定している場合はそのウィンドウの位置と大きさを対象のウィンドウに合わせる。
                    let mut first = true;
                    for window in self.windows.iter_mut() {
                        if window.wallpaper.name == target.wallpaper
                                && &window.target == title {
                            // もし対象のウィンドウが見つかったのならそのウィンドウに背景ウィンドウを移動させる。
                            window.set_rect_from_vec(&rect);
                            if main { window.on_front(); };
                            done.push(window.webview.window().id());
                            first = false;
                            break;
                        };
                    };
                    if first { make = Some((target.wallpaper.clone(), target.alpha, title)); };
                    break;
                };
            };

            if let Some(target) = make {
                // もしまだ作っていない背景ウィンドウなら作る。
                return if let Some(wallpaper) = self.data.get_wallpaper(&target.0) {
                    let _ = self.add(
                        event_loop, wallpaper.clone(), target.1, target.2.clone()
                    );
                    Ok(())
                } else {
                    Err(t!("core.general.findAppropriateWallpaperFailed", name=&target.0))
                }
            };
        };

        // もし既に存在していないアプリへの背景ウィンドウがあるなら必要ないので消す。
        if done.len() < self.windows.len() {
            for index in 0..self.windows.len() {
                if !done.contains(&self.windows[index].webview.window().id()) {
                    self.windows.remove(index);
                };
            };
        };

        Ok(())
    }
}