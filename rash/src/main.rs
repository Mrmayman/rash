use compiler::{compile, print_func_addresses, MEMORY};
use input_primitives::STRINGS_TO_DROP;

mod block_test;
mod blocks;
mod callbacks;
mod compiler;
mod constant_set;
mod data_types;
mod input_primitives;
mod ins_shortcuts;
mod json;
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
    // let arg1 = std::env::args().nth(1).unwrap();
    // println!("opening dir {arg1}");

    assert_eq!(std::mem::size_of::<usize>(), 8);

    print_func_addresses();

    // let compiler = Compiler::new();
    compile();

    drop_strings();

    // print memory
    for (i, obj) in MEMORY.lock().unwrap().iter().enumerate().take(10) {
        println!("{i}: {obj:?}");
    }
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
