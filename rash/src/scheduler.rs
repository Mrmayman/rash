use std::{collections::HashMap, fmt::Debug, sync::Arc};

use memmap2::Mmap;
use rash_render::{CostumeId, IntermediateCostume, IntermediateState, Run, RunState, SpriteId};

use crate::{compile_fn::compile, compiler::ScratchBlock, data_types::ScratchObject};

pub type ScratchFunction = unsafe extern "sysv64" fn(
    i64,
    *mut Vec<i64>,
    *const ScratchObject,
    *mut Scheduler,
    *mut RunState,
) -> i64;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct CustomBlockId(pub usize);

pub struct CustomBlock {
    pub thread: ScratchThread,
    pub is_screen_refresh: bool,
}

pub struct Script {
    pub blocks: Vec<ScratchBlock>,
    pub kind: ScriptKind,
}

impl Script {
    pub fn new_green_flag(blocks: Vec<ScratchBlock>) -> Script {
        Self {
            blocks,
            kind: ScriptKind::GreenFlag,
        }
    }

    pub fn new_custom_block(
        blocks: Vec<ScratchBlock>,
        num_args: usize,
        id: CustomBlockId,
        is_screen_refresh: bool,
    ) -> Script {
        Self {
            blocks,
            kind: ScriptKind::CustomBlock {
                id,
                num_args,
                is_screen_refresh,
            },
        }
    }
}

pub enum ScriptKind {
    GreenFlag,
    CustomBlock {
        id: CustomBlockId,
        num_args: usize,
        is_screen_refresh: bool,
    },
}

pub struct SpriteBuilder {
    id: SpriteId,
    scripts: Scripts,
}

impl SpriteBuilder {
    pub fn new(id: SpriteId) -> Self {
        Self {
            id,
            scripts: Scripts::default(),
        }
    }

    pub fn add_script(&mut self, script: &Script) {
        let num_args = match script.kind {
            ScriptKind::GreenFlag => 0,
            ScriptKind::CustomBlock { num_args, .. } => num_args,
        };
        let thread = compile(&script.blocks, self.id, num_args);
        match script.kind {
            ScriptKind::GreenFlag => {
                self.scripts.green_flags.push(thread);
            }
            ScriptKind::CustomBlock {
                id,
                is_screen_refresh,
                ..
            } => {
                self.scripts.custom_blocks.insert(
                    id,
                    CustomBlock {
                        thread,
                        is_screen_refresh,
                    },
                );
            }
        }
    }
}

pub struct ProjectBuilder {
    scheduler: Scheduler,
}

impl ProjectBuilder {
    pub fn new() -> Self {
        Self {
            scheduler: Scheduler {
                sprite_order: Vec::new(),
                threads: Vec::new(),
                scripts: Scripts::default(),
                costume_names: HashMap::new(),
                costume_numbers: HashMap::new(),
                costume_hashes: HashMap::new(),
                costume_intermediate: HashMap::new(),
                temp_init_state: HashMap::new(),
            },
        }
    }

    pub fn finish_sprite(&mut self, sprite: SpriteBuilder) {
        // TODO: Implement proper sprite ordering
        self.scheduler.sprite_order.push(sprite.id);

        self.scheduler.scripts.push(sprite.scripts);
    }

    pub fn set_costume(
        &mut self,
        costume_names: HashMap<(SpriteId, String), String>,
        costume_numbers: HashMap<(SpriteId, usize), String>,
        costume_hashes: HashMap<String, CostumeId>,
        costume_intermediate: HashMap<CostumeId, IntermediateCostume>,
    ) {
        self.scheduler.costume_names = costume_names;
        self.scheduler.costume_numbers = costume_numbers;
        self.scheduler.costume_hashes = costume_hashes;
        self.scheduler.costume_intermediate = costume_intermediate;
    }

    pub fn finish(mut self) -> Scheduler {
        self.scheduler.init();
        self.scheduler
    }

    pub fn set_init_state(&mut self, state_map: HashMap<SpriteId, rash_render::IntermediateState>) {
        self.scheduler.temp_init_state = state_map;
    }
}

