use std::{
    fs::{read, File},
    io::Write,
};

use tera::{Context, Tera};

#[cfg(target_os = "windows")]
use tauri_winres::WindowsResource;

fn main_of_main() {
    // HTMLのレンダリングを行う。
    let tera = match Tera::new("pages/*.html") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };
    for name in [
        "home.html",
        "setting.html",
        "wallpapers.html",
        "extensions.html",
        "credit.html",
    ] {
        let mut f = File::create(format!("pages/_{}", name)).unwrap();
        f.write_all(tera.render(name, &Context::new()).unwrap().as_bytes())
            .unwrap();
    }
    let mut f = File::create("pages/freedomwall/utils.js").unwrap();
    f.write_all(
        String::from_utf8_lossy(&read("pages/freedomwall/_utils.js").unwrap())
            .to_string()
            .replace(
                "__SCHEME__",
                if cfg!(target_os = "windows") {
                    "https://wry."
                } else {
                    "wry://"
                },
            )
            .as_bytes(),
    )
    .unwrap();
}

#[cfg(target_os = "macos")]
fn main() {
    // ウィンドウ名のCFStringをRustのStringに変えるのに使うライブラリをリンクしておく。
    println!("cargo:rustc-link-lib=framework=CoreFoundation");
    main_of_main();
}

#[cfg(target_os = "windows")]
fn main() {
    // アプリのアイコン等を設定する。
    let mut res = WindowsResource::new();
    res.set_icon("logo/FreedomWall.ico");
    res.compile().unwrap();
    main_of_main();
}
