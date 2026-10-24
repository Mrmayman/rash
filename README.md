# rash
A Scratch JIT compiler written in rust using [cranelift](https://cranelift.dev/).

> **Warning:** This is highly experimental, incomplete software.
> If you want an existing production-ready solution, see [scratchcpp](https://github.com/scratchcpp/scratchcpp-player/)

# What is this?

Scratch is a visual programming language that people push far beyond simple games,
building projects like 3D Minecraft clones, path tracers, and neural networks.

Rash aims to make these projects run faster by compiling Scratch code to
native machine code using a JIT compiler, powered by Rust and [Cranelift](https://cranelift.dev/).

# Benchmarks

Pi calculation (10,000,000 iterations)

**System**: Ryzen 7 7840HS, 24 GB RAM, Fedora GNOME

| **Platform** | **Time** |
| --- | --- |
| Scratch | \~6770 ms (6.8s) |
| [ScratchCPP](https://github.com/scratchcpp/scratchcpp-player/) | \~126 ms |
| Turbowarp (Warp timer on) | \~53 ms |
| Turbowarp | \~37 ms |
| Rash | \~26.3 ms |
| Rash (with lossy flags) | \~24.7 ms |

# What's the progress?

You can see the progress in implementing blocks in this [Google Docs spreadsheet](https://docs.google.com/spreadsheets/d/1jYi5lsAyq6XeJPCKCpk4UkqF1YWVPX9C4d7-eTbXw9U/edit?usp=sharing)

# Running

- Install the Rust language if you haven't already.
- Clone the repository: `git clone https://github.com/Mrmayman/rash.git`
- Change directory: `cd rash`
- Compile and run Rash: `cargo run --release -- path/to/file.sb3`
- To run the test suite, do: `cargo test`

Env vars:
- `RASH_PRINT_IR` to print the cranelift IR of the compiled code.
- `RASH_PRINT_FUNCTIONS` to print IR IDs of the callback functions.
- `RASH_PRINT_MEMORY` to print memory values after running.
- `RASH_SKIP_INLINE` to skip inlining functions
  (useful for debugging/blackbox benchmarking)

# Technical details

This includes:

- Parsing SB3 files
- Compiling Scratch blocks to cranelift IR
- Basic optimization, handling relocation, etc
- Custom coroutine implementation for Scratch screen refreshes
- Aggressive type analysis and type-related optimizations (almost zero-overhead dynamic types)
- Compile time effects analysis, transformations without sacrificing correctness
- Extensible architecture for VM
- A `wgpu` based renderer (basic, can be extended in future)

Rash currently doesn't support many Scratch features (pen, clones, broadcasts, lists, etc).
A roadmap can be found [here](./ROADMAP.md)

# Contributing

Feel free to submit any changes you make as a pull request, I'll be happy to review it.

# Note on AI coding

AI assistance was used in parts of this project only as a tool.
Most of the compiler itself, as well as the core architecture, remains strictly human.
Main use of AI here was for generating lots of test cases.
This project is NOT and will NEVER be "vibe-coded".
