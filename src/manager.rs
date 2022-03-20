//! FreedomWall - Manager

use std::{
    path::PathBuf, fs::{ canonicalize, read },
    collections::HashMap, rc::Rc, cell::{ Ref, RefCell }
};

use wry::{
    application::{
        event_loop::{ EventLoopProxy, EventLoopWindowTarget },
        window::WindowBuilder
    },
    webview::{ WebViewBuilder, WebView },
    http::{ Request, ResponseBuilder, Response },
    Error
};
use serde_json::{ to_string, from_str };
use url::Url;
use rust_i18n::{ t, set_locale };

use super::{
    window::{ Window, WindowTrait },
    data_manager::{ DataManager, Wallpaper, Target },
    platform::get_windows, APPLICATION_NAME
};


/// ウィンドウ等を管理するための構造体です。
pub struct Manager {
    pub windows: Vec<Window>,
    pub data: DataManager,
    pub setting: Option<WebView>,
    pub proxy: EventLoopProxy<UserEvents>,
    pub queues: Rc<RefCell<Vec<Queue>>>
}


/// レスポンスキューです。
pub struct Queue {
    pub status: u16,
    pub body: Vec<u8>
}


/// イベントループのハンドラーにて処理するリクエストのデータをまとめるための構造体です。
pub struct RequestData {
    pub uri: String,
    pub body: String
}


/// レスポンスを入れてmain.rsにて実行するのに使うユーザーイベントの列挙型です。
pub enum UserEvents { Request(RequestData) }


