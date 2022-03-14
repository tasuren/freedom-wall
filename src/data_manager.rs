//! FreedomWall - DataManager

use std::{ collections::HashMap, env::args};

use serde::{ Serialize, Deserialize };
use serde_json::{ to_string, from_str };
use platform_dirs::AppDirs;

use super::APPLICATION_NAME;


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
    format!(
        "{}{}",
        AppDirs::new(Some(""), false)
            .expect("設定を保存する場所のパスが見つかりませんでした。")
            .data_dir.to_str()
            .expect("設定を保存する場所のパスの処理中になんらかのエラーが発生しました。"),
        ""
    )
}


/// 壁紙プロファイルの設定ファイルである`data.json`の構造体です。
#[derive(Serialize, Deserialize)]
struct WallpaperJson {
    author: String,
    description: String,
    alpha: f32,
    target: Vec<String>,
    exception: Vec<String>,
    setting: HashMap<String, String>
}


/// 壁紙の設定データの構造体です。
pub struct Wallpaper {
    name: String,
    path: String,
    detail: WallpaperJson
}


/// FreedomWallの設定ファイルの構造体です。
#[derive(Serialize, Deserialize)]
pub struct GeneralSetting {
    language: String,
    wallpapers: HashMap<String, Vec<String>>,
    updateInterval: f32
}


/// セーブデータを管理するための構造体です。
type Wallpapers = HashMap<String, WallpaperJson>;
struct DataManager {
    general: GeneralSetting,
    wallpapers: Wallpapers
}


/// DataManagerの実装です。
/// もしデータが存在しない場合は壁紙プロファイルと拡張機能以外なら新規作成をします。
impl DataManager {
    /// 壁紙のデータを読み込みます。
    fn read_wallpapers(&self) -> Wallpapers {
        let mut wallpapers = HashMap::new();
        wallpapers
    }
}