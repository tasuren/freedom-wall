//! FreedomWall by tasuren

use std::fs::{ canonicalize, read };
use std::time::{ Instant, Duration };

use wry::{
    application::{
        event::{ Event, StartCause, WindowEvent },
        event_loop::{ ControlFlow, EventLoop },
        window::WindowBuilder
    },
    webview::WebViewBuilder, http::ResponseBuilder
};

mod window;
mod platform;
mod data_manager;

use window::{ Window, WindowTrait };


pub const VERSION: &str = "2.0.0a";
pub const APPLICATION_NAME: &str = "FreedomWall";


fn main() -> wry::Result<()> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Freedom Wall")
        .with_decorations(false)
        .build(&event_loop)?;
    let webview = WebViewBuilder::new(window)?
        .with_custom_protocol("wry".into(), move |request| {
            let path = request.uri().replace("wry://", "");
            ResponseBuilder::new()
                .mimetype("text/html")
                .body(read(canonicalize(&path)?)?)
        })
        .with_url("wry://src/main.html")?
        .build()?;

    let window_manager = Window::new(webview);
    let update_interval = Duration::from_secs_f32(0.1);

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::NewEvents(StartCause::Init) => {
                println!("FreedomWall {} by tasuren", VERSION);
                *control_flow = ControlFlow::WaitUntil(Instant::now() + update_interval);
            },
            Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                *control_flow = ControlFlow::WaitUntil(Instant::now() + update_interval);
                window_manager.process_position();
            },
            Event::WindowEvent {
                event: WindowEvent::CloseRequested, ..
            } => *control_flow = ControlFlow::Exit,
            _ => ()
        };
    });
}