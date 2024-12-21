// Code taken from tutorial: https://sotrh.github.io/learn-wgpu/

use renderer::Renderer;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

mod renderer;

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("Rash")
        .build(&event_loop)
        .unwrap();

    let mut renderer = Renderer::new(&window).await;

    event_loop
        .run(move |event, control_flow| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == renderer.window.id() => match event {
                WindowEvent::RedrawRequested => {
                    renderer.tick(control_flow);
                }
                WindowEvent::Resized(physical_size) => {
                    renderer.resize(*physical_size);
                }
                WindowEvent::CloseRequested => control_flow.exit(),
                _ => {}
            },
            _ => {}
        })
        .unwrap();
}

pub fn to_bytes<T: ?Sized>(s: &T, size: usize) -> &[u8] {
    let ptr = s as *const T as *const u8;
    unsafe { std::slice::from_raw_parts(ptr, size) }
}
