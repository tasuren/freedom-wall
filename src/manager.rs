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
    data_manager::{
        DataManager, Wallpaper, WallpaperJson, Target,
        add_setting_path
    },
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
fn request2response(request: &Request) -> Result<Response, Error> {
    #[cfg(target_os="windows")]
    let raw = request.uri().replace("wry://c//", "file:///c:/");
    #[cfg(target_os="windows")]
    let uri = &raw;
    #[cfg(target_os="macos")]
    let uri = request.uri();

    if let Ok(url) = Url::parse(uri) {
        if let Ok(path) = if uri.starts_with("wry://pages/") {
            Ok(PathBuf::from(format!("pages/{}", url.path())))
         } else { url.to_file_path() } {
            println!("File request: {}", uri);
            let test;
            return ResponseBuilder::new()
                .mimetype(
                    match mime_guess::from_path(&path).first() {
                        Some(mime) => {
                            test = format!("{}/{}", mime.type_(), mime.subtype());
                            &test
                        }, _ => "text/plain"
                    }
                )
                .status(200)
                .body(read(canonicalize(path.to_str().unwrap())?)?);
        };
    };
    println!("File request (NotFound): {}", uri);
    ResponseBuilder::new()
        .header("Location", "wry://pages/NotFound.html")
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
    } else { request2response(request) }
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
                    if new == 0.0 {
                        break;
                    } else {
                        cloned_interval = Duration::from_secs_f32(new);
                        thread::sleep(cloned_interval);
                    };
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
            .with_devtools(true)
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
                    .with_initialization_script(&format!(
                        "window.escapeHTML = function (str) {{
                          // HTMLをエスケープする関数
                          return str.replace(/&/g, '&amp;').replace(/</g, '&lt;')
                              .replace(/>/g, '&gt;').replace(/\"/g, '&quot;')
                              .replace(/'/g, '&#39;');
                        }};

                        window.addEventListener('load', function (_) {{
                            // 拡張機能を読み込む。
                            let head = document.getElementsByTagName('head')[0];
                            console.log(head);
                            for (let key of [{}]) {{
                                var script = document.createElement('script');
    
                                script.type = 'module';
                                script.src = `wry://${{key}}/init.js`;
                                console.log('Load extension:', key);
    
                                head.appendChild(script);
                            }};
                        }});
                        window.__WINDOWS__ = {};{}",
                        if self.data.extensions.is_empty() { "".to_string() }
                        else { format!(
                            "\"{}\"",
                            self.data.extensions.iter().map(|x| x.path.replace("\"", "\\\""))
                                .collect::<Vec<String>>().join("\", \"")
                        ) }, if cfg!(target_os="windows") { "true" } else { "false" },
                        if data.detail.forceSize {
                            "// ウィンドウのサイズに壁紙のサイズを合わせるためのスクリプトを実行する。
                            let resizeElement = function (element) {
                                element.style.width = `${window.innerWidth}px`;
                                element.style.height = `${window.innerHeight}px`;
                            };
                            let resize = function () {
                                for (let element of document.getElementsByClassName('background')) {
                                    console.log(element);
                                    resizeElement(element);
                                };
                                if (window.__backgrounds__)
                                    for (let element of window.__backgrounds__)
                                        resizeElement(element);
                            };
                            window.addEventListener('resize', resize);
                            window.addEventListener('load', function (_) {
                                // 画面全体にHTMLが表示されるようにする。
                                document.getElementsByTagName('head')[0].innerHTML += `
                                    <style type=\"text/css\">
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
                            });"
                        } else { "" }))
                    .with_devtools(self.data.general.dev)
                    .build().expect("Failed to build the webview.");
                let new = Window::new(data, webview, alpha, target);
                if self.data.general.dev { new.webview.open_devtools(); };
                self.windows.push(new);
            Ok(())
            }, _ => Err(t!("core.general.processQueryParameterFailed"))
        }
    }

    /// 背景ウィンドウの処理をします。
    /// 設定されている背景ウィンドウの場所とサイズを対象のアプリに合わせます。
    pub fn process_windows(&mut self, event_loop: &EventLoopWindowTarget<UserEvents>) -> Result<(), String>{
        let (titles, rects) = get_windows();
        let mut done = Vec::new();
        // DEBUG: println!("{}", self.windows.len());

        // 背景を設定すべきウィンドウを探す。
        for (title, (rect, main, extra)) in titles.iter().zip(rects) {
            if title.contains("FreedomWall") { continue; };
            let mut make = None;
            for target in self.data.general.wallpapers.iter() {
                // 背景を設定すべきウィンドウかどうかを調べる。
                if target.targets.iter().any(|target| title.contains(target))
                        && (target.exceptions.len() == 0 || target.exceptions.iter()
                            .all(|exception| !title.contains(exception))) {
                    let mut first = true;
                    for window in self.windows.iter_mut() {
                        if window.wallpaper.name == target.wallpaper
                                && &window.target == title {
                            // もし対象のウィンドウなら背景ウィンドウのサイズの変更や移動をさせたりする。
                            window.set_front(main);
                            window.set_rect_from_vec(&rect);
                            window.set_order(extra);
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
            for index in self.get_windows_range() {
                if !done.contains(&self.windows[index].webview.window().id()) {
                    self.remove(index);
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

        let (ok, notfound) = (Ok("Ok".to_string()), Err("Not found".to_string()));

        let length = path.len();
        if length < 3 { return make_error(None); };
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
                    "language" => if is_update {
                        if "jaen".contains(&data) {
                            set_locale(&data);
                            self.data.general.language = data;
                            ok
                        } else { Err(t!("core.setting.notAppropriateLanguage")) }
                    } else { Ok(self.data.general.language.clone()) },
                    // 登録されている壁紙
                    "wallpapers" => if is_update {
                        if let Ok(wallpapers) =
                                from_str::<Vec<Target>>(&data) {
                            self.data.general.wallpapers = wallpapers;
                            // 現在開かれている背景ウィンドウを消す。
                            self.reset_windows();
                            ok
                        } else { Err(t!("core.setting.loadJsonFailed")) }
                    } else {
                        Ok(to_string(&self.data.general.wallpapers).unwrap())
                    },
                    // 背景ウィンドウの位置とサイズ更新の設定
                    "interval" => if is_update {
                        if let Ok(value) = data.parse() {
                            self.data.general.updateInterval = value;
                            self.heartbeat_sender.send(value,).unwrap();
                            ok
                        } else { Err("Failed to parse value.".to_string()) }
                    } else { Ok(self.data.general.updateInterval.to_string()) },
                    // 開発者モードをONにするかどうか。
                    "dev" => if is_update {
                        self.data.general.dev = if data == "1" { true } else { false };
                        self.reset_windows();
                        ok
                    } else { Ok((self.data.general.dev as usize).to_string()) },
                    _ => notfound
                }
            },
            // wallpapers/...
            // 壁紙リストの壁紙の設定
            "wallpapers" => match path[1] {
                // 全ての壁紙を取得します。または指定された壁紙を追加します。
                "all" => if is_update {
                    let data: Vec<&str> = data.split("?").collect();
                    match self.data.add_wallpaper(
                        data[0].to_string(), data[1].to_string()
                    ) {
                        Err(message) => Err(message), _ => ok
                    }
                } else {
                    let mut response_data = HashMap::new();
                    for wallpaper in self.data.wallpapers.iter() {
                        response_data.insert(
                            wallpaper.name.clone(), wallpaper.detail.clone()
                        );
                    };
                    Ok(to_string(&response_data).unwrap())
                },
                // 壁紙プロファイルの削除か更新
                // wallpapers/one/update/<name>/<subject>
                "one" => if is_update && length >= 5 {
                    match self.data.get_wallpaper_index(path[3]) {
                        Some(index) => match if path[4] == "write" {
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
                                self.reset_windows();

                                // データを書き込む。
                                match self.data.write_setting() {
                                    Ok(_) => ok, Err(message) => Err(message)
                                }
                            },
                            Err(message) => Err(message)
                        }, _ => notfound
                    }
                } else { ok },
                // 壁紙プロファイルの名前変更
                "rename" => if length >= 5 {
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
                        self.reset_windows();
                        // 設定を書き込む。
                        match self.data.write_setting() {
                            Ok(_) => ok, Err(message) => Err(message)
                        }
                    }
                } else { ok }, _ => notfound
            },
            // templates/all/get
            // テンプレートの取得を行えます。
            "templates" => Ok(to_string(&self.data.templates).unwrap()),
            // extensions/
            // 拡張機能
            "extensions" => match path[1] {
                "all" => match path[2] {
                    "get" => {
                        let mut data = HashMap::new();
                        for extension in self.data.extensions.iter() {
                            data.insert(&extension.name, &extension.detail);
                        };
                        Ok(to_string(&data).unwrap())
                    },
                    "update" => notfound,
                    _ => notfound
                },
                "one" => if length >= 4 {
                    match self.data.get_extension(path[3]) {
                        Some((index, extension)) => match path[2] {
                            "get" => Ok(to_string(&extension.detail).unwrap()),
                            "update" => match from_str::<HashMap<String, String>>(&data) {
                                Ok(value) => {
                                    self.data.extensions[index].detail.setting = value;
                                    self.reset_windows();
                                    match self.data.write_extension(path[3].to_string()){
                                        Ok(_) => ok,
                                        _ => Err(t!("core.setting.failedWrite", path=path[3]))
                                    }
                                }, _ => notfound
                            }, _ => notfound
                        }, _ => notfound
                    }
                } else { notfound },
                "reload" => match self.data.read_extensions() {
                    Ok(_) => ok, Err(message) => Err(message)
                },
                _ => notfound
            },
            // gettext/<text>/get
            "gettext" => Ok(t!(path[2])),
            "open" => {
                // open/.../...
                // ファイル選択
                let cloned = self.proxy.clone();
                self.file_dialog = Some(thread::spawn(move || {
                    let _ = cloned.send_event(UserEvents::FileSelected(match FileDialog::new().pick_file() {
                        Some(path) => path.as_path().display().to_string(),
                        _ => { utils::error(&t!("core.setting.failedRead")); panic!("Error occurred."); }
                    }));
                    ()
                }));
                ok
            },
            "openFolder" => {
                // openFolder/.../...
                // フォルダを開く。
                utils::open_folder(data);
                ok
            },
            "getPath" => add_setting_path(""),
            _ => notfound
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

    fn get_windows_range(&mut self) -> Vec<usize> {
        let mut range: Vec<usize> = (0..self.windows.len()).collect();
        range.reverse();
        range
    }

    /// ウィンドウを閉じます。
    pub fn remove(&mut self, index: usize) {
        println!("Remove window: {}", self.windows[index].target);
        self.windows[index].webview.evaluate_script(
            "window.close();"
        ).unwrap();
        self.windows.remove(index);
    }

    /// 全ての背景ウィンドウをリセットします。
    pub fn reset_windows(&mut self) {
        println!("Reset windows");
        for index in self.get_windows_range() {
            self.remove(index);
        };
    }

    /// お片付けをします。
    pub fn stop(&mut self) {
        if let Some(handle) = self.heartbeat.take() {
            let _ = self.heartbeat_sender.send(0.0);
            handle.join().expect("Failed to join heartbeat thread.");
        };
        self.data.general.wallpapers = Vec::new();
        self.reset_windows();
    }
}