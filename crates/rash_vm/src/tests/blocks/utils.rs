use std::sync::MutexGuard;

use crate::{
    ProjectBuilder, SpriteBuilder,
    compiler::{MEMORY, ScratchBlock},
    data_types::ScratchObject,
    print_function_addresses, print_memory,
    runtime::Script,
};
use rash_core::{RunState, SpriteId};

fn run(program: Vec<ScratchBlock>, memory: &[ScratchObject]) {
    let mut sprite = SpriteBuilder::new(SpriteId(0));
    sprite.add_script(Script::new_green_flag(program));
    let mut builder = ProjectBuilder::new();
    builder.add_sprite(sprite);
    let mut vm = builder.build(&memory);
    let mut state = RunState::new(); // No graphics operations here

    while !vm.update(&mut state) {}
}

/// Simple headless runner for the JIT, limited in scope.
/// Takes in an array of [`ScratchBlock`] operations, compiles and executes them.
///
/// This is only used for the test-suite.
///
/// Doesn't support:
/// - Screen refresh (pausable functions)
/// - Custom blocks (calling other functions)
/// - Graphical or audio operations
/// - Environment operations (input/sensing)
/// - Timer operations
///
/// # Safety
/// As long as the compiler is functioning correctly,
/// this will be safe, as the machine code under correct
/// circumstances would function correctly.
pub fn run_code<'a>(code: Vec<ScratchBlock>) -> MutexGuard<'a, Box<[ScratchObject]>> {
    let memory = MEMORY.lock().unwrap();
    if std::env::var("RASH_PRINT_FUNCTIONS").is_ok() {
        print_function_addresses();
    }
    run(code, &memory);
    if std::env::var("RASH_PRINT_MEMORY").is_ok() {
        print_memory();
    }
    memory
}
