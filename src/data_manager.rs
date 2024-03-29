use std::{
    cell::RefCell,
    collections::HashMap,
    env::args,
    ffi::OsStr,
    fs::{copy, create_dir, read_to_string, remove_dir_all, rename, File},
    io::Write,
    path::{Path, PathBuf},
};

use smallvec::SmallVec;

use platform_dirs::AppDirs;
use rust_i18n::t;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string_pretty};

use super::APPLICATION_NAME;

#[cfg(target_os = "macos")]
use super::platform::macos::get_bundle_path;

const FAILED_JSON: &str = "JSON生成時にエラーが発生しました。";
const DATA_DEFAULT: &str = r#"{
    "language": "ja", "wallpapers": [], "update_interval": 0.05, "dev": false
}"#;

/// ベースパスを取得します。
/// 普通は`./`を返します。
/// Macの場合はアプリに(Bundleに)した場合、カレントディレクトリが`/`になってしまうので、ビルド後のMac用アプリの場合は絶対パスが返されます。
pub fn get_base() -> String {
    #[cfg(target_os = "windows")]
    return "./".to_string();
    #[cfg(target_os = "macos")]
    return if cfg!(debug_assertions) {
        "./".to_string()
    } else {
        format!("{}/Contents/Resources", get_bundle_path())
    };
}

/// パスに`add_base`で取得したパスを最初に付けます。
pub fn add_base(path: &str) -> String {
    format!("{}/{}", get_base(), path)
}

//  壁紙設定
/// アプリ名を取得します。
fn get_application_name() -> String {
    let data: Vec<String> = args().collect();
    if data.len() == 1 {
        APPLICATION_NAME.to_string()
    } else if data[1] == "test" {
        format!("{}Dev", APPLICATION_NAME)
    } else {
        APPLICATION_NAME.to_string()
    }
}

/// 渡されたパスに設定ファイルを保存する場所のパスを追加します。
pub fn add_setting_path(path: &str) -> Result<String, String> {
    match AppDirs::new(Some(&get_application_name()), false) {
        Some(dir) => {
            let data_dir = dir.data_dir;
            match data_dir.to_str() {
                Some(app_dir) => Ok(if app_dir.is_empty() {
                    app_dir.to_string()
                } else {
                    format!("{}/{}", app_dir, path)
                }),
                _ => Err(t!("core.general.getSettingPathFailed")),
            }
        }
        _ => Err(t!("core.general.settingPathNotFound")),
    }
}

/// 壁紙プロファイルの設定ファイルである`data.json`の構造体です。
#[derive(Serialize, Deserialize, Clone)]
pub struct WallpaperJson {
    pub author: String,
    pub description: String,
    pub setting: HashMap<String, String>,
    pub force_size: bool,
}

/// 壁紙の設定データの構造体です。
#[derive(Serialize, Deserialize, Clone)]
pub struct Wallpaper {
    pub name: String,
    pub path: String,
    pub detail: WallpaperJson,
}

/// 壁紙ウィンドウのサイズ調整用のデータです。
#[derive(Serialize, Deserialize, Clone)]
pub struct Shift {
    pub up: i32,
    pub down: i32,
    pub left: i32,
    pub right: i32,
}

/// 背景対象となるウィンドウのデータの構造体です。
#[derive(Serialize, Deserialize)]
pub struct Target {
    pub targets: SmallVec<[String; 4]>,
    pub exceptions: SmallVec<[String; 3]>,
    pub alpha: f64,
    pub wallpaper: String,
    pub shift: Shift,
}

/// FreedomWallの設定ファイルの構造体です。
#[derive(Serialize, Deserialize)]
pub struct GeneralSetting {
    pub language: String,
    pub wallpapers: SmallVec<[Target; 5]>,
    pub update_interval: f32,
    pub dev: bool,
}

/// 拡張機能のJSONデータの構造体です。
#[derive(Serialize, Deserialize, Clone)]
pub struct ExtensionJson {
    pub description: String,
    pub author: String,
    pub version: String,
    pub setting: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Extension {
    pub name: String,
    pub path: String,
    pub detail: ExtensionJson,
}

/// セーブデータを管理するための構造体です。
type Wallpapers = Vec<Wallpaper>;
type Templates = Vec<String>;
type Extensions = Vec<Extension>;
pub struct DataManager {
    pub general: GeneralSetting,
    pub wallpapers: Wallpapers,
    pub templates: Templates,
    pub extensions: Extensions,
}

fn failed_read(path: String) -> String {
    t!("core.general.failedRead", path = &path)
}

/// 指定されたパスにあるファイル,フォルダのVecを取得します。
fn get_files(target: &PathBuf, dir: bool) -> Option<Vec<PathBuf>> {
    match target.read_dir() {
        Ok(entries) => {
            let mut result = Vec::<PathBuf>::new();

            for tentative in entries {
                if tentative.is_ok() {
                    let entry = tentative.unwrap().path();
                    if !dir || entry.is_dir() {
                        result.push(entry);
                    };
                } else {
                    println!("Error on getting files: {:?}", tentative.err());
                    return None;
                };
            }
            Some(result)
        }
        _ => None,
    }
}

/// 渡されたPathBufから名前を取得します。
fn get_name<'a>(path: &'a PathBuf) -> &'a str {
    path.file_name()
        .unwrap_or_else(|| &OsStr::new(""))
        .to_str()
        .unwrap()
}

