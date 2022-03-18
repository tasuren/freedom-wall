//! FreedomWall - DataManager

use std::{
    collections::HashMap, env::args, path::{ Path, PathBuf },
    ffi::OsStr, fs::{ File, read_to_string, create_dir },
    io::Write, cell::RefCell
};

use serde::{ Serialize, Deserialize };
use serde_json::{ to_string_pretty, from_str };
use platform_dirs::AppDirs;

use super::{ utils::error, APPLICATION_NAME };


const FAILED_JSON: &str = "JSON生成時にエラーが発生しました。";
const DATA_DEFAULT: &str = r#"{
    "language": "ja", "wallpapers": [], "updateInterval": 0.1
}"#;


//  壁紙設定
/// アプリ名を取得します。
fn get_application_name() -> String {
    let data: Vec<String> = args().collect();
    if data.len() == 1 {
        APPLICATION_NAME.to_string()
    } else if data[1] == "test" {
        format!("{}Dev", APPLICATION_NAME)
    } else { APPLICATION_NAME.to_string() }
}


/// 渡されたパスに設定ファイルを保存する場所のパスを追加します。
fn add_setting_path(path: &str) -> String {
    let data_dir = AppDirs::new(Some(&get_application_name()), false)
        .expect("設定を保存する場所のパスが見つかりませんでした。")
        .data_dir;
    let app_dir = data_dir.to_str()
        .expect("設定を保存する場所のパスの処理中になんらかのエラーが発生しました。");
    if path.is_empty() { app_dir.to_string() }
    else { format!("{}/{}", app_dir, path) }
}


/// 壁紙プロファイルの設定ファイルである`data.json`の構造体です。
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WallpaperJson {
    pub author: String,
    pub description: String,
    pub setting: HashMap<String, String>,
    pub dev: bool
}


/// 壁紙の設定データの構造体です。
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Wallpaper {
    pub name: String,
    pub path: String,
    pub detail: WallpaperJson
}


/// 背景対象となるウィンドウのデータの構造体です。
#[derive(Serialize, Deserialize)]
pub struct Target {
    pub targets: Vec<String>,
    pub exception: Vec<String>,
    pub alpha: f64,
    pub wallpaper: String
}


/// FreedomWallの設定ファイルの構造体です。
#[derive(Serialize, Deserialize)]
pub struct GeneralSetting {
    pub language: String,
    pub wallpapers: Vec<Target>,
    pub updateInterval: f32
}


/// セーブデータを管理するための構造体です。
type Wallpapers = Vec<Wallpaper>;
pub struct DataManager {
    pub general: GeneralSetting,
    pub wallpapers: Wallpapers
}


fn failed_read(path: String) -> String { format!("{}の読み込みに失敗しました。", path) }


/// 指定されたパスにあるファイル,フォルダのVecを取得します。
fn get_files(target: &PathBuf, dir: bool) -> Option<Vec<PathBuf>> {
    match target.read_dir() {
        Ok(entries) => {
            let mut result: Vec<PathBuf> = Vec::new();
            for tentative in entries {
                let entry = tentative.expect(&failed_read("ファイル".to_string())).path();
                if !dir || entry.is_dir() {
                    result.push(entry);
                };
            };
            Some(result)
        }, _ => None
    }
}


/// 渡されたPathBufから名前を取得します。
fn get_name<'a>(path: &'a PathBuf) -> &'a str {
    path.file_name().unwrap_or(&OsStr::new("")).to_str().unwrap()
}


/// 指定されたパスのフォルダにあるすべてのフォルダkらあファイル検索を行います。
/// また、on_found引数でファイルの読み込み処理等も行うこともできます。
/// on_foundに渡されるものは左から順にフォルダのパス,フォルダのPathBuf,ファイル名,ファイルのパス
fn search_files<F: Fn(String, &PathBuf, &str, &String) -> ()>(
    path: &str, targets: Vec<&str>, on_found: F
) -> Result<(), String> {
    if let Some(dirs) = get_files(&PathBuf::from(add_setting_path(path)), true) {
        // ファイルを探す。
        for path in dirs {
            let path_string = &path.display().to_string();
            let mut ok: Vec<&str> = Vec::new();

            if let Some(files) = get_files(&path, false) {
                for file in files.iter().filter(|x| x.is_file()) {
                    let file_name = get_name(&file);
                    if targets.contains(&file_name) {
                        ok.push(file_name);
                        let file_path = format!("{}/{}", path_string, file_name);
                        on_found(
                            file_path.replace(&format!("/{}", file_name), ""),
                            &path, file_name, &file_path
                        );
                    };
                };

                // もし指定されたファイル等を全て見つけられなかったのならエラーとする。
                if ok.len() < targets.len() {
                    return Err(format!(
                        "{}に`{}`が存在しないため壁紙を読み込めません。",
                        path_string, targets.iter()
                            .filter(|x| !ok.contains(x))
                            .map(|x| x.to_string())
                            .collect::<Vec<String>>().join("`, `")
                    ));
                };
            } else {
                return Err(failed_read(path_string.to_string()))
            };
        };
        Ok(())
    } else { Err(failed_read(path.to_string())) }
}