pub struct Scheduler {
    pub sprite_order: Vec<SpriteId>,
    pub threads: Vec<ScratchThread>,
    pub scripts: Scripts,
    pub costume_names: HashMap<(SpriteId, String), String>,
    pub costume_numbers: HashMap<(SpriteId, usize), String>,
    pub costume_hashes: HashMap<String, CostumeId>,
    pub costume_intermediate: HashMap<CostumeId, IntermediateCostume>,
    pub temp_init_state: HashMap<SpriteId, IntermediateState>,
}

impl Run for Scheduler {
    fn update(&mut self, state: &mut RunState) -> bool {
        self.sort();
        self.run_threads(state)
    }

    fn get_num_sprites(&self) -> usize {
        self.sprite_order.len()
    }

    fn get_sprite_order(&self) -> Vec<SpriteId> {
        self.sprite_order.clone()
    }

    fn get_costumes(&self) -> HashMap<CostumeId, IntermediateCostume> {
        self.costume_intermediate.clone()
    }

    fn get_state(&self) -> HashMap<SpriteId, rash_render::IntermediateState> {
        self.temp_init_state.clone()
    }
}

impl Scheduler {
    pub fn init(&mut self) {
        let mut green_flags = Vec::new();
        std::mem::swap(&mut self.scripts.green_flags, &mut green_flags);
        self.threads.extend(green_flags);
    }

    fn run_threads(&mut self, graphics: &mut RunState) -> bool {
        // TODO: Potential race condition
        let self_ptr = self as *mut Self;
        let mut ended_threads = Vec::new();

        for (i, thread) in self.threads.iter_mut().enumerate() {
            let has_ended = thread.tick(self_ptr, graphics);
            if has_ended {
                ended_threads.push(i);
            }
        }

        ended_threads.sort_by_key(|&i| std::cmp::Reverse(i));
        for thread in ended_threads {
            self.threads.remove(thread);
        }

        self.threads.is_empty()
    }

    fn sort(&mut self) {
        self.threads.sort_by_key(|thread| {
            self.sprite_order
                .iter()
                .rposition(|&id| id == thread.sprite_id)
        });
    }
}

#[derive(Default)]
pub struct Scripts {
    pub green_flags: Vec<ScratchThread>,
    pub custom_blocks: HashMap<CustomBlockId, CustomBlock>,
}

impl Scripts {
    pub fn push(&mut self, script: Self) {
        self.green_flags.extend(script.green_flags);
        self.custom_blocks.extend(script.custom_blocks);
    }
}

pub struct ScratchThread {
    sprite_id: SpriteId,
    buffer: Arc<Mmap>,
    stack: Vec<i64>,
    arguments: Option<*const ScratchObject>,
    func: ScratchFunction,
    jumped_point: i64,
}

impl Debug for ScratchThread {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ScratchThread {{ {:?}, jumped_point: {} }}",
            self.sprite_id, self.jumped_point
        )
    }
}

impl ScratchThread {
    pub fn spawn(&self, arguments: Option<*const ScratchObject>) -> Self {
        // Non-standard clone behaviour for use
        // when spawning new threads.
        Self {
            buffer: self.buffer.clone(),
            stack: Vec::new(),
            func: self.func,
            jumped_point: 0,
            sprite_id: self.sprite_id,
            arguments,
        }
    }

    pub fn new(buf: &[u8], sprite_id: SpriteId, arguments: Option<*const ScratchObject>) -> Self {
        let mut buffer = memmap2::MmapOptions::new()
            .len(buf.len())
            .map_anon()
            .unwrap();

        buffer.copy_from_slice(buf);
        let buffer = buffer.make_exec().unwrap();

        let func: ScratchFunction = unsafe { std::mem::transmute(buffer.as_ptr()) };

        Self {
            buffer: buffer.into(),
            stack: Vec::new(),
            func,
            jumped_point: 0,
            sprite_id,
            arguments,
        }
    }

    /// Returns true if the thread has finished.
    pub fn tick(&mut self, scheduler_ptr: *mut Scheduler, graphics: *mut RunState) -> bool {
        let result = unsafe {
            (self.func)(
                self.jumped_point,
                &mut self.stack,
                self.arguments.unwrap_or(std::ptr::null()),
                scheduler_ptr,
                graphics,
            )
        };
        self.jumped_point = result;
        result == -1
    }
}
