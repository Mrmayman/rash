# rash
A Scratch JIT compiler written in rust using [cranelift](https://cranelift.dev/).

# Benchmarks

Pi calculation:

Scratch: 2803 ms
Turbowarp: 30 ms
Rash: 5.6 ms
Rash (without NaN check): 4.2 ms