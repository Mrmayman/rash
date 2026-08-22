use std::collections::HashMap;

use crate::{CostumeData, CostumeId, SpriteId};

#[derive(Default)]
pub struct Costumes {
    sprites: Vec<SpriteCostumes>,
    pub costumes: Vec<CostumeData>,

    dedup: HashMap<String, CostumeId>,
}

impl Costumes {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_costume(
        &mut self,
        costume: CostumeData,
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
