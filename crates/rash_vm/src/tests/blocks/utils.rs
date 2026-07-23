use std::{collections::HashMap, sync::MutexGuard};

use crate::{
    ProjectBuilder, RunState, SpriteBuilder, SpriteData,
    compiler::{MEMORY, ScratchBlock},
    data_types::ScratchObject,
    graphics::SpriteId,
    print_function_addresses, print_memory,
    runtime::Script,
};

fn run(program: Vec<ScratchBlock>, memory: &[ScratchObject]) {
    let mut sprite = SpriteBuilder::new(SpriteId(0));
    sprite.add_script(Script::new_green_flag(program), &memory);
    let mut builder = ProjectBuilder::new();
    builder.add_sprite(sprite);
    let mut vm = builder.build(&memory);
    let mut state = RunState {
        // We won't do any graphics operations here
        sprites: HashMap::from([(SpriteId(0), SpriteData::default())]),
    };

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
#[allow(unused)]
pub fn run_code<'a>(code: Vec<ScratchBlock>) -> MutexGuard<'a, Box<[ScratchObject]>> {
    let mut memory = MEMORY.lock().unwrap();
    if std::env::var("RASH_PRINT_FUNCTIONS").is_ok() {
        print_function_addresses();
    }
    run(code, &memory);
    if std::env::var("RASH_PRINT_MEMORY").is_ok() {
        print_memory();
    }
    memory
}