/// 指定されたパスのフォルダにあるすべてのフォルダからファイル検索を行います。
/// また、on_found引数でファイルの読み込み処理等も行うこともできます。
/// on_foundに渡されるものは左から順にフォルダのパス,フォルダのPathBuf,ファイル名,ファイルのパス
fn search_files<const N: usize, F: Fn(String, &PathBuf, &str, &String) -> ()>(
    path: &str,
    targets: [&str; N],
    on_found: F,
    do_add_setting_path: bool,
) -> Result<(), String> {
    if let Some(dirs) = get_files(
        &PathBuf::from(if do_add_setting_path {
            add_setting_path(path)?
        } else {
            path.to_string()
        }),
        true,
    ) {
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
                            &path,
                            file_name,
                            &file_path,
                        );
                    };
                }

                // もし指定されたファイル等を全て見つけられなかったのならエラーとする。
                if ok.len() < targets.len() {
                    return Err(t!(
                        "core.general.wallpaperNotFound",
                        place = path_string,
                        targets = &targets
                            .iter()
                            .filter(|x| !ok.contains(x))
                            .map(|x| x.to_string())
                            .collect::<Vec<String>>()
                            .join("`, `")
                    ));
                };
            } else {
                return Err(failed_read(path_string.to_string()));
            };
        }
        Ok(())
    } else {
        Err(failed_read(path.to_string()))
    }
}

/// 指定されたパスにあるファイルの文字列を全て読み込みます。
fn read(path: &String) -> Result<String, String> {
    if let Ok(raw) = read_to_string(path) {
        Ok(raw)
    } else {
        Err(failed_read(path.to_string()))
    }
}

/// 指定されたファイルが存在するか確認をします。
fn exists(path: &str) -> Result<(), String> {
    if !PathBuf::from(path).exists() {
        Err(failed_read(path.to_string()))
    } else {
        Ok(())
    }
}

/// 指定されたパスに指定された文字列をすべて書き込みます。
fn write(path: &str, data: &str) -> Result<(), String> {
    if let Ok(tentative) = File::create(path) {
        let mut f = tentative;

        if f.write_all(data.as_bytes()).is_err() {
            return Err(t!("core.general.failedWrite", path = path));
        };
    };

    Ok(())
}

/// 設定を読み込みます。
fn read_setting() -> Result<GeneralSetting, String> {
    let path = add_setting_path("data.json")?;

    exists(&path)?;
    let mut raw = read(&path)?;

    // 後方互換のための文字列交換。
    if raw.contains("\"updateInterval\"") {
        raw = raw.replace("\"updateInterval\"", "\"update_interval\"");
        write(&path, &raw)?;
    };

    match from_str::<GeneralSetting>(&raw) {
        Ok(data) => Ok(data),
        Err(e) => Err(format!("{}\nCode: {}", failed_read(path), e)),
    }
}

/// 壁紙の設定を読み込みます。
fn read_wallpapers() -> Result<Wallpapers, String> {
    let error = RefCell::new(String::new());
    let wallpapers = RefCell::new(Vec::new());

    search_files(
        "wallpapers",
        ["index.html", "data.json"],
        |path, dir, file_name, file_path| {
            if file_name == "data.json" {
                if let Ok(mut raw) = read(&file_path) {
                    // 後方互換 for 2.0.0
                    if raw.contains("\"forceSize\"") {
                        raw = raw.replace("\"forceSize\"", "\"force_size\"");
                        if let Err(e) = write(file_path, &raw) {
                            *error.borrow_mut() = e;
                        };
                    };

                    if let Ok(data) = from_str::<WallpaperJson>(&raw) {
                        wallpapers.borrow_mut().push(Wallpaper {
                            name: get_name(dir).to_string(),
                            path: path,
                            detail: data,
                        });

                        return;
                    };
                };

                *error.borrow_mut() = failed_read(path);
            };
        },
        true,
    )?;

    if !error.borrow().is_empty() {
        println!("a");
        return Err(error.into_inner());
    };

    Ok(wallpapers.into_inner())
}

