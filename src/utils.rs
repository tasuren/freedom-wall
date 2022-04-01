//! FreedomWall - Utils

use std::process::Command;

use rfd::{ MessageDialog, MessageButtons, MessageLevel };

use super::APPLICATION_NAME;


/// ダイアログを表示します。
pub fn dialog(message: &str, level: MessageLevel, button: MessageButtons) {
    MessageDialog::new()
        .set_title(APPLICATION_NAME)
        .set_description(message)
        .set_buttons(button)
        .set_level(level)
        .show();
}


/// エラーを表示します。
pub fn error(message: &str) {
    dialog(message, MessageLevel::Error, MessageButtons::Ok);
}


/// フォルダを開きます。
pub fn open_folder(path: String) {
    if cfg!(target_os="macos") {
        Command::new("open")
            .arg(path).status().unwrap();
    } else if cfg!(target_os="windows") {
        Command::new("explorer")
            .arg(path.replace("/", "\\"))
            .status().unwrap();
    };
}