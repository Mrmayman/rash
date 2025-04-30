use std::{collections::HashMap, sync::Arc};

use renderer::{buffers::GraphicsState, texture::Costume, InnerRenderer};
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

mod renderer;
pub use renderer::texture::IntermediateCostume;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct SpriteId(pub i64);

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default)]
pub struct CostumeId(pub i32);

#[derive(Clone, Debug, Default)]
pub struct SpriteData {
    graphics: GraphicsState,
}

pub struct Renderer<'a> {
    renderer: InnerRenderer<'a>,
    #[allow(unused)]
    window: Arc<Window>,
    event_loop: EventLoop<()>,
    runnable: Box<dyn Run>,
    costumes: HashMap<CostumeId, Costume>,
    graphics: RunState,
}

impl Renderer<'_> {
    pub async fn new(title: &str, runnable: Box<dyn Run>) -> Result<Self, image::ImageError> {
        let event_loop = EventLoop::new().unwrap();
        let window = Arc::new(
            WindowBuilder::new()
                .with_title(title)
                .build(&event_loop)
                .unwrap(),
        );

        let num_sprites = runnable.get_num_sprites();
        let renderer = InnerRenderer::new(window.clone(), num_sprites).await;

        let costumes: Result<HashMap<CostumeId, Costume>, image::ImageError> = runnable
            .get_costumes()
            .into_iter()
            .map(|(id, costume)| {
                Costume::from_bytes(
                    &costume,
                    &renderer.device,
                    &renderer.queue,
                    &renderer.sampler,
                    &renderer.costume_layout,
                )
                .map(|n| (id, n))
            })
            .collect();
        let costumes = costumes?;

        let init_state = runnable.get_state();
        let graphics = init_state
            .into_iter()
            .map(|(id, data)| {
                let costume_info = costumes.get(&data.costume).unwrap();
                let graphics = GraphicsState {
                    x: data.x as f32,
                    y: data.y as f32,
                    texture_width: costume_info.texture_width as f32,
                    texture_height: costume_info.texture_height as f32,
                    size: data.size as f32,
                    costume: data.costume,
                    center_x: costume_info.rotation_center_x as f32,
                    center_y: costume_info.rotation_center_y as f32,
                };
                (id, SpriteData { graphics })
            })
            .collect();

        Ok(Self {
            renderer,
            window,
            event_loop,
            runnable,
            graphics: RunState { graphics },
            costumes,
        })
    }

    pub fn run(mut self) {
        self.event_loop
            .run(move |event, control_flow| match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == self.renderer.window.id() => match event {
                    WindowEvent::RedrawRequested => {
                        let _exited = self.runnable.update(&mut self.graphics);

                        let sprite_order = self.runnable.get_sprite_order();
                        let mut graphics_state: Vec<(&SpriteId, &SpriteData)> =
                            self.graphics.graphics.iter().collect();
                        graphics_state.sort_by(|(id1, _), (id2, _)| id1.0.cmp(&id2.0));

                        let graphics_state: Vec<GraphicsState> = graphics_state
                            .into_iter()
                            .map(|(_, data)| data.graphics)
                            .collect();

                        self.renderer.tick(
                            control_flow,
                            &graphics_state,
                            &sprite_order,
                            &self.costumes,
                        );
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

#[derive(Debug, Clone, Copy)]
pub struct IntermediateState {
    pub x: f64,
    pub y: f64,
    pub size: f64,
    pub costume: CostumeId,
}

pub trait Run {
    fn update(&mut self, state: &mut RunState) -> bool;
    fn get_num_sprites(&self) -> usize;
    fn get_sprite_order(&self) -> Vec<SpriteId>;
    fn get_costumes(&self) -> HashMap<CostumeId, IntermediateCostume>;
    fn get_state(&self) -> HashMap<SpriteId, IntermediateState>;
}

#[derive(Debug, Clone, Default)]
pub struct RunState {
    graphics: HashMap<SpriteId, SpriteData>,
}

impl RunState {
    // TODO: Implement Pen trails

    /// # Safety
    /// `this` must point to a valid instance of `RunState`
    pub unsafe extern "C" fn c_go_to(this: *mut Self, id: i64, x: f64, y: f64) {
        assert!(!this.is_null());
        (unsafe { &mut *this }).go_to(SpriteId(id), x as f32, y as f32);
    }

    pub fn go_to(&mut self, id: SpriteId, x: f32, y: f32) {
        let state = self.graphics.get_mut(&id).unwrap();
        state.graphics.x = x;
        state.graphics.y = y;
    }

    /// # Safety
    /// `this` must point to a valid instance of `RunState`
    pub unsafe extern "C" fn c_set_x(this: *mut Self, id: i64, x: f64) {
        assert!(!this.is_null());
        (unsafe { &mut *this }).set_x(SpriteId(id), x as f32);
    }

    pub fn set_x(&mut self, id: SpriteId, x: f32) {
        let state = self.graphics.get_mut(&id).unwrap();
        state.graphics.x = x;
    }

    /// # Safety
    /// `this` must point to a valid instance of `RunState`
    pub unsafe extern "C" fn c_set_y(this: *mut Self, id: i64, y: f64) {
        assert!(!this.is_null());
        (unsafe { &mut *this }).set_y(SpriteId(id), y as f32);
    }

    pub fn set_y(&mut self, id: SpriteId, y: f32) {
        let state = self.graphics.get_mut(&id).unwrap();
        state.graphics.y = y;
    }

    /// # Safety
    /// `this` must point to a valid instance of `RunState`
    pub unsafe extern "C" fn c_get_x(this: *mut Self, id: i64) -> f64 {
        assert!(!this.is_null());
        (unsafe { &mut *this }).get_x(SpriteId(id)) as f64
    }

    /// # Safety
    /// `this` must point to a valid instance of `RunState`
    pub unsafe extern "C" fn c_get_y(this: *mut Self, id: i64) -> f64 {
        assert!(!this.is_null());
        (unsafe { &mut *this }).get_y(SpriteId(id)) as f64
    }

    pub fn get_x(&mut self, id: SpriteId) -> f32 {
        let state = self.graphics.get_mut(&id).unwrap();
        state.graphics.x
    }

    pub fn get_y(&mut self, id: SpriteId) -> f32 {
        let state = self.graphics.get_mut(&id).unwrap();
        state.graphics.y
    }

    /// # Safety
    /// `this` must point to a valid instance of `RunState`
    pub unsafe extern "C" fn c_change_x(this: *mut Self, id: i64, x: f64) {
        assert!(!this.is_null());
        (unsafe { &mut *this }).change_x(SpriteId(id), x as f32);
    }

    /// # Safety
    /// `this` must point to a valid instance of `RunState`
    pub unsafe extern "C" fn c_change_y(this: *mut Self, id: i64, y: f64) {
        assert!(!this.is_null());
        (unsafe { &mut *this }).change_y(SpriteId(id), y as f32);
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
