//! FreedomWall - Build


fn main_of_main() {
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