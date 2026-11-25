//! The launcher as a desktop app: one window, the OS's own webview, the page
//! from `ui/`.
//!
//! The interface is the HTML file the player can open in a browser too, so
//! there is one implementation of every screen, not two that drift. The window
//! is the OS's renderer (WebView2 on Windows, WebKit elsewhere), so the binary
//! stays a few megabytes: no Chromium, no Node, no `node_modules`.
//!
//! State lives in Rust. The page sends commands (`{"cmd": "...", ...}`) and
//! gets answers back as JSON; `bridge.rs` is the document the in-game mods read
//! from the same settings, so the HUD preview and the HUD in game agree.

mod bridge;
mod page;
mod state;

use std::borrow::Cow;

use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::WebViewBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("Alpinoxi Client")
        .with_inner_size(LogicalSize::new(1180.0, 760.0))
        .with_min_inner_size(LogicalSize::new(960.0, 620.0))
        .build(&event_loop)?;

    let mut state = state::LauncherState::load();

    // The whole page is inlined, so the window opens with no file next to the
    // exe to find, and no relative path that can break.
    let webview = WebViewBuilder::new(&window)
        .with_html(Cow::Borrowed(page::INDEX_HTML))
        .with_devtools(cfg!(debug_assertions))
        .with_ipc_handler(move |message: String| {
            // Every command goes through one door: the page can ask questions
            // and change settings, and never anything else.
            let _ = state.handle(&message);
        })
        .build()?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                state.save();
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                let _ = webview.set_bounds(wry::Rect {
                    position: wry::dpi::Position::Logical(wry::dpi::LogicalPosition::new(0.0, 0.0)),
                    size: wry::dpi::Size::Physical(
                        window.inner_size().to_physical(window.scale_factor()),
                    ),
                });
            }
            _ => (),
        }
    });
}
