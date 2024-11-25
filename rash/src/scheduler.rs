use std::{fmt::Debug, sync::Arc};

use memmap2::Mmap;

use crate::{compile_fn::compile, compiler::ScratchBlock};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SpriteId(pub usize);

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

    pub fn add_script(&mut self, script: Vec<ScratchBlock>) {
        let thread = compile(&script, self.id);
        self.scripts.green_flags.push(thread);
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
                thread_groups: Vec::new(),
                scripts: Scripts::default(),
            },
        }
    }

    pub fn finish_sprite(&mut self, sprite: SpriteBuilder) {
        // TODO: Implement proper sprite ordering
        self.scheduler.sprite_order.push(sprite.id);

        self.scheduler.scripts.push(sprite.scripts);
    }

    pub fn finish(mut self) -> Scheduler {
        self.scheduler.init();
        self.scheduler
    }
}

pub struct Scheduler {
    pub sprite_order: Vec<SpriteId>,
    pub thread_groups: Vec<Vec<ScratchThread>>,
    pub scripts: Scripts,
}

impl Scheduler {
    pub fn init(&mut self) {
        let mut green_flags = Vec::new();
        std::mem::swap(&mut self.scripts.green_flags, &mut green_flags);
        self.thread_groups.push(green_flags);
    }

    pub fn tick(&mut self) -> bool {
        self.sort();
        self.run_threads()
    }

    fn run_threads(&mut self) -> bool {
        let mut ended_groups = Vec::new();
        for (i, group) in self.thread_groups.iter_mut().enumerate() {
            let mut ended_threads = Vec::new();

            for (i, thread) in group.iter_mut().enumerate() {
                let has_ended = thread.tick();
                if has_ended {
                    ended_threads.push(i);
                }
            }

            ended_threads.sort_by_key(|&i| std::cmp::Reverse(i));

            for thread in ended_threads {
                group.remove(thread);
            }

            if group.is_empty() {
                ended_groups.push(i);
            }
        }

        ended_groups.sort_by_key(|&i| std::cmp::Reverse(i));

        for group in ended_groups {
            self.thread_groups.remove(group);
        }
        self.thread_groups.is_empty()
    }

    fn sort(&mut self) {
        for group in &mut self.thread_groups {
            group.sort_by_key(|thread| {
                self.sprite_order
                    .iter()
                    .rposition(|&id| id == thread.sprite_id)
            });
        }
    }
}

#[derive(Default)]
pub struct Scripts {
    pub green_flags: Vec<ScratchThread>,
}

impl Scripts {
    pub fn push(&mut self, script: Self) {
        self.green_flags.extend(script.green_flags);
    }
}

pub struct ScratchThread {
    sprite_id: SpriteId,
    _buffer: Arc<Mmap>,
    stack: Vec<i64>,
    func: unsafe extern "sysv64" fn(i64, *mut Vec<i64>) -> i64,
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

impl Clone for ScratchThread {
    fn clone(&self) -> Self {
        // Non-standard clone behaviour for use
        // when spawning new threads.
        Self {
            _buffer: self._buffer.clone(),
            stack: Vec::new(),
            func: self.func,
            jumped_point: 0,
            sprite_id: self.sprite_id,
        }
    }
}

impl ScratchThread {
    pub fn new(buf: &[u8], sprite_id: SpriteId) -> Self {
        let mut buffer = memmap2::MmapOptions::new()
            .len(buf.len())
            .map_anon()
            .unwrap();

        buffer.copy_from_slice(buf);
        let buffer = buffer.make_exec().unwrap();

        let func: unsafe extern "sysv64" fn(i64, *mut Vec<i64>) -> i64 =
            unsafe { std::mem::transmute(buffer.as_ptr()) };

        Self {
            _buffer: buffer.into(),
            stack: Vec::new(),
            func,
            jumped_point: 0,
            sprite_id,
        }
    }

    /// Returns true if the thread has finished.
    pub fn tick(&mut self) -> bool {
        let result = unsafe { (self.func)(self.jumped_point, &mut self.stack) };
        self.jumped_point = result;
        result == -1
    }
}
