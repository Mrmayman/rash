use compiler::{ScratchBlock, MEMORY};
use data_types::ScratchObject;
use input_primitives::{Ptr, STRINGS_TO_DROP};
use rash_render::{Renderer, Run, RunState, SpriteId};
use sb3::ProjectLoader;
use scheduler::{CustomBlockId, ProjectBuilder, Scheduler, Script, SpriteBuilder};

mod block_print;
mod blocks;
mod callbacks;
mod compile_fn;
mod compiler;
mod constant_set;
mod data_types;
mod error;
mod input_primitives;
mod ins_shortcuts;
mod sb3;
mod scheduler;
mod stack_cache;
mod tests;

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
/// - With NaN check: `6.5 ms`
const ARITHMETIC_NAN_CHECK: bool = true;

fn main() {
    assert_eq!(std::mem::size_of::<usize>(), 8);

    let Some(value) = std::env::args().nth(1) else {
        eprintln!("Usage: rash /path/to/project.sb3");
        return;
    };

    if value == "demo" {
        run_demo();
        drop_strings();
        print_memory();
        std::process::exit(0);
    }

    let loader = match ProjectLoader::new(&std::path::PathBuf::from(value)) {
        Ok(n) => n,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    };
    let scheduler = match loader.build() {
        Ok(n) => n,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    };

    // return;

    pollster::block_on(run(scheduler));

    // TODO: Skip screen refresh in some very specific loops.

    drop_strings();
    print_memory();
}

fn run_demo() {
    let mut builder = ProjectBuilder::new();
    compiler::print_func_addresses();

    let memory = MEMORY.lock().unwrap();

    let mut sprite1 = SpriteBuilder::new(SpriteId(0));
    sprite1.add_script(
        &Script::new_custom_block(
            vec![ScratchBlock::ControlRepeat(
                3.0.into(),
                vec![ScratchBlock::VarChange(
                    Ptr(3),
                    ScratchBlock::FunctionGetArg(0).into(),
                )],
            )],
            1,
            CustomBlockId(1),
            true,
        ),
        &memory,
    );
    sprite1.add_script(
        &Script::new_custom_block(
            vec![
                ScratchBlock::VarSet(Ptr(3), 0.5.into()),
                ScratchBlock::ControlRepeat(
                    5.0.into(),
                    vec![ScratchBlock::FunctionCallScreenRefresh(
                        CustomBlockId(1),
                        vec![1.0.into()],
                    )],
                ),
            ],
            0,
            CustomBlockId(0),
            true,
        ),
        &memory,
    );
    sprite1.add_script(
        &Script::new_green_flag(vec![
            ScratchBlock::VarSet(Ptr(3), 0.5.into()),
            // ScratchBlock::FunctionCallScreenRefresh(CustomBlockId(1), vec![1.0.into()]),
            ScratchBlock::FunctionCallScreenRefresh(CustomBlockId(0), Vec::new()),
        ]),
        &memory,
    );
    // sprite1.add_script(Script::new_green_flag(block_test::repeat_until()));
    // TODO: Skip screen refresh in some very specific loops.
    // sprite1.add_script(Script::new_green_flag(
    //     block_test::screen_refresh_nested_repeat(),
    // ));
    builder.finish_sprite(sprite1);

    let mut scheduler = builder.finish();

    let mut num_ticks = 1;
    let mut graphics = RunState::default();
    while !scheduler.update(&mut graphics) {
        num_ticks += 1;
        // std::thread::sleep(std::time::Duration::from_millis(1000 / 30))
    }
    println!("Ticks: {num_ticks}");
}

async fn run(scheduler: Scheduler) {
    let renderer = Renderer::new("Rash", Box::new(scheduler)).await.unwrap();
    renderer.run();
}

fn print_memory() {
    let lock = MEMORY.lock().unwrap();

    println!("MEMORY: {:X}", lock.as_ptr() as usize);

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
