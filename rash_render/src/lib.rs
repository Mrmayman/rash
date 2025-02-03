use std::{collections::HashMap, sync::Arc};

use renderer::{buffers::GraphicsState, InnerRenderer};
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

mod renderer;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct SpriteId(pub i64);

pub struct SpriteData {
    graphics: GraphicsState,
    id: SpriteId,
}

pub struct Renderer<'a> {
    renderer: InnerRenderer<'a>,
    window: Arc<Window>,
    event_loop: EventLoop<()>,
    runnable: Box<dyn Run>,
    graphics: RunState,
}

impl Renderer<'_> {
    pub async fn new(title: &str, runnable: Box<dyn Run>) -> Self {
        let event_loop = EventLoop::new().unwrap();
        let window = Arc::new(
            WindowBuilder::new()
                .with_title(title)
                .build(&event_loop)
                .unwrap(),
        );

        let num_sprites = runnable.num_sprites();
        let renderer = InnerRenderer::new(window.clone(), num_sprites).await;

        let mut graphics = HashMap::new();
        for i in 0..(num_sprites as i64) {
            graphics.insert(
                SpriteId(i),
                SpriteData {
                    id: SpriteId(i),
                    graphics: GraphicsState {
                        x: 0.0,
                        y: 0.0,
                        texture_width: 100.0,
                        texture_height: 100.0,
                        size: 100.0,
                        _padding: Default::default(),
                    },
                },
            );
        }

        Self {
            renderer,
            window,
            event_loop,
            runnable,
            graphics: RunState { graphics },
        }
    }

    pub fn run(mut self) {
        self.event_loop
            .run(move |event, control_flow| match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == self.renderer.window.id() => match event {
                    WindowEvent::RedrawRequested => {
                        let exited = self.runnable.update(&mut self.graphics);

                        let sprite_order = self.runnable.sprite_order();
                        let mut graphics_state: Vec<(&SpriteId, &SpriteData)> =
                            self.graphics.graphics.iter().collect();
                        graphics_state.sort_by(|(id1, _), (id2, _)| id1.0.cmp(&id2.0));

                        let graphics_state: Vec<GraphicsState> = graphics_state
                            .into_iter()
                            .map(|(_, data)| data.graphics.clone())
                            .collect();

                        self.renderer
                            .tick(control_flow, &graphics_state, &sprite_order);
                    }
                    WindowEvent::Resized(physical_size) => {
                        self.renderer.resize(*physical_size);
                    }
                    WindowEvent::CloseRequested => control_flow.exit(),
                    _ => {}
                },
                _ => {}
            })
            .unwrap();
    }
}

pub trait Run {
    fn update(&mut self, state: &mut RunState) -> bool;
    fn num_sprites(&self) -> usize;
    fn sprite_order(&self) -> Vec<SpriteId>;
}

pub struct RunState {
    graphics: HashMap<SpriteId, SpriteData>,
}

impl RunState {
    pub unsafe extern "C" fn c_go_to(this: *mut Self, id: i64, x: f64, y: f64) {
        println!("Going to {x}, {y}");
        (unsafe { &mut *this }).go_to(SpriteId(id), x as f32, y as f32);
    }

    pub fn go_to(&mut self, id: SpriteId, x: f32, y: f32) {
        let state = self.graphics.get_mut(&id).unwrap();
        state.graphics.x = x;
        state.graphics.y = y;
    }

    pub fn set_x(&mut self, id: SpriteId, x: f32) {
        let state = self.graphics.get_mut(&id).unwrap();
        state.graphics.x = x;
    }

    pub fn set_y(&mut self, id: SpriteId, y: f32) {
        let state = self.graphics.get_mut(&id).unwrap();
        state.graphics.y = y;
    }

    pub fn get_x(&mut self, id: SpriteId) -> f32 {
        let state = self.graphics.get_mut(&id).unwrap();
        state.graphics.x
    }

    pub fn get_y(&mut self, id: SpriteId) -> f32 {
        let state = self.graphics.get_mut(&id).unwrap();
        state.graphics.y
    }

    pub fn change_x(&mut self, id: SpriteId, x: f32) {
        let state = self.graphics.get_mut(&id).unwrap();
        state.graphics.x += x;
    }

    pub fn change_y(&mut self, id: SpriteId, y: f32) {
        let state = self.graphics.get_mut(&id).unwrap();
        state.graphics.y += y;
    }
}

// Code taken from tutorial: https://sotrh.github.io/learn-wgpu/
