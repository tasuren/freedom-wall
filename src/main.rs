//! FreedomWall by tasuren

use std::time::{ Instant, Duration };

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
        let update_interval = Duration::from_secs_f32(manager.data.general.updateInterval);

        event_loop.run(move |event, event_loop_target, control_flow| {
            match event {
                Event::NewEvents(StartCause::Init) => {
                    println!("FreedomWall {} by tasuren", VERSION);
                    *control_flow = ControlFlow::WaitUntil(Instant::now() + update_interval);
                },
                Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                    *control_flow = ControlFlow::WaitUntil(Instant::now() + update_interval);
                    // 背景ウィンドウの場所を調整したりする。
                    if let Err(message) = manager.process_windows(&event_loop_target) {
                        println!("Error while process_windows: {}", message);
                        *control_flow = ControlFlow::Exit;
                    };
                },
                Event::UserEvent(UserEvents::Request(request)) => {
                    let response = manager.on_request(&request.uri, request.body.clone());
                    let mut queues = manager.queues.borrow_mut();
                    queues.push(response);
                    // もしQueueが余っているのならメモリリークを防止するために消す。
                    if !queues.is_empty() { for i in 0..queues.len() { queues.remove(i); }; };
                },
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested, ..
                } => *control_flow = ControlFlow::Exit,
                _ => ()
            };
        });
    };
}