//! FreedomWall by tasuren

use std::{
    cell::RefCell,
    time::{ Instant, Duration }
};

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


pub const VERSION: &str = "2.0.0a";
pub const APPLICATION_NAME: &str = "FreedomWall";


fn main() -> wry::Result<()> {
    let event_loop = EventLoop::new();
    let mut manager = Manager::new(event_loop.clone()).expect("設定読み込みに失敗しました。");
    let update_interval = Duration::from_secs_f32(manager.data.general.updateInterval);

    event_loop.run(|event, _, control_flow| {
        match event {
            Event::NewEvents(StartCause::Init) => {
                println!("FreedomWall {} by tasuren", VERSION);
                *control_flow = ControlFlow::WaitUntil(Instant::now() + update_interval);
            },
            Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                *control_flow = ControlFlow::WaitUntil(Instant::now() + update_interval);
                manager.process_windows();
            },
            Event::WindowEvent {
                event: WindowEvent::CloseRequested, ..
            } => *control_flow = ControlFlow::Exit,
            _ => ()
        };
    });
}