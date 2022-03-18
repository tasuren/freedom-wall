//! FreedomWall by tasuren

use std::time::{ Instant, Duration };

use wry::{
    application::{
        event::{ Event, StartCause, WindowEvent },
        event_loop::{ ControlFlow, EventLoop }
    }
};

mod window;
mod platform;
mod data_manager;
mod manager;
mod utils;

use manager::Manager;
use utils::error;


pub const VERSION: &str = "2.0.0a";
pub const APPLICATION_NAME: &str = "FreedomWall";


fn main() {
    let event_loop = EventLoop::new();
    let mut manager = Manager::new().expect("設定読み込みに失敗しました。");
    let update_interval = Duration::from_secs_f32(manager.data.general.updateInterval);

    event_loop.run(move |event, event_loop_target, control_flow| {
        match event {
            Event::NewEvents(StartCause::Init) => {
                println!("FreedomWall {} by tasuren", VERSION);
                *control_flow = ControlFlow::WaitUntil(Instant::now() + update_interval);
            },
            Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                *control_flow = ControlFlow::WaitUntil(Instant::now() + update_interval);
                if let Err(message) = manager.process_windows(&event_loop_target) {
                    println!("Error while process_windows: {}", message);
                    *control_flow = ControlFlow::Exit;
                };
            },
            Event::WindowEvent {
                event: WindowEvent::CloseRequested, ..
            } => *control_flow = ControlFlow::Exit,
            _ => ()
        };
    });
}