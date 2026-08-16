use std::{collections::HashMap, fmt::Debug, sync::Arc};

use cranelift::codegen::CompiledCode;
use memmap2::Mmap;
use smol_str::SmolStr;

use crate::{
    compile_fn::{compile, prepare_buffer},
    compiler::{FuncMap, ScratchBlock},
    data_types::ScratchObject,
    graphics::{CostumeData, CostumeHash, CostumeId, RunState, SpriteId, SpriteLoadData},
};

#[doc = include_str!("../../../docs/JIT_SIGNATURE.md")]
type JitFunction = unsafe extern "C" fn(
    JumpId,
    *mut Vec<i64>, // Stack repeat
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

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct CustomBlockId(pub usize);

pub enum CustomBlockFunc {
    ToCompile(Script),
    Compiled(ScratchThread),
}

pub struct CustomBlock {
    pub script: CustomBlockFunc,
    pub sprite_id: SpriteId,
    pub is_screen_refresh: bool,
    pub num_args: usize,
}

pub struct Script {
    pub blocks: Vec<ScratchBlock>,
    pub kind: ScriptKind,
}

impl Script {
    #[must_use]
    pub fn new_green_flag(blocks: Vec<ScratchBlock>) -> Script {
        Self {
            blocks,
            kind: ScriptKind::GreenFlag,
        }
    }

    #[must_use]
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

    fn insert_refreshes(&mut self) {
        fn check_block(b: &mut ScratchBlock) {
            match b {
                ScratchBlock::ControlRepeat(_, blocks)
                | ScratchBlock::ControlRepeatUntil(_, blocks)
                | ScratchBlock::ControlForever(blocks) => {
                    let mut trigger = false;
                    for block in blocks.iter_mut() {
                        if block.could_trigger_refresh() {
                            trigger = true;
                        }
                        check_block(block);
                    }
                    if trigger && !matches!(blocks.last(), Some(ScratchBlock::ScreenRefresh)) {
                        blocks.push(ScratchBlock::ScreenRefresh);
                    }
                }

                ScratchBlock::ControlIf(_, b_then) => {
                    for block in b_then {
                        check_block(block);
                    }
                }
                ScratchBlock::ControlIfElse(_, b_then, b_else) => {
                    for block in b_then {
                        check_block(block);
                    }
                    for block in b_else {
                        check_block(block);
                    }
                }

                ScratchBlock::VarSet(_, _)
                | ScratchBlock::VarChange(_, _)
                | ScratchBlock::VarRead(_)
                | ScratchBlock::OpAdd(_, _)
                | ScratchBlock::OpSub(_, _)
                | ScratchBlock::OpMul(_, _)
                | ScratchBlock::OpDiv(_, _)
                | ScratchBlock::OpRound(_)
                | ScratchBlock::OpStrJoin(_, _)
                | ScratchBlock::OpMod(_, _)
                | ScratchBlock::OpStrLen(_)
                | ScratchBlock::OpBAnd(_, _)
                | ScratchBlock::OpBNot(_)
                | ScratchBlock::OpBOr(_, _)
                | ScratchBlock::OpMFloor(_)
                | ScratchBlock::OpMAbs(_)
                | ScratchBlock::OpMSqrt(_)
                | ScratchBlock::OpMSin(_)
                | ScratchBlock::OpMCos(_)
                | ScratchBlock::OpMTan(_)
                | ScratchBlock::OpCmp(_, _, _)
                | ScratchBlock::OpRandom(_, _)
                | ScratchBlock::OpStrLetterOf(_, _)
                | ScratchBlock::OpStrContains(_, _)
                | ScratchBlock::ControlStopThisScript
                | ScratchBlock::FunctionCallNoScreenRefresh(_, _)
                | ScratchBlock::FunctionCallScreenRefresh(_, _)
                | ScratchBlock::FunctionGetArg(_)
                | ScratchBlock::ScreenRefresh
                | ScratchBlock::MotionGoToXY(_, _)
                | ScratchBlock::MotionChangeX(_)
                | ScratchBlock::MotionChangeY(_)
                | ScratchBlock::MotionSetX(_)
                | ScratchBlock::MotionSetY(_)
                | ScratchBlock::MotionGetX
                | ScratchBlock::MotionGetY
                | ScratchBlock::LooksShown(_)
                | ScratchBlock::SensingDaysSince2000
                | ScratchBlock::Log(_) => {}
            }
        }

        if !self.kind.is_screen_refresh() {
            return;
        }

        for block in &mut self.blocks {
            check_block(block);
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
    #[must_use]
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
    static_strings: Vec<SmolStr>,
}

impl SpriteBuilder {
    #[must_use]
    pub fn new(id: SpriteId) -> Self {
        Self {
            id,
            scripts: Scripts::default(),
            static_strings: Vec::new(),
        }
    }

    pub fn add_script(&mut self, mut script: Script, memory: &[ScratchObject]) {
        script.insert_refreshes();

        let num_args = match script.kind {
            ScriptKind::GreenFlag => 0,
            ScriptKind::CustomBlock { num_args, .. } => num_args,
        };
        match script.kind {
            ScriptKind::GreenFlag => {
                // This is where all your magic happens :D
                let (thread, static_strings) = compile(
                    &script.blocks,
                    memory,
                    self.id,
                    num_args,
                    script.kind.is_screen_refresh(),
                );

                self.static_strings.extend(static_strings);
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
                        // Delaying this till later for intelligent analysis
                        script: CustomBlockFunc::ToCompile(script),
                        is_screen_refresh,
                        num_args,
                        sprite_id: self.id,
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
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_sprite(&mut self, sprite: SpriteBuilder) {
        // TODO: Implement proper sprite ordering
        self.runtime.sprite_order.push(sprite.id);
        self.runtime.static_strings.extend(sprite.static_strings);
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

    #[must_use]
    pub fn build(mut self, memory: &[ScratchObject]) -> Runtime {
        for script in self.runtime.scripts.custom_blocks.values_mut() {
            let CustomBlockFunc::ToCompile(scr) = &script.script else {
                continue;
            };

            // Compiling custom blocks at last moment
            // so that we get the most data for analysis
            let (thread, static_strings) = compile(
                &scr.blocks,
                memory,
                script.sprite_id,
                script.num_args,
                scr.kind.is_screen_refresh(),
            );

            script.script = CustomBlockFunc::Compiled(thread);
            self.runtime.static_strings.extend(static_strings);
        }

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
    static_strings: Vec<SmolStr>,
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

    stack_repeat: Vec<i64>,
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
    #[must_use]
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

    #[must_use]
    pub fn new(
        code: &CompiledCode,
        func_map: &FuncMap,
        sprite_id: SpriteId,
        is_screen_refresh: bool,
    ) -> Self {
        let buf = code.code_buffer();

        let mut buffer = memmap2::MmapOptions::new()
            .len(buf.len())
            .map_anon()
            .unwrap();

        buffer.copy_from_slice(buf);

        prepare_buffer(code, &buffer, func_map);

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
                &raw mut self.stack_repeat,
                self.arguments.as_ptr(),
                scripts,
                state,
                self.is_screen_refresh,
                &raw mut *self.child_thread,
            )
        };
        self.jumped_point = result;

        result.is_done()
    }
}
