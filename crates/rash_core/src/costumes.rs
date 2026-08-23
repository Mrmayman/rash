use std::collections::HashMap;

use crate::{CostumeId, SpriteId};

/// The raw costume information as read from the project file.
#[derive(Clone)]
pub struct RawCostumeData {
    pub bytes: Vec<u8>,
    pub name: String,
    pub hash: String,
    pub rotation_center_x: f64,
    pub rotation_center_y: f64,
    pub is_svg: bool,
}

#[derive(Default)]
pub struct CostumeStore {
    sprites: Vec<SpriteCostumes>,
    pub costumes: Vec<RawCostumeData>,

    dedup: HashMap<String, CostumeId>,
}

impl CostumeStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn free_memory(&mut self) {
        for c in &mut self.costumes {
            c.bytes.clear();
        }
    }

    pub fn add_costume(
        &mut self,
        costume: RawCostumeData,
        name: String,
        hash: String,
        sprite: SpriteId,
    ) {
        let id = if let Some(id) = self.dedup.get(&hash) {
            *id
        } else {
            let id = CostumeId(self.costumes.len() as i32);
            self.dedup.insert(hash, id);
            self.costumes.push(costume);
            id
        };

        let sprite = self.get_sprite(sprite);
        sprite.names.insert(name, id);
        sprite.numbers.push(id);
    }

    fn get_sprite(&mut self, sprite: SpriteId) -> &mut SpriteCostumes {
        while self.sprites.len() <= sprite.0 as usize {
            self.sprites.push(SpriteCostumes::default());
        }
        self.sprites.get_mut(sprite.0 as usize).unwrap()
    }

    pub fn get_by_number(&self, sprite: SpriteId, number: usize) -> Option<CostumeId> {
        self.sprites
            .get(sprite.0 as usize)
            .and_then(|sprite| sprite.numbers.get(number))
            .copied()
    }
}

#[derive(Default)]
struct SpriteCostumes {
    names: HashMap<String, CostumeId>,
    numbers: Vec<CostumeId>,
}
