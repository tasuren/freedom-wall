#![cfg_attr(not(test), windows_subsystem = "windows")]
#![cfg_attr(test, windows_subsystem = "console")]

use wry::application::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use rust_i18n::i18n;

mod data_manager;
mod manager;
mod platform;
mod utils;
mod window;

use manager::{Manager, UserEvents};
use utils::{error, escape_for_js};

i18n!("src/locales");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APPLICATION_NAME: &str = "FreedomWall";

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn main() {
    let event_loop: EventLoop<UserEvents> = EventLoop::with_user_event();
    let manager_option = Manager::new(&event_loop, event_loop.create_proxy());

    if let Err(message) = manager_option {
        error(&message);
    } else {
        let mut manager = manager_option.unwrap();

        event_loop.run(move |event, event_loop_target, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::NewEvents(StartCause::Init) => {
                    println!("FreedomWall {} by tasuren", VERSION);
                }
                Event::UserEvent(UserEvents::PassedInterval()) => {
                    // 背景ウィンドウの場所を調整したりする。
                    if let Err(message) = manager.process_windows(&event_loop_target) {
                        println!("Error while process_windows: {}", message);
                        *control_flow = ControlFlow::Exit;
                    };
                }
                Event::UserEvent(UserEvents::FileSelected(path)) => {
                    // ファイルダイアログによりファイルが選択された場合はJavaScriptのコールバックを呼び出してWebViewにパスを渡す。
                    manager
                        .setting
                        .as_ref()
                        .unwrap()
                        .evaluate_script(&format!(
                            "window._fileSelected(`{}`);",
                            escape_for_js(path)
                        ))
                        .unwrap();
                    manager.file_dialog = None;
                }
                Event::UserEvent(UserEvents::Request(request)) => {
                    // APIリクエストを処理する。ここでやらなければエラーが起きてしまう。理由は`manager.rs`にて記述済み。
                    manager.on_request(&request.uri, request.body.clone());
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    manager.stop();
                    println!("Bye");
                    *control_flow = ControlFlow::Exit;
                }
                _ => (),
            };
        });
    };
}

#[cfg(target_os = "linux")]
fn main() {
    panic!("Linux is not supported now.");
}
