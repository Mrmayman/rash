use std::{collections::HashMap, rc::Rc};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct SpriteId(pub i64);

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default)]
pub struct CostumeId(pub i32);

/// The global state of the VM at runtime.
#[derive(Debug, Clone, Default)]
pub struct RunState {
    pub sprites: HashMap<SpriteId, SpriteData>,
}

impl RunState {
    // TODO: Implement Pen trails

    /// # Safety
    /// `this` must point to a valid instance of `RunState`
    pub unsafe extern "C" fn c_go_to(this: *mut Self, id: SpriteId, x: f64, y: f64) {
        debug_assert!(!this.is_null());
        (unsafe { &mut *this }).go_to(id, x as f32, y as f32);
    }

    pub fn go_to(&mut self, id: SpriteId, x: f32, y: f32) {
        let state = self.sprites.get_mut(&id).unwrap();
        state.graphics.x = x;
        state.graphics.y = y;
    }

    /// # Safety
    /// `this` must point to a valid instance of `RunState`
    pub unsafe extern "C" fn c_set_x(this: *mut Self, id: SpriteId, x: f64) {
        debug_assert!(!this.is_null());
        (unsafe { &mut *this }).set_x(id, x as f32);
    }

    pub fn set_x(&mut self, id: SpriteId, x: f32) {
        let state = self.sprites.get_mut(&id).unwrap();
        state.graphics.x = x;
    }

    /// # Safety
    /// `this` must point to a valid instance of `RunState`
    pub unsafe extern "C" fn c_set_y(this: *mut Self, id: SpriteId, y: f64) {
        debug_assert!(!this.is_null());
        (unsafe { &mut *this }).set_y(id, y as f32);
    }

    pub fn set_y(&mut self, id: SpriteId, y: f32) {
        let state = self.sprites.get_mut(&id).unwrap();
        state.graphics.y = y;
    }

    /// # Safety
    /// `this` must point to a valid instance of `RunState`
    pub unsafe extern "C" fn c_get_x(this: *mut Self, id: SpriteId) -> f64 {
        debug_assert!(!this.is_null());
        (unsafe { &mut *this }).get_x(id) as f64
    }

    /// # Safety
    /// `this` must point to a valid instance of `RunState`
    pub unsafe extern "C" fn c_get_y(this: *mut Self, id: SpriteId) -> f64 {
        debug_assert!(!this.is_null());
        (unsafe { &mut *this }).get_y(id) as f64
    }

    pub fn get_x(&mut self, id: SpriteId) -> f32 {
        let state = self.sprites.get_mut(&id).unwrap();
        state.graphics.x
    }

    pub fn get_y(&mut self, id: SpriteId) -> f32 {
        let state = self.sprites.get_mut(&id).unwrap();
        state.graphics.y
    }

    /// # Safety
    /// `this` must point to a valid instance of `RunState`
    pub unsafe extern "C" fn c_change_x(this: *mut Self, id: SpriteId, x: f64) {
        debug_assert!(!this.is_null());
        (unsafe { &mut *this }).change_x(id, x as f32);
    }

    /// # Safety
    /// `this` must point to a valid instance of `RunState`
    pub unsafe extern "C" fn c_change_y(this: *mut Self, id: SpriteId, y: f64) {
        debug_assert!(!this.is_null());
        (unsafe { &mut *this }).change_y(id, y as f32);
    }

    pub fn change_x(&mut self, id: SpriteId, x: f32) {
        let state = self.sprites.get_mut(&id).unwrap();
        state.graphics.x += x;
    }

    pub fn change_y(&mut self, id: SpriteId, y: f32) {
        let state = self.sprites.get_mut(&id).unwrap();
        state.graphics.y += y;
    }

    pub fn shown(&mut self, id: SpriteId, shown: bool) {
        let state = self.sprites.get_mut(&id).unwrap();
        state.graphics.shown = shown as i32;
    }

    /// # Safety
    /// `this` must point to a valid instance of `RunState`
    pub unsafe extern "C" fn c_shown(this: *mut Self, id: SpriteId, shown: i64) {
        debug_assert!(!this.is_null());
        unsafe { &mut *this }.shown(id, shown == 1);
    }
}

const _E: () = {
    assert!(std::mem::size_of::<GraphicsState>() == 16 * 4);
};

// WARNING: If you change this,
// update the shader-side definition too in
// `crates/rash_render/src/shaders/common.wgsl`
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct GraphicsState {
    pub x: f32,
    pub y: f32,
    pub texture_width: f32,
    pub texture_height: f32,

    pub size: f32,
    pub current_costume: CostumeId,
    pub center_x: f32,
    pub center_y: f32,

    pub shown: i32,
    pub padding: [i32; 7],
}

impl Default for GraphicsState {
    fn default() -> Self {
        Self {
            x: 36.0,
            y: 28.0,
            size: 100.0,
            current_costume: CostumeId(0),
            texture_width: 100.0,
            texture_height: 100.0,
            center_x: 0.0,
            center_y: 0.0,
            shown: 1,
            padding: [0; _],
        }
    }
}

/// The global state of each sprite at runtime.
#[derive(Clone, Debug, Default)]
pub struct SpriteData {
    pub graphics: GraphicsState,
}

#[derive(Clone)]
pub struct CostumeData {
    pub bytes: Vec<u8>,
    pub name: String,
    pub hash: String,
    pub rotation_center_x: f64,
    pub rotation_center_y: f64,
    pub is_svg: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct SpriteLoadData {
    pub x: f64,
    pub y: f64,
    pub size: f64,
    pub costume: CostumeId,
    pub shown: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CostumeHash(Rc<str>);

impl CostumeHash {
    pub fn new(s: &str) -> Self {
        Self(Rc::from(s))
    }
}
