//! FreedomWall by tasuren

#![allow(non_snake_case)]

use wry::{
    application::{
        event::{ Event, StartCause, WindowEvent },
        event_loop::{ ControlFlow, EventLoop }
    }
};

use rust_i18n::{ i18n, t };

mod window;
mod platform;
mod data_manager;
mod manager;
mod utils;

use manager::{ UserEvents, Manager };
use utils::error;


i18n!("locales/app");
pub const VERSION: &str = "2.0.0a";
pub const APPLICATION_NAME: &str = "FreedomWall";


fn main() {
    let event_loop: EventLoop<UserEvents> = EventLoop::with_user_event();
    let manager_option = Manager::new(&event_loop, event_loop.create_proxy());
    if let Err(message) = manager_option {
        let text = t!(&message);
        error(&text);
    } else {
        let mut manager = manager_option.unwrap();

        event_loop.run(move |event, event_loop_target, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::NewEvents(StartCause::Init) => {
                    println!("FreedomWall {} by tasuren", VERSION);
                },
                Event::UserEvent(UserEvents::PassedInterval()) => {
                    // 背景ウィンドウの場所を調整したりする。
                    if let Err(message) = manager.process_windows(&event_loop_target) {
                        println!("Error while process_windows: {}", message);
                        *control_flow = ControlFlow::Exit;
                    };
                },
                Event::UserEvent(UserEvents::FileSelected(path)) => {
                    // ファイルダイアログによりファイルが選択された場合はJavaScriptのコールバックを呼び出してWebViewにパスを渡す。
                    manager.setting.as_ref().unwrap().evaluate_script(&format!(
                        "window._fileSelected(`{}`);", path
                            .replace("`", "\\`").replace("\\", "\\\\")
                    )).unwrap();
                    manager.file_dialog = None;
                },
                Event::UserEvent(UserEvents::Request(request)) => {
                    // APIリクエストを処理する。ここでやらなければエラーが起きてしまう。理由は`manager.rs`にて記述済み。
                    let response = manager.on_request(&request.uri, request.body.clone());
                    let mut queues = manager.queues.borrow_mut();

                    // リクエストの結果の返信をキューに追加する。
                    queues.push(response);
                },
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested, ..
                } => {
                    manager.stop();
                    println!("Bye");
                    *control_flow = ControlFlow::Exit;
                },
                _ => ()
            };
        });
    };
}