/// 指定されたパスにあるファイルの文字列を全て読み込みます。
fn read(path: &String) -> Result<String, String> {
    if let Ok(raw) = read_to_string(path) {
        Ok(raw)
    } else { Err(failed_read(path.to_string())) }
}


/// 指定されたファイルが存在するか確認をします。
fn exists(path: &str) -> Result<(), String> {
    if !PathBuf::from(path).exists() {
        Err(format!("{}が見つかりませんでした。", path))
    } else { Ok(()) }
}


/// 指定されたパスに指定された文字列をすべて書き込みます。
fn write(path: &String, data: String) -> Result<(), String> {
    if let Ok(tentative) = File::create(path) {
        let mut f = tentative;
        if f.write_all(data.as_bytes()).is_err() {
            return Err(format!("{}へのデータの書き込みに失敗しました。", path));
        };
    };
    Ok(())
}


/// 設定を読み込みます。
fn read_setting() -> Result<GeneralSetting, String> {
    let path = add_setting_path("data.json");
    exists(&path)?;
    let raw = read(&path)?;
    if let Ok(data) = from_str::<GeneralSetting>(&raw) {
        Ok(data)
    } else { Err(failed_read(path)) }
}


/// 壁紙の設定を読み込みます。
fn read_wallpapers() -> Result<Wallpapers, String> {
    let error = RefCell::new(String::new());
    let wallpapers = RefCell::new(Vec::new());
    search_files(
        "wallpapers", vec!["index.html", "data.json"], |path, dir, file_name, file_path| {
            if file_name == "data.json" {
                if let Ok(raw) = read(&file_path) {
                    if let Ok(data) = from_str::<WallpaperJson>(&raw) {
                        wallpapers.borrow_mut().push(
                            Wallpaper {
                                name: dir.file_name().unwrap().to_str().unwrap().to_string(),
                                path: path, detail: data
                            }
                        );
                        return;
                    };
                };
                *error.borrow_mut() = format!("{}の読み込みに失敗しました。", path);
            };
        }
    )?;
    if !error.borrow().is_empty() { return Err(error.into_inner()) };
    Ok(wallpapers.into_inner())
}


/// DataManagerの実装です。
/// もしデータが存在しない場合は壁紙プロファイルと拡張機能以外なら新規作成をします。
impl DataManager {
    pub fn new() -> Result<Self, String> {
        let path = &add_setting_path("");
        if !Path::new(&path).exists() {
            create_dir(path).unwrap_or_else(
                |_| {
                    error(&format!("設定フォルダの作成に失敗しました。\nPath:{}", path));
                    panic!("Failed to mkdir setting folder");
                }
            );
            // 初回起動時の場合は必要なファイルとフォルダ等を準備する。
            for (file_name, default) in vec![
                ("data.json", DATA_DEFAULT), ("wallpapers", "_dir_"), ("extensions", "_dir_")
            ] {
                let mut error = String::new();
                let path = add_setting_path(file_name);
                if !Path::new(&path).exists() {
                    // もし必要なファイルまたはフォルダがまだないのなら新しく作る。
                    if default == "_dir_" {
                        create_dir(&path).unwrap_or_else(|_| error = format!("Path:{}", path));
                    } else {
                        write(&path, default.to_string())
                            .unwrap_or_else(
                                |detail| error = format!("{}の作成に失敗しました。\n{}", file_name, detail)
                            );
                    };
                };
                if !error.is_empty() { return Err(error); };
            };
        };
        Ok(DataManager { general: read_setting()?, wallpapers: read_wallpapers()? })
    }

    /// 設定を読み込みます。
    pub fn read_setting(&mut self) -> Result<&GeneralSetting, String> {
        self.general = read_setting()?;
        Ok(&self.general)
    }

    /// 設定を書き込みます。
    pub fn write_setting(&self) -> Result<(), String> {
        write(
            &add_setting_path("data.json"), to_string_pretty(&self.general)
                .expect(FAILED_JSON)
        )
    }

    /// 壁紙の設定を読み込みます。
    pub fn read_wallpapers(&mut self) -> Result<&Wallpapers, String> {
        self.wallpapers = read_wallpapers()?;
        Ok(&self.wallpapers)
    }

    /// 壁紙の設定を更新します。
    pub fn write_wallpaper(&self, index: usize) -> Result<(), String> {
        match self.wallpapers.get(index) {
            Some(wallpaper) => write(
                &format!("{}/data.json", wallpaper.path), to_string_pretty(&wallpaper.detail)
                    .expect(FAILED_JSON)
            ),
            _ => Err("壁紙が見つかりませんでした。".to_string())
        }
    }

    /// 壁紙の設定を取得します。
    pub fn get_wallpaper(&self, name: &str) -> Option<Wallpaper> {
        for wallpaper in self.wallpapers.iter() {
            if wallpaper.name == name {
                return Some(wallpaper.clone());
            };
        };
        None
    }
}