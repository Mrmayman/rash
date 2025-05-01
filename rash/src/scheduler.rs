use std::{collections::HashMap, fmt::Debug, sync::Arc};

use memmap2::Mmap;
use rash_render::{CostumeId, IntermediateCostume, IntermediateState, Run, RunState, SpriteId};

use crate::{compile_fn::compile, compiler::ScratchBlock, data_types::ScratchObject};

/// The signature of the JIT-compiled function that
/// runs Scratch code.
///
/// # Arguments
/// ## `i64`: Jump ID
/// Some scratch functions can pause.
/// In that case, it returns a number representing
/// where it paused. You pass that number to it again
/// to continue the function from that point.
///
/// Default: `0`
///
/// ## `*mut Vec<i64>`: The loop stack
/// This represents what loop we're inside and how
/// much progress out of how much is done inside that loop.
/// Usually it's empty but if the function pauses inside
/// a loop, it will have contents.
///
/// Can be `null` if the function doesn't pause.
///
/// For example:
///
/// ```no_run
/// # fn repeat(_: usize) {}
/// repeat(10) {
///     repeat(15) {
///         // stack =
///         // 3, 10 <- (done (3 for example), out of)
///         // 5, 15
///     }
///     // stack =
///     // 3, 10 <- (done (3 for example), out of)
///
///     // We finished the inner `repeat 15` loop,
///     // so now there's just 2 entries (1 loop)
///     // left
/// }
/// ```
///
/// ## `*const ScratchObject`
/// The arguments to the function if it is a custom block.
///
/// Can be `null` if there aren't any arguments
/// (don't worry, the function will figure out for itself).
///
/// This is a pointer to the first item (easier to handle in machine code),
/// the function will automatically offset and call the next items as necessary
///
/// ## `*const Scripts`: Ready-made Scripts for running
/// A handle to the [`Scripts`]. This is used for "spawning"
/// Custom Blocks to be called, ie. getting a handle to another JIT
/// function to be called from a JIT function.
///
/// Can be `null` if you aren't calling other Custom Blocks (functions)
/// from your Scratch code.
///
/// ## `*mut RunState`: Running (and graphics) state.
/// Primarily used for managing the graphical state of the project.
/// Sprite positioning, scaling, colors, costumes and so on.
///
/// Can be `null` if you aren't doing any graphics operations.
///
/// ## `i64`: Is Screen Refresh (Pausable)
/// Yes (recommended default) if `1`, No (faster) if `0`.
/// This is `i64` instead of `bool` for easy ABI handling in machine code.
///
/// If it is screen refresh, the function **is capable**
/// of pausing and resuming from within like a coroutine.
///
/// More explanation above in the comment for first argument.
///
/// ## `*mut Option<ScratchThread>`: The "Child" function to be called
///
/// Must point to a valid `None` if not used. **Cannot be null!**.
///
/// Let's say we have a function `a` that calls function `b`.
///
/// ```no_run
/// # fn repeat(_: usize) {}
/// fn a() {
///     b()
/// }
/// fn b() {
///     repeat(5) {
///         // do something
///         yield; // screen refresh
///     }
/// }
/// ```
///
/// If function `b` pauses (the screen refresh or `yield`), while
/// called by function `a`, then function `a` stores the `b`
/// [`ScratchThread`] inside this `Option` (which is `None` by default),
/// then function `a` also pauses.
///
/// Later when the scheduler continues `a`, it will see the child thread `b`
/// and resume from there.
pub type ScratchFunction = unsafe extern "C" fn(
    i64, // Jump ID
    *mut Vec<i64>,
    *const ScratchObject,
    *const Scripts,
    *mut RunState,
    i64, // Is screen refresh
    *mut Option<ScratchThread>,
) -> i64;

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

    pub fn add_script(&mut self, script: &Script) {
        let num_args = match script.kind {
            ScriptKind::GreenFlag => 0,
            ScriptKind::CustomBlock { num_args, .. } => num_args,
        };
        let thread = compile(
            &script.blocks,
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
        let mut ended_threads = Vec::new();

        for (i, thread) in self.threads.iter_mut().enumerate() {
            let has_ended = unsafe { thread.tick(&self.scripts, graphics) };
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

    stack_repeat: Vec<i64>,
    jumped_point: i64,
    child_thread: Box<Option<ScratchThread>>,

    buffer: Arc<Mmap>,
    func: ScratchFunction,
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
    pub fn spawn(&self, is_screen_refresh: bool, arguments: Vec<ScratchObject>) -> Self {
        // Non-standard clone behaviour for use
        // when spawning new threads.
        Self {
            buffer: self.buffer.clone(),
            stack_repeat: Vec::new(),
            func: self.func,
            jumped_point: 0,
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
        let func: ScratchFunction = unsafe { std::mem::transmute(buffer.as_ptr()) };

        Self {
            buffer: buffer.into(),
            stack_repeat: Vec::new(),
            func,
            jumped_point: 0,
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
    /// - `graphics` points to a valid, global instance
    ///   of [`RunState`]
    /// - There hopefully aren't any compiler bugs
    ///   creating broken code
    pub unsafe fn tick(&mut self, scripts: &Scripts, graphics: &mut RunState) -> bool {
        const DONE: i64 = -1;

        // If the parent (current) thread paused while
        // running a child thread which also paused,
        // then tick the child thread instead until it ends,
        if let Some(thread) = &mut *self.child_thread {
            let child_ended = thread.tick(scripts, graphics);
            if child_ended {
                *self.child_thread = None;
            }
            return child_ended;
        }

        let result = unsafe {
            (self.func)(
                self.jumped_point,
                &mut self.stack_repeat,
                if self.arguments.is_empty() {
                    std::ptr::null()
                } else {
                    self.arguments.as_ptr()
                },
                scripts,
                graphics,
                self.is_screen_refresh as i64,
                &mut *self.child_thread,
            )
        };
        self.jumped_point = result;
        result == DONE
    }
}
