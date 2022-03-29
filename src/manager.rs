//! FreedomWall - Manager

use std::{
    path::PathBuf, fs::{ canonicalize, read }, time::Duration, thread,
    sync::mpsc::{ channel, Sender },
    collections::HashMap, rc::Rc, cell::{ RefCell, RefMut }
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
use rfd::FileDialog;
use serde_json::{ to_string, from_str };
use url::Url;
use urlencoding::decode;
use rust_i18n::{ t, set_locale };

use super::{
    window::{ Window, WindowTrait },
    data_manager::{ DataManager, Wallpaper, WallpaperJson, Target },
    platform::get_windows, APPLICATION_NAME, utils
};


/// ウィンドウ等を管理するための構造体です。
pub struct Manager {
    pub windows: Vec<Window>,
    pub data: DataManager,
    pub setting: Option<WebView>,
    pub proxy: EventLoopProxy<UserEvents>,
    pub queues: Rc<RefCell<Vec<Queue>>>,
    pub is_setting: bool,
    pub file_dialog: Option<thread::JoinHandle<()>>,
    pub heartbeat_sender: Sender<f32>,
    pub heartbeat: Option<thread::JoinHandle<()>>
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
pub enum UserEvents {
    Request(RequestData),
    FileSelected(String),
    PassedInterval()
}


/// リクエストから適切なファイルを探し出しそれを返します。
fn request2response(uri: &str) -> Result<Response, Error> {
    if let Ok(url) = Url::parse(uri) {
        if let Ok(path) = if uri.starts_with("wry://pages/") {
            Ok(PathBuf::from(format!("pages/{}", url.path())))
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
        .header("Location", "pages/NotFound.html")
        .status(301)
        .body(Vec::new())
}


/// リクエストをイベントでラップしてイベントループに送信します。
/// APIリクエストの処理を実行するのはmain.rsにあるイベントループのイベントハンドラー内からです。
/// (ライフタイムがどうたらこうたらの関係上設計がこうなっています。もし誰か対処法を知っているのなら教えてほしいです)
fn request2waiter(
    proxy: EventLoopProxy<UserEvents>, mut queues: RefMut<Vec<Queue>>, request: &Request
) -> Result<Response, Error> {
    if request.uri().starts_with("wry://api/") {
        if request.uri().contains("reply") {
            // リクエストの返信をあればする。
            let mut replied = false;
            let response = match queues.first() {
                Some(queue) => {
                    replied = true;
                    ResponseBuilder::new()
                        .header("Access-Control-Allow-Origin", "*")
                        .status(queue.status).body(queue.body.clone())
                }, _ => ResponseBuilder::new()
                    .header("Access-Control-Allow-Origin", "*")
                    .status(503).body(Vec::new())
            };
            // もし返信をするのなら最後の返信キューを削除する。
            if replied { queues.remove(0); };
            response
        } else {
            // APIリクエストのイベントを送信する。
            let _ = proxy.send_event(UserEvents::Request(RequestData {
                uri: request.uri().to_string(), body: String::from_utf8(request.body.clone())
                    .unwrap_or("".to_string())
            }));
            ResponseBuilder::new()
                .header("Access-Control-Allow-Origin", "*")
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
        let (tx, rx) = channel();
        let mut manager = Self {
            windows: Vec::new(), data: data, setting: None, proxy: proxy,
            queues: Rc::new(RefCell::new(Vec::new())), is_setting: false,
            file_dialog: None, heartbeat_sender: tx, heartbeat: None
        };
        // 設定画面のウィンドウを作る。
        manager.setting = Some(manager.make_setting_window(event_loop));
        // 定期的にイベントを呼び出すためのスレッドを動かす。
        let cloned_proxy = manager.proxy.clone();
        let mut cloned_interval = Duration::from_secs_f32(manager.data.general.updateInterval);
        manager.heartbeat = Some(thread::spawn(move || {
            loop {
                if let Ok(new) = rx.recv_timeout(cloned_interval) {
                    cloned_interval = Duration::from_secs_f32(new);
                    thread::sleep(cloned_interval);
                };
                if cloned_proxy.send_event(UserEvents::PassedInterval()).is_err() {
                    break;
                };
            };
        }));
        Ok(manager)
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
                    proxy.clone(), cloned_queues.borrow_mut(), request
                )
            )
            .with_url("wry://pages/_home.html").unwrap()
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
            &data.detail.setting,
        ) {
            Ok(url) => {
                let proxy = self.proxy.clone();
                let cloned_queues = self.queues.clone();
                let webview = WebViewBuilder::new(window).unwrap()
                    .with_custom_protocol(
                        "wry".into(), move |request| request2waiter(
                            proxy.clone(), cloned_queues.borrow_mut(), request
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
                        window.addEventListener('resize', resize);
                        window.addEventListener('load', function (_) {
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
                        });"# } else { "" })
                    .with_dev_tool(self.data.general.dev)
                    .build().expect("Failed to build the webview.");
                let mut new = Window::new(data, webview, alpha, target);
                new.set_click_through(!self.data.general.dev);
                self.windows.push(new);
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
            if title.contains("FreedomWall") { continue; };
            let mut make = None;
            for target in self.data.general.wallpapers.iter() {
                // 背景を設定すべきウィンドウかどうかを調べる。
                if target.targets.iter().any(|target| title.contains(target))
                        && (target.exceptions.len() == 0 || target.exceptions.iter()
                            .all(|exception| !title.contains(exception))) {
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
                    println!("Add window: {}", target.2);
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
            for index in self.windows.len()..0 {
                if !done.contains(&self.windows[index].webview.window().id()) {
                    println!("Remove window: {}", self.windows[index].target);
                    self.windows.remove(index);
                };
            };
        };

        Ok(())
    }

    /// APIリクエストを処理します。
    pub fn on_request(&mut self, uri: &str, data: String) -> Queue {
        // 色々整えたり下準備をする。
        let tentative_path = decode(
            &uri.replace("wry://api/", "").replace("?", "/")
        ).unwrap().to_string();
        let path: Vec<&str> = tentative_path.split("/").collect();
        let error = RefCell::new("Not found".to_string());
        let borrowed = error.borrow();
        let make_error = |body: Option<String>| Queue {
            status: 400, body: match body {
                Some(value) => value.bytes().collect(),
                _ => borrowed.bytes().collect()
            }
        };

        /* 以下は使わないと言い切れないコードなのでコメントアウトをしています。
        let tentative_url = Url::parse(uri);
        if let Err(_) = tentative_url { return make_error(); };
        let url = tentative_url.unwrap();
        */

        let length = path.len();
        if length < 3 { return make_error(None); };
        let (OK, NOTFOUND) = (
            Ok("Ok".to_string()), Err("Not found".to_string())
        );
        let mut write_mode = "";
        let is_update = path[2] == "update";

        println!("Request: {} {}", uri, data);

        // リクエストを処理する。
        let tentative = match path[0] {
            "setting" => {
                // setting/...
                // 一般の設定の取得と更新
                write_mode = "general";
                match path[1] {
                    // 言語設定
                    "language" => {
                        if is_update {
                            if "jaen".contains(&data) {
                                set_locale(&data);
                                self.data.general.language = data;
                                OK
                            } else { Err(t!("core.setting.notAppropriateLanguage")) }
                        } else { Ok(self.data.general.language.clone()) }
                    },
                    "wallpapers" => {
                        // 登録されている壁紙
                        if is_update {
                            if let Ok(wallpapers) =
                                    from_str::<Vec<Target>>(&data) {
                                self.data.general.wallpapers = wallpapers;
                                // 現在開かれている背景ウィンドウを消す。
                                self.windows = Vec::new();
                                OK
                            } else { Err(t!("core.setting.loadJsonFailed")) }
                        } else {
                            Ok(to_string(&self.data.general.wallpapers).unwrap())
                        }
                    },
                    "interval" => {
                        // 背景ウィンドウの位置とサイズ更新の設定
                        if is_update {
                            if let Ok(value) = data.parse() {
                                self.data.general.updateInterval = value;
                                self.heartbeat_sender.send(value,).unwrap();
                                OK
                            } else { Err("Failed to parse value.".to_string()) }
                        } else { Ok(self.data.general.updateInterval.to_string()) }
                    },
                    "dev" => {
                        // 開発者モードをONにするかどうか。
                        if is_update {
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
                match path[1] {
                    "all" => {
                        // 全ての壁紙を取得します。または指定された壁紙を追加します。
                        if is_update {
                            let data: Vec<&str> = data.split("?").collect();
                            match self.data.add_wallpaper(
                                data[0].to_string(), data[1].to_string()
                            ) {
                                Err(message) => Err(message), _ => OK
                            }
                        } else {
                            let mut response_data = HashMap::new();
                            for wallpaper in self.data.wallpapers.iter() {
                                response_data.insert(
                                    wallpaper.name.clone(), wallpaper.detail.clone()
                                );
                            };
                            Ok(to_string(&response_data).unwrap())
                        }
                    },
                    "one" => {
                        // 壁紙プロファイルの削除か更新
                        // wallpapers/one/update/<name>/<subject>
                        if is_update && length >= 5 {
                            match self.data.get_wallpaper_index(path[3]) {
                                Some(index) => {
                                    match if path[4] == "write" {
                                        match from_str::<WallpaperJson>(&data) {
                                            Ok(value) => {
                                                self.data.wallpapers[index].detail = value;
                                                self.data.write_wallpaper(index)
                                            },
                                            Err(message) => Err(format!(
                                                "{}\nDetail: {}",
                                                t!("core.setting.loadJsonFailed"),
                                                message.to_string()
                                            ))
                                        }
                                    } else {
                                        self.data.remove_wallpaper(index)
                                    } {
                                        Ok(_) => {
                                            let mut update_queue = Vec::new();
                                            for (index, _) in self.data.get_wallpaper_setting(path[3]) {
                                                if path[4] == "remove" { update_queue.push(index); };
                                            };

                                            // 使われている壁紙設定を削除する。(削除時限定)
                                            update_queue.sort(); update_queue.reverse();
                                            for index in update_queue {
                                                self.data.general.wallpapers.remove(index);
                                            };
                                            // ウィンドウをリセットする。
                                            self.windows = Vec::new();

                                            // データを書き込む。
                                            match self.data.write_setting() {
                                                Ok(_) => OK, Err(message) => Err(message)
                                            }
                                        },
                                        Err(message) => Err(message)
                                    }
                                }, _ => NOTFOUND
                            }
                        } else { OK }
                    },
                    "rename" => {
                        // 壁紙プロファイルの名前変更
                        if length >= 5 {
                            if let Err(message) = self.data.mv_wallpaper(path[3], path[4]) {
                                Err(message)
                            } else {
                                // 既に使われているプロファイルの場合は再設定を行う。
                                let mut update_queue = Vec::new();
                                for (index, _) in self.data.get_wallpaper_setting(path[3]) {
                                    update_queue.push(index);
                                };
                                update_queue.sort(); update_queue.reverse();
                                for index in update_queue {
                                    let mut before = self.data.general.wallpapers
                                        .remove(index);
                                    before.wallpaper = path[4].to_string();
                                    self.data.general.wallpapers.push(before);
                                };
                                // ウィンドウをリセットする。
                                self.windows = Vec::new();
                                // 設定を書き込む。
                                match self.data.write_setting() {
                                    Ok(_) => OK, Err(message) => Err(message)
                                }
                            }
                        } else { OK }
                    }, _ => NOTFOUND
                }
            },
            "templates" => {
                // templates/all/get
                // テンプレートの取得を行えます。
                Ok(to_string(&self.data.templates).unwrap())
            },
            "gettext" => {
                // gettext/<text>/get
                Ok(t!(path[2]))
            },
            "open" => {
                // open/.../...
                // ファイル選択
                let cloned = self.proxy.clone();
                self.file_dialog = Some(thread::spawn(move || {
                    let _ = cloned.send_event(UserEvents::FileSelected(match FileDialog::new().pick_file() {
                        Some(path) => path.to_str().unwrap().to_string(),
                        _ => { utils::error(&t!("core.setting.failedRead")); panic!("Error occurred."); }
                    }));
                    ()
                }));
                OK
            },
            _ => NOTFOUND
        };

        // もしデータ書き込みが必要なら書き込む。
        if is_update && !write_mode.is_empty() {
            match write_mode {
                "general" => self.data.write_setting(),
                _ => Err("The world is revolving!".to_string())
            }.unwrap();
        };

        // レスポンスデータをまとめる。
        match tentative {
            Ok(response_data) => Queue { status: 200, body: response_data.bytes().collect() },
            _ => make_error(Some(tentative.err().unwrap()))
        }
    }
}