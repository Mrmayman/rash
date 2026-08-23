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

/// The store for all costumes in the project.
///
/// You can access costumes by name, by index or by [`CostumeId`] here.
#[derive(Default)]
pub struct CostumeStore {
    sprites: Vec<SpriteCostumes>,
    costumes: Vec<RawCostumeData>,

    dedup: HashMap<String, CostumeId>,
}

impl CostumeStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Iterates over all costumes by sequential order of [`CostumeId`].
    pub fn iter_by_id(&self) -> impl Iterator<Item = (CostumeId, &RawCostumeData)> {
        self.costumes
            .iter()
            .enumerate()
            .map(|(i, c)| (CostumeId(i as i32), c))
    }

    /// Frees the CPU-side memory used by costumes.
    ///
    /// Useful for saving resources once you've
    /// uploaded them to the GPU.
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
        sprite.by_name.insert(name, id);
        sprite.by_index.push(id);
    }

    fn get_sprite(&mut self, sprite: SpriteId) -> &mut SpriteCostumes {
        while self.sprites.len() <= sprite.0 as usize {
            self.sprites.push(SpriteCostumes::default());
        }
        self.sprites.get_mut(sprite.0 as usize).unwrap()
    }

    pub fn index_to_id(&self, sprite: SpriteId, index: usize) -> Option<CostumeId> {
        self.sprites
            .get(sprite.0 as usize)
            .and_then(|sprite| sprite.by_index.get(index))
            .copied()
    }

    pub fn name_to_id(&self, sprite: SpriteId, name: &str) -> Option<CostumeId> {
        self.sprites
            .get(sprite.0 as usize)
            .and_then(|sprite| sprite.by_name.get(name))
            .copied()
    }

    pub fn id_to_costume(&self, id: CostumeId) -> Option<&RawCostumeData> {
        self.costumes.get(id.0 as usize)
    }
}

#[derive(Default)]
struct SpriteCostumes {
    by_name: HashMap<String, CostumeId>,
    by_index: Vec<CostumeId>,
}
