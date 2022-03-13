//! FreedomWall - Build


#[cfg(target_os="macos")]
fn main() {
    // ウィンドウ名のCFStringをRustのStringに変えるのに使うライブラリをリンクしておく。
    println!("cargo:rustc-link-lib=framework=CoreFoundation");
}


#[cfg(target_os="windows")]
fn main() {}