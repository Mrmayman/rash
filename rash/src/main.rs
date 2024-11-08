use compiler::{print_func_addresses, Compiler, MEMORY};
use input_primitives::STRINGS_TO_DROP;

mod blocks;
mod callbacks;
mod compiler;
mod data_types;
mod input_primitives;
mod ins_shortcuts;
mod json;
mod test_programs;

fn main() {
    // let arg1 = std::env::args().nth(1).unwrap();
    // println!("opening dir {arg1}");

    print_func_addresses();

    let compiler = Compiler::new();
    compiler.compile();

    drop_strings();

    // print memory
    for (i, obj) in MEMORY.iter().enumerate().take(10) {
        println!("{}: {:?}", i, obj);
    }
}

fn drop_strings() {
    let mut strings_buf = STRINGS_TO_DROP.lock().unwrap();
    let mut strings: Vec<[i64; 3]> = Vec::new();
    std::mem::swap(strings_buf.as_mut(), &mut strings);

    for string in strings.into_iter() {
        let _string: String = unsafe { std::mem::transmute(string) };
        println!("Dropping string {_string}");
        // Drop string
    }
}
