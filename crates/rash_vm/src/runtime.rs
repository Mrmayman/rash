use std::{collections::HashMap, fmt::Debug, sync::Arc};

use memmap2::Mmap;

use crate::{
    compile_fn::compile,
    compiler::ScratchBlock,
    data_types::ScratchObject,
    graphics::{CostumeData, CostumeHash, CostumeId, RunState, SpriteId, SpriteLoadData},
    input_primitives::STRINGS_TO_DROP,
};

#[doc = include_str!("../../../docs/JIT_SIGNATURE.md")]
type JitFunction = unsafe extern "C" fn(
    JumpId,
    *mut Vec<LoopFrame>,
    *const ScratchObject,
    *const Scripts,
    *mut RunState,
    bool, // Is screen refresh
    *mut Option<ScratchThread>,
) -> JumpId;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Default)]
struct JumpId(i64);

impl JumpId {
    const DONE: Self = Self(-1);

    fn is_done(self) -> bool {
        self == Self::DONE
    }
}

impl Debug for JumpId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_done() {
            write!(f, "JumpId(finished)")
        } else {
            f.debug_tuple("JumpId").field(&self.0).finish()
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct LoopFrame {
    pub done: i64,
    pub out_of: i64,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct CustomBlockId(pub usize);

pub struct CustomBlock {
    pub thread: ScratchThread,
    pub is_screen_refresh: bool,
    pub num_args: usize,
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

impl ScriptKind {
    pub fn is_screen_refresh(&self) -> bool {
        match self {
            ScriptKind::GreenFlag => true,
            ScriptKind::CustomBlock {
                is_screen_refresh, ..
            } => *is_screen_refresh,
        }
    }
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

    pub fn add_script(&mut self, script: &Script, memory: &[ScratchObject]) {
        let num_args = match script.kind {
            ScriptKind::GreenFlag => 0,
            ScriptKind::CustomBlock { num_args, .. } => num_args,
        };
        let thread = compile(
            &script.blocks,
            memory,
            self.id,
            num_args,
            script.kind.is_screen_refresh(),
        );
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
                        num_args,
                    },
                );
            }
        }
    }
}

#[derive(Default)]
pub struct ProjectBuilder {
    runtime: Runtime,
}

impl ProjectBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_sprite(&mut self, sprite: SpriteBuilder) {
        // TODO: Implement proper sprite ordering
        self.runtime.sprite_order.push(sprite.id);

        self.runtime.scripts.push(sprite.scripts);
    }

    pub fn set_costume(
        &mut self,
        costume_names: HashMap<(SpriteId, String), CostumeHash>,
        costume_numbers: HashMap<(SpriteId, usize), CostumeHash>,
        costume_hashes: HashMap<CostumeHash, CostumeId>,
        costume_intermediate: HashMap<CostumeId, CostumeData>,
    ) {
        self.runtime.costume_names = costume_names;
        self.runtime.costume_numbers = costume_numbers;
        self.runtime.costume_hashes = costume_hashes;
        self.runtime.costume_data = costume_intermediate;
    }

    pub fn build(mut self) -> Runtime {
        self.runtime.init();
        self.runtime
    }

    pub fn set_init_state(&mut self, state_map: HashMap<SpriteId, SpriteLoadData>) {
        self.runtime.sprite_load_info = state_map;
    }
}

#[derive(Default)]
pub struct Runtime {
    pub sprite_order: Vec<SpriteId>,
    threads: Vec<ScratchThread>,
    scripts: Scripts,

    costume_names: HashMap<(SpriteId, String), CostumeHash>,
    costume_numbers: HashMap<(SpriteId, usize), CostumeHash>,
    costume_hashes: HashMap<CostumeHash, CostumeId>,
    pub costume_data: HashMap<CostumeId, CostumeData>,

    pub sprite_load_info: HashMap<SpriteId, SpriteLoadData>,
}

impl Runtime {
    pub fn init(&mut self) {
        // We currently only support 64-bit platforms
        // (may change in the future if we add a Bytecode VM)
        assert_eq!(std::mem::size_of::<usize>(), 8);

        let mut green_flags = Vec::new();
        std::mem::swap(&mut self.scripts.green_flags, &mut green_flags);
        self.threads.extend(green_flags);
    }

