mod callbacks;
mod compiler;
mod data_types;
mod input_primitives;
mod ins_shortcuts;
mod json;

fn main() {
    test_case();
    compiler::c_main();
}

fn test_case() {
    let instant = std::time::Instant::now();
    let mut n = 0.0;
    for _ in 0..100000 {
        let s = true as i64 as f64;
        n += s;
    }
    println!("Time taken: {:?}", instant.elapsed());
}