/// テンプレートを読み込みます。
fn read_templates() -> Result<Templates, String> {
    let templates = add_base("templates");

    search_files(
        &templates,
        ["index.html", "data.json"],
        |_, _, _, _| {},
        false,
    )?;

    if let Some(data) = get_files(&PathBuf::from(&templates), true) {
        let mut result = Vec::new();
        for path in data {
            result.push(get_name(&path).to_string());
        }
        Ok(result)
    } else {
        Err(failed_read(templates))
    }
}

/// 拡張機能を読み込みます。
fn read_extensions() -> Result<Extensions, String> {
    let error = RefCell::new(String::new());
    let extensions = RefCell::new(Vec::new());

    search_files(
        "extensions",
        ["init.js", "data.json"],
        |path, dir, file_name, file_path| {
            if file_name == "data.json" {
                if let Ok(raw) = read(&file_path) {
                    if let Ok(data) = from_str::<ExtensionJson>(&raw) {
                        extensions.borrow_mut().push(Extension {
                            name: get_name(dir).to_string(),
                            path: path,
                            detail: data,
                        });
                        return;
                    };
                };
                *error.borrow_mut() = failed_read(path);
            };
        },
        true,
    )?;

    if !error.borrow().is_empty() {
        return Err(error.into_inner());
    };

    Ok(extensions.into_inner())
}

/// DataManagerの実装です。
/// もしデータが存在しない場合は壁紙プロファイルと拡張機能以外なら新規作成をします。
impl DataManager {
    pub fn new() -> Result<Self, String> {
        println!(
            "Setting Path: \"{}\"",
            AppDirs::new(Some(&get_application_name()), false)
                .unwrap()
                .data_dir
                .display()
        );

        let path = &add_setting_path("")?;

        if !Path::new(&path).exists() {
            if create_dir(path).is_err() {
                return Err(t!("core.general.createSettingFolderFailed", path = path));
            };

            // 初回起動時の場合は必要なファイルとフォルダ等を準備する。
            for (file_name, default) in [
                ("data.json", DATA_DEFAULT),
                ("wallpapers", "_dir_"),
                ("extensions", "_dir_"),
            ] {
                let mut error = String::new();
                let path = add_setting_path(file_name)?;

                if !Path::new(&path).exists() {
                    // もし必要なファイルまたはフォルダがまだないのなら新しく作る。
                    if default == "_dir_" {
                        create_dir(&path).unwrap_or_else(|_| error = format!("Path:{}", path));
                    } else {
                        write(&path, default).unwrap_or_else(|_| {
                            error = t!("core.general.failedWrite", path = file_name)
                        });
                    };
                };

                if !error.is_empty() {
                    return Err(error);
                };
            }
        };
        Ok(DataManager {
            general: read_setting()?,
            wallpapers: read_wallpapers()?,
            templates: read_templates()?,
            extensions: read_extensions()?,
        })
    }

    /* 将来性を考慮して削除ではなくコメントアウト
    /// テンプレート情報を取得します。
    pub fn read_templates(&mut self) -> Result<&Templates, String> {
        self.templates = read_templates()?;
        Ok(&self.templates)
    }

    /// 設定を読み込みます。
    pub fn read_setting(&mut self) -> Result<&GeneralSetting, String> {
        self.general = read_setting()?;
        Ok(&self.general)
    }
    */

    /// 設定を書き込みます。
    pub fn write_setting(&self) -> Result<(), String> {
        write(
            &add_setting_path("data.json")?,
            &to_string_pretty(&self.general).expect(FAILED_JSON),
        )
    }

    /* 将来性を考慮して削除ではなくコメントアウト
    /// 壁紙の設定を読み込みます。
    pub fn read_wallpapers(&mut self) -> Result<&Wallpapers, String> {
        self.wallpapers = read_wallpapers()?;
        Ok(&self.wallpapers)
    }
    */

    /// 拡張機能を読み込みます。
    pub fn read_extensions(&mut self) -> Result<&Extensions, String> {
        self.extensions = read_extensions()?;
        Ok(&self.extensions)
    }

    /// 拡張機能のデータを取得します。
    pub fn get_extension(&self, name: &str) -> Option<(usize, &Extension)> {
        for (index, extension) in self.extensions.iter().enumerate() {
            if &extension.name == name {
                return Some((index, extension));
            };
        }
        None
    }

