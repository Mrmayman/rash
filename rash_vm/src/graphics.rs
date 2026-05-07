use std::{collections::HashMap, rc::Rc};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub struct SpriteId(pub i64);

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default)]
pub struct CostumeId(pub i32);

#[derive(Debug, Clone, Default)]
pub struct RunState {
    pub graphics: HashMap<SpriteId, SpriteData>,
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

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct GraphicsState {
    pub x: f32,
    pub y: f32,
    pub texture_width: f32,
    pub texture_height: f32,
    pub size: f32,
    pub current_costume: CostumeId,
    pub center_x: f32,
    pub center_y: f32,
}

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
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CostumeHash(Rc<str>);

impl CostumeHash {
    pub fn new(s: &str) -> Self {
        Self(Rc::from(s))
    }
}
