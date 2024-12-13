use compiler::{ScratchBlock, MEMORY};
use data_types::ScratchObject;
use input_primitives::{Ptr, STRINGS_TO_DROP};
use scheduler::{CustomBlockId, ProjectBuilder, Script, SpriteBuilder, SpriteId};

mod block_test;
mod blocks;
mod callbacks;
mod compile_fn;
mod compiler;
mod constant_set;
mod data_types;
mod input_primitives;
mod ins_shortcuts;
mod json;
mod scheduler;
mod stack_cache;

/// Scratch has a special edge case for math with NaN.
/// Any operation with NaN will be treated as
/// an operation with 0.
///
/// For example, `NaN + 1` will be `0 + 1`.
///
/// This is a special case for Scratch, and is not
/// a standard behavior for most programming languages.
/// Enabling this check adds special behavior for NaN in the
/// compiled code, making it more correct but slower.
///
/// # Performance
/// Pi benchmark:
/// - Without NaN check: `4.6 ms`
/// - With NaN check: `8.8 ms`
const ARITHMETIC_NAN_CHECK: bool = true;

fn main() {
    assert_eq!(std::mem::size_of::<usize>(), 8);
    // Uncomment this to run the GUI.
    pollster::block_on(rash_render::run());
    std::process::exit(0);

    let mut builder = ProjectBuilder::new();
    compiler::print_func_addresses();

    let mut sprite1 = SpriteBuilder::new(SpriteId(0));
    sprite1.add_script(&Script::new_custom_block(
        vec![ScratchBlock::VarChange(Ptr(3), 2.0.into())],
        0,
        CustomBlockId(1),
        false,
    ));
    sprite1.add_script(&Script::new_custom_block(
        vec![
            ScratchBlock::VarSet(Ptr(3), 1.5.into()),
            ScratchBlock::FunctionCallNoScreenRefresh(CustomBlockId(1), Vec::new()),
        ],
        0,
        CustomBlockId(0),
        false,
    ));
    sprite1.add_script(&Script::new_green_flag(vec![
        ScratchBlock::FunctionCallNoScreenRefresh(CustomBlockId(0), Vec::new()),
    ]));

    let mut sprite2 = SpriteBuilder::new(SpriteId(1));
    sprite2.add_script(&Script::new_green_flag(vec![ScratchBlock::VarSet(
        Ptr(3),
        1.0.into(),
    )]));
    // sprite1.add_script(Script::new_green_flag(block_test::repeat_until()));
    // TODO: Skip screen refresh in some very specific loops.
    // sprite1.add_script(Script::new_green_flag(
    //     block_test::screen_refresh_nested_repeat(),
    // ));
    builder.finish_sprite(sprite2);
    builder.finish_sprite(sprite1);

    let mut scheduler = builder.finish();

    let mut num_ticks = 1;
    while !scheduler.tick() {
        num_ticks += 1;
        // std::thread::sleep(std::time::Duration::from_millis(1000 / 30))
    }
    println!("Ticks: {num_ticks}");

    drop_strings();
    print_memory();
}

fn print_memory() {
    let lock = MEMORY.lock().unwrap();

    // Only print the changed values that aren't zero.
    let print_until_idx = lock
        .iter()
        .enumerate()
        .rev()
        .find(|(_, n)| !matches!(**n, ScratchObject::Number(0.0)))
        .map(|(i, _)| i);
    if let Some(print_until_idx) = print_until_idx {
        for (i, obj) in lock.iter().enumerate().take(print_until_idx + 1) {
            println!("{i}: {obj:?}");
        }
    }
    println!("...: {:?}", ScratchObject::Number(0.0));
}

fn drop_strings() {
    let mut strings_buf = STRINGS_TO_DROP.lock().unwrap();
    let mut strings: Vec<[i64; 3]> = Vec::new();
    std::mem::swap(strings_buf.as_mut(), &mut strings);

    for string in strings {
        let _string: String = unsafe { std::mem::transmute(string) };
        // println!("Dropping string {_string}");
        // Drop string
    }
}