/// リクエストから適切なファイルを探し出しそれを返します。します。
fn request2response(uri: &str) -> Result<Response, Error> {
    if let Ok(url) = Url::parse(uri) {
        if let Ok(path) = if uri.starts_with("wry://") {
            Ok(PathBuf::from(format!("./{}", uri.replace("wry://", ""))))
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


/// リクエストをイベントでラップしてイベントループに送信します。
/// APIリクエストの処理を実行するのはmain.rsからです。(ライフタイムがどうたらこうたらの関係上の設計がこれです)
fn request2waiter(
    proxy: EventLoopProxy<UserEvents>, queues: Ref<Vec<Queue>>, request: &Request
) -> Result<Response, Error> {
    if request.uri().starts_with("wry://api/") {
        if request.uri().contains("reply") {
            match queues.first() {
                Some(queue) => ResponseBuilder::new()
                    .status(200).body(queue.body.clone()),
                _ => ResponseBuilder::new()
                    .status(503).body(Vec::new())
            }
        } else {
            // イベントを送信する。
            let _ = proxy.send_event(UserEvents::Request(RequestData {
                uri: request.uri().to_string(), body: String::from_utf8(request.body.clone())
                    .unwrap_or("".to_string())
            }));
            ResponseBuilder::new()
                .status(201)
                .body(Vec::new())
        }
    } else { request2response(request.uri()) }
}


/// Managerの実装です。
impl Manager {
    pub fn new(
        event_loop: &EventLoopWindowTarget<UserEvents>,
        proxy: EventLoopProxy<UserEvents>
    ) -> Result<Self, String> {
        let data = DataManager::new()?;
        // 言語設定を適用させる。
        set_locale(&data.general.language);
        let mut manager = Self {
            windows: Vec::new(), data: data, setting: None,
            proxy: proxy, queues: Rc::new(RefCell::new(Vec::new()))
        };
        manager.setting = Some(manager.make_setting_window(event_loop));
        Ok(manager)
    }

    /// APIリクエストを処理します。
    pub fn on_request(&mut self, uri: &str, data: String) -> Queue {
        // 色々整えたり下準備をする。
        let tentative_path = uri.replace("wry://api/", "")
            .replace("?", "/");
        let path: Vec<&str> = tentative_path.split("/").collect();
        let error = RefCell::new("Not found".to_string());
        let borrowed = error.borrow();
        let make_error = || Queue {
            status: 400, body: borrowed.bytes().collect()
        };

        /* 以下は将来性を考慮してでのコメントアウトです。
        let tentative_url = Url::parse(uri);
        if let Err(_) = tentative_url { return make_error(); };
        let url = tentative_url.unwrap();
        */

        let length = path.len();
        let (OK, NOTFOUND) = (
            Ok("Ok".to_string()), Err("Not found".to_string())
        );
        if length <= 3 { return make_error(); };
        let mut write_mode = "";

        // リクエストを処理する。
        let tentative = match path[0] {
            "setting" => {
                // setting/...
                // 一般の設定の取得と更新
                write_mode = "general";
                match path[1] {
                    // 言語設定
                    "language" => {
                        if path[2] == "update" {
                            if "jaen".contains(&data) {
                                set_locale(&data);
                                OK
                            } else { Err(t!("core.setting.notAppropriateLanguage")) }
                        } else { Ok(to_string(&self.data.general.language).unwrap()) }
                    },
                    "wallpapers" => {
                        // 登録されている壁紙
                        if path[2] == "update" {
                            if let Ok(wallpapers) =
                                    from_str::<Vec<Target>>(&data) {
                                self.data.general.wallpapers = wallpapers;
                                OK
                            } else { Err(t!("core.setting.loadJsonFailed")) }
                        } else {
                            Ok(to_string(&self.data.general.wallpapers).unwrap())
                        }
                    },
                    "updateInterval" => {
                        // 背景ウィンドウの位置とサイズ更新の設定
                        if path[2] == "update" {
                            if let Ok(value) = data.parse() {
                                self.data.general.updateInterval = value;
                                OK
                            } else { Err("Failed to parse value.".to_string()) }
                        } else { Ok(self.data.general.updateInterval.to_string()) }
                    },
                    "dev" => {
                        // 開発者モードをONにするかどうか。
                        if path[2] == "update" {
                            self.data.general.dev = if data == "1" { true } else { false };
                            OK
                        } else { Ok((self.data.general.dev as usize).to_string()) }
                    },
                    _ => NOTFOUND
                }
            },
            "wallpapers" => {
                // wallpapers/...
                // 壁紙リストの壁紙の設定
                write_mode = "wallpapers";
                match path[1] {
                    "all" => {
                        // 全ての壁紙を取得します。
                        if path[2] == "update" { OK }
                        else {
                            let mut response_data = HashMap::new();
                            for wallpaper in self.data.wallpapers.iter() {
                                response_data.insert(
                                    wallpaper.name.clone(), wallpaper.detail.clone()
                                );
                            };
                            Ok(to_string(&response_data).unwrap())
                        }
                    }, _ => NOTFOUND
                }
            },
            _ => NOTFOUND
        };

        // もしデータ書き込みが必要なら書き込む。
        if !write_mode.is_empty() {
            match write_mode {
                "general" => self.data.write_setting(),
                "wallpapers" => self.data.write_wallpaper(1),
                _ => Err("The world is revolving!".to_string())
            }.unwrap();
        };

        // レスポンスデータをまとめる。
        if let Ok(response_data) = tentative {
            return Queue { status: 200, body: response_data.bytes().collect() };
        } else { *error.borrow_mut() = tentative.err().unwrap() };

        make_error()
    }

    /// 設定画面を作ります。
    pub fn make_setting_window(&mut self, event_loop: &EventLoopWindowTarget<UserEvents>) -> WebView {
        // 設定ウィンドウを作る。
        let window = WindowBuilder::new()
            .with_title(format!("{} Setting", APPLICATION_NAME))
            .build(event_loop).expect("Failed to build the setting window.");
        let proxy = self.proxy.clone();
        let cloned_queues = self.queues.clone();
        WebViewBuilder::new(window).unwrap()
            .with_custom_protocol(
                "wry".into(), move |request| request2waiter(
                    proxy.clone(), cloned_queues.borrow(), request
                )
            )
            .with_url("wry://src/setting.html").unwrap()
            .with_dev_tool(self.data.general.dev)
            .build().expect("Failed to build the setting webview.")
    }

    /// 背景ウィンドウを追加します。
    pub fn add(
        &mut self, event_loop: &EventLoopWindowTarget<UserEvents>,
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
                let proxy = self.proxy.clone();
                let cloned_queues = self.queues.clone();
                let webview = WebViewBuilder::new(window).unwrap()
                    .with_custom_protocol(
                        "wry".into(), move |request| request2waiter(
                            proxy.clone(), cloned_queues.borrow(), request
                        )
                    )
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
    pub fn process_windows(&mut self, event_loop: &EventLoopWindowTarget<UserEvents>) -> Result<(), String>{
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