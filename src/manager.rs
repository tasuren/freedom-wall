//! FreedomWall - Manager

use std::fs::{ canonicalize, read };

use wry::{
    application::{
        event_loop::EventLoopWindowTarget,
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
pub struct Manager {
    pub windows: Vec<Window>,
    pub data: DataManager
}


/// managerの実装です。
impl Manager {
    pub fn new() -> Option<Self> {
        match DataManager::new() {
            Ok(data) => Some(
                Self { windows: Vec::new(), data: data }
            ),
            Err(message) => { error(&message); None }
        }
    }

    /// 背景ウィンドウを追加します。
    pub fn add(
        &mut self, event_loop: &EventLoopWindowTarget<()>,
        data: Wallpaper, alpha: f64, target: String
    ) -> wry::Result<()> {
        let window = WindowBuilder::new()
            .with_title(format!("FreedomWall - {} Wallpaper Window", data.name))
            .with_decorations(false)
            .build(event_loop)?;
        let webview = WebViewBuilder::new(window)?
            .with_custom_protocol("wry".into(), |request| {
                match Url::parse(&request.uri().replace("wry://", "file://")) {
                    Ok(url) => {
                        let path = url.to_file_path().unwrap();
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
                            .body(read(canonicalize(path.to_str().unwrap())?)?)
                    }, _ => ResponseBuilder::new()
                        .header("Location", "src/NotFound.html")
                        .status(301)
                        .body(Vec::new())
                }
            })
            .with_url(&Url::parse_with_params(
                &format!("wry://{}", format!("{}/index.html", &data.path)),
                &data.detail.setting
            ).expect("クエリパラメータの処理に失敗しました。").to_string())?
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
            .build()?;
        self.windows.push(Window::new(data, webview, alpha, target));
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
                            window.set_click_through(main);
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
                    Err(format!("{}に対応する壁紙が見つかりませんでした。", target.0))
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