    /// 拡張機能の設定を書き込みます。
    pub fn write_extension(&self, name: String) -> Result<(), String> {
        if let Some((_, extension)) = self.get_extension(&name) {
            write(
                &format!("{}/data.json", extension.path),
                &to_string_pretty(&extension.detail).expect(FAILED_JSON),
            )?;
            Ok(())
        } else {
            Err(t!("core.general.failedRead", path = &name))
        }
    }

    /// インデックス番号から壁紙プロファイルを取得します。
    pub fn get_wallpaper_by_index(&self, index: usize) -> Result<&Wallpaper, String> {
        match self.wallpapers.get(index) {
            Some(wallpaper) => Ok(wallpaper),
            _ => Err(t!("core.general.searchWallpaperFailed")),
        }
    }

    /// 壁紙プロファイルの設定を更新します。
    pub fn write_wallpaper(&self, index: usize) -> Result<(), String> {
        let wallpaper = self.get_wallpaper_by_index(index)?;
        write(
            &format!("{}/data.json", wallpaper.path),
            &to_string_pretty(&wallpaper.detail).expect(FAILED_JSON),
        )?;
        Ok(())
    }

    /// 壁紙プロファイルを削除します。
    pub fn remove_wallpaper(&mut self, index: usize) -> Result<(), String> {
        let wallpaper = self.get_wallpaper_by_index(index)?;
        match remove_dir_all(wallpaper.path.clone()) {
            Ok(_) => {
                self.wallpapers.remove(index);
                Ok(())
            }
            _ => Err(format!(
                "{}\nDetail: {}",
                t!("core.general.removeDirFailed"),
                wallpaper.path
            )),
        }
    }

    /// テンプレートから壁紙プロファイルを追加して書き込みをします。
    pub fn add_wallpaper(&mut self, template: String, name: String) -> Result<(), String> {
        if self.templates.contains(&template) {
            if self.get_wallpaper(&name).is_none() {
                let path = add_setting_path(&format!("wallpapers/{}", name))?;
                // フォルダの作る。
                match create_dir(&path) {
                    Ok(_) => {
                        let original_path = &format!("{}/{}", add_base("templates"), template);
                        // ファイルのコピーを行う。
                        for filename in ["index.html", "data.json"] {
                            if let Err(_) = copy(
                                format!("{}/{}", original_path, filename),
                                format!("{}/{}", path, filename),
                            ) {
                                return Err(t!("core.general.copyFailed", path = &name));
                            };
                        }
                        if let Ok(wallpaper) =
                            from_str::<WallpaperJson>(&read(&format!("{}/{}", path, "data.json"))?)
                        {
                            self.wallpapers.push(Wallpaper {
                                name: name,
                                path: path,
                                detail: wallpaper,
                            });
                            Ok(())
                        } else {
                            Err(t!("core.general.failedRead", path = "data.json"))
                        }
                    }
                    _ => Err(t!("core.general.mkdirFailed", path = &path)),
                }
            } else {
                Err(t!("core.general.alreadyAdded", name = &name))
            }
        } else {
            Err(t!("core.general.failedRead", path = &template))
        }
    }

    /// 壁紙の設定のインデックス番号を取得します。
    pub fn get_wallpaper_index(&self, name: &str) -> Option<usize> {
        for (index, wallpaper) in self.wallpapers.iter().enumerate() {
            if wallpaper.name == name {
                return Some(index);
            };
        }
        None
    }

    /// 壁紙の設定を取得します。
    pub fn get_wallpaper(&self, name: &str) -> Option<Wallpaper> {
        for wallpaper in self.wallpapers.iter() {
            if wallpaper.name == name {
                return Some(wallpaper.clone());
            };
        }
        None
    }

    /// 壁紙の名前を変更します。
    pub fn mv_wallpaper(&mut self, before: &str, after: &str) -> Result<(), String> {
        if self.get_wallpaper(before).is_some() {
            for wallpaper in self.wallpapers.iter_mut() {
                if wallpaper.name == before {
                    wallpaper.name = after.to_string();
                    let path = add_setting_path(&format!("wallpapers/{}", after))?;
                    if rename(wallpaper.path.to_string(), &path).is_err() {
                        return Err(t!("core.general.renameFailed"));
                    };
                    wallpaper.path = path;
                };
            }
            Ok(())
        } else {
            Err(t!(
                "core.general.findAppropriateWallpaperFailed",
                name = before
            ))
        }
    }

    /// 壁紙設定を取得ます。
    pub fn get_wallpaper_setting(&self, name: &str) -> Vec<(usize, &Target)> {
        let mut data = Vec::new();
        for (index, target) in self.general.wallpapers.iter().enumerate() {
            if target.wallpaper == name {
                data.push((index, target));
            };
        }
        data
    }
}
