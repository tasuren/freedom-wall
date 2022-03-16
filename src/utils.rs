//! FreedomWall - Utils

use native_dialog::{ MessageDialog, MessageType };

use super::APPLICATION_NAME;


/// ダイアログを表示します。
pub fn dialog(message: &str, message_type: MessageType) {
    MessageDialog::new()
        .set_type(message_type)
        .set_title(APPLICATION_NAME)
        .set_text(message)
        .show_alert()
        .unwrap();
}


/// エラーを表示します。
pub fn error(message: &str) {
    dialog(message, MessageType::Error);
}


/// 渡されたResultがエラーの場合はエラーダイアログを表示します。
pub fn handle_error<T>(e: Result<T, String>) {
    if let Err(message) = e {
        error(&message);
    };
}