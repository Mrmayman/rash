use compiler::{print_func_addresses, Compiler, MEMORY};

mod callbacks;
mod compiler;
mod compiler_blocks;
mod data_types;
mod input_primitives;
mod ins_shortcuts;
mod json;
mod test_programs;

fn main() {
    test_case();
    // let arg1 = std::env::args().nth(1).unwrap();
    // println!("opening dir {arg1}");

    print_func_addresses();

    let compiler = Compiler::new();
    compiler.compile();

    // print memory
    for (i, obj) in MEMORY.iter().enumerate().take(10) {
        println!("{}: {:?}", i, obj);
    }
}

fn test_case() {
    let instant = std::time::Instant::now();
    let mut n = 0.0;
    for _ in 0..100000 {
        let s = true as i64 as f64;
        n += s;
        n += s;
    }
    println!("Time taken: {:?}", instant.elapsed());
}
