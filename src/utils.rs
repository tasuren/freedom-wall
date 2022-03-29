//! FreedomWall - Utils

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