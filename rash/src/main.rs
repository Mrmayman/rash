use compiler::MEMORY;
use data_types::ScratchObject;
use input_primitives::STRINGS_TO_DROP;
use rash_render::Renderer;
use sb3::ProjectLoader;
use scheduler::Scheduler;

mod block_print;
mod block_test;
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

    let loader = ProjectLoader::new(&std::path::PathBuf::from(value)).unwrap();
    let scheduler = match loader.build() {
        Ok(n) => n,
        Err(err) => {
            println!("{err}");
            return;
        }
    };

    pollster::block_on(run(scheduler));

    // TODO: Skip screen refresh in some very specific loops.

    drop_strings();
    print_memory();
}

async fn run(scheduler: Scheduler) {
    let renderer = Renderer::new("Rash", Box::new(scheduler)).await.unwrap();
    renderer.run();
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