    pub fn update(&mut self, state: &mut RunState) -> bool {
        self.sort();

        let mut ended_threads = Vec::new();

        for (i, thread) in self.threads.iter_mut().enumerate() {
            // Safety: Many invariants are checked by the runtime
            let has_ended = unsafe { thread.tick(&self.scripts, state) };
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
    is_screen_refresh: bool,
    arguments: Vec<ScratchObject>,

    stack_repeat: Vec<LoopFrame>,
    jumped_point: JumpId,
    child_thread: Box<Option<ScratchThread>>,

    buffer: Arc<Mmap>,
    func: JitFunction,
}

impl Debug for ScratchThread {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScratchThread")
            .field("sprite_id", &self.sprite_id)
            .field("is_screen_refresh", &self.is_screen_refresh)
            .field("arguments", &self.arguments)
            .field("stack_repeat", &self.stack_repeat)
            .field("jumped_point", &self.jumped_point)
            .field("child_thread", &self.child_thread)
            .finish_non_exhaustive()
    }
}

impl ScratchThread {
    pub fn spawn(&self, is_screen_refresh: bool, arguments: Vec<ScratchObject>) -> Self {
        // Non-standard clone behaviour for use
        // when spawning new threads.
        Self {
            buffer: self.buffer.clone(),
            stack_repeat: Vec::new(),
            func: self.func,
            jumped_point: JumpId::default(),
            sprite_id: self.sprite_id,
            is_screen_refresh,
            child_thread: Box::new(None),
            arguments,
        }
    }

    pub fn new(buf: &[u8], sprite_id: SpriteId, is_screen_refresh: bool) -> Self {
        let mut buffer = memmap2::MmapOptions::new()
            .len(buf.len())
            .map_anon()
            .unwrap();

        buffer.copy_from_slice(buf);
        let buffer = buffer.make_exec().unwrap();

        // Safety:
        // If the cranelift compiler is working properly (I hope!)
        // then this should be safe, as it is a valid function.
        let func: JitFunction = unsafe { std::mem::transmute(buffer.as_ptr()) };

        Self {
            buffer: buffer.into(),
            stack_repeat: Vec::new(),
            func,
            jumped_point: JumpId::default(),
            sprite_id,
            is_screen_refresh,
            child_thread: Box::new(None),
            arguments: Vec::new(),
        }
    }

    /// Returns true if the thread has finished.
    ///
    /// # Safety
    /// This is highly unsafe because you're running
    /// arbitrary machine code made by a kinda buggy compiler.
    ///
    /// This should be safe **as long as**:
    /// - Number of custom-block arguments (when compiling)
    ///   matches number of arguments when running (failing to do so
    ///   may result in panics or undefined behaviour).
    /// - There hopefully aren't any compiler bugs
    ///   creating broken code
    pub unsafe fn tick(&mut self, scripts: &Scripts, state: &mut RunState) -> bool {
        if self.jumped_point.is_done() {
            return true;
        }

        // If the parent (current) thread paused while
        // running a child thread which also paused,
        // then tick the child thread instead until it ends,
        if let Some(thread) = &mut *self.child_thread {
            let child_ended = unsafe { thread.tick(scripts, state) };
            if child_ended {
                *self.child_thread = None;
            } else {
                return false;
            }
        }

        let result = unsafe {
            (self.func)(
                self.jumped_point,
                &mut self.stack_repeat,
                self.arguments.as_ptr(),
                scripts,
                state,
                self.is_screen_refresh,
                &mut *self.child_thread,
            )
        };
        self.jumped_point = result;

        result.is_done()
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        let mut strings_buf = STRINGS_TO_DROP.lock().unwrap();
        let s: &mut Vec<[i64; 3]> = strings_buf.as_mut();
        let strings = std::mem::take(s);

        for string in strings {
            let _string: String = unsafe { std::mem::transmute(string) };
            // println!("Dropping string {_string}");
            // Drop string
        }
    }
}
