//! FreedomWall - Build

use std::{ fs::File, io::Write };

use tera::{ Tera, Context };


fn main_of_main() {
    // HTMLのレンダリングを行う。
    let tera = match Tera::new("pages/*.html") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };
    for name in vec!["home.html", "setting.html", "wallpapers.html", "extensions.html"] {
        let mut f = File::create(format!("pages/_{}", name)).unwrap();
        f.write_all(
            tera.render(name, &Context::new())
                .unwrap().as_bytes()
        ).unwrap();
    };
}


#[cfg(target_os="macos")]
fn main() {
    // ウィンドウ名のCFStringをRustのStringに変えるのに使うライブラリをリンクしておく。
    println!("cargo:rustc-link-lib=framework=CoreFoundation");
    main_of_main();
}


#[cfg(target_os="windows")]
fn main() {
    main_of_main();
}