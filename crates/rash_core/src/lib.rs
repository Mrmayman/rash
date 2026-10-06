//! Some core, shared types and utilities used across Rash.
//!
//! Split into separate crate to reduce compile times.

mod costumes;
pub use costumes::{CostumeStore, RawCostumeData};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct SpriteId(pub i64);

/// The raw VM ID of a costume.
///
/// Not necessarily in any meaningful Scratch-related order,
/// purely used for internal storage and access.
/// If you want to access by-index or by-name, see [`CostumeStore`].
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default)]
pub struct CostumeId(pub i32);

/// The global state of the VM at runtime.
#[derive(Debug, Clone, Default)]
pub struct RunState {
    pub sprites: Vec<(SpriteId, SpriteData)>,
}

impl RunState {
    pub fn new() -> Self {
        Self::default()
    }

    // TODO: Implement Pen trails
    pub fn go_to(&mut self, id: SpriteId, x: f32, y: f32) {
        let state = &mut self.sprites[id.0 as usize].1;
        state.graphics.x = x;
        state.graphics.y = y;
    }

    pub fn set_x(&mut self, id: SpriteId, x: f32) {
        let state = &mut self.sprites[id.0 as usize].1;
        state.graphics.x = x;
    }

    pub fn set_y(&mut self, id: SpriteId, y: f32) {
        let state = &mut self.sprites[id.0 as usize].1;
        state.graphics.y = y;
    }

    pub fn get_x(&mut self, id: SpriteId) -> f32 {
        let state = &mut self.sprites[id.0 as usize].1;
        state.graphics.x
    }

    pub fn get_y(&mut self, id: SpriteId) -> f32 {
        let state = &mut self.sprites[id.0 as usize].1;
        state.graphics.y
    }

    pub fn change_x(&mut self, id: SpriteId, x: f32) {
        let state = &mut self.sprites[id.0 as usize].1;
        state.graphics.x += x;
    }

    pub fn change_y(&mut self, id: SpriteId, y: f32) {
        let state = &mut self.sprites[id.0 as usize].1;
        state.graphics.y += y;
    }

    pub fn shown(&mut self, id: SpriteId, shown: bool) {
        let state = &mut self.sprites[id.0 as usize].1;
        state.graphics.shown = i32::from(shown);
    }
}

const _E: () = {
    assert!(std::mem::size_of::<ShaderState>() == 16 * 4);
};

// WARNING: If you change this,
// update the shader-side definition too in
// `crates/rash_render/src/shaders/common.wgsl`
/// The graphical state of each sprite. Passed to the shader.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ShaderState {
    pub x: f32,
    pub y: f32,
    pub texture_width: f32,
    pub texture_height: f32,

    pub size: f32,
    pub current_costume: CostumeId,
    pub center_x: f32,
    pub center_y: f32,

    pub shown: i32,
    /// Ignored, just do `[0; _]`
    pub padding: [i32; 7],
}

impl Default for ShaderState {
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
    pub graphics: ShaderState,
}

/// Info of a sprite loaded from disk.
///
/// The difference between this and [`ShaderState`]/[`RunState`]
/// is that some info in that is computed at runtime
/// while this is loaded straight from disk.
#[derive(Debug, Clone, Copy)]
pub struct SpriteLoadData {
    pub x: f64,
    pub y: f64,
    pub size: f64,
    pub costume: CostumeId,
    pub shown: bool,
}
