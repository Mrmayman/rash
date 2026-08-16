# rash
A Scratch JIT compiler written in rust using [cranelift](https://cranelift.dev/).

> **Warning:** This is highly experimental, incomplete software.
> If you want an existing production-ready solution, see [scratchcpp](https://github.com/scratchcpp/scratchcpp-player/)

# What is this?

Scratch is a visual programming language aimed at children,
and people push it to its limits (with projects like path tracers,
3d minecraft clones, neural networks and more),
running into performance bottlenecks.

Rash aims to run scratch code with a JIT compiler, where the code is
compiled to machine code and executed directly, much like Java, C#, JavaScript, etc.

# Benchmarks

Pi calculation (10,000,000 iterations)

**System**: Ryzen 7 7840HS, 24 GB RAM, Fedora GNOME

| **Platform** | **Time** |
| --- | --- |
| Scratch | \~6770 ms (6.8s) |
| [ScratchCPP](https://github.com/scratchcpp/scratchcpp-player/) | \~126 ms |
| Turbowarp (Warp timer on) | \~112 ms |
| Turbowarp | \~72 ms |
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

## Todo list:

(WIP means Work-In-Progress)

- [ ] Implement loading from SB3 files (WIP)
- [ ] Implement Control operations
  - [x] If, If-Else
  - [x] Repeat, Repeat Until, Forever
  - [ ] Wait
  - [ ] Wait Until
  - [x] Stop this script
  - [ ] Stop all
  - [ ] Stop other scripts in sprite
- [ ] Math Operations
  - [x] Add, subtract, multiply, divide
  - [x] && || !
  - [x] < >
  - [ ] ==
  - [x] String: Join, Contains, Length
  - [x] Mod, Round, Abs
  - [x] Floor
  - [ ] Ceiling
  - [x] Sqrt
  - [x] Sin, Cos, Tan
  - [ ] ASin, ACos, ATan
  - [ ] Ln, Log
  - [ ] E^, 10^
- [ ] Other blocks
  - [x] Days since 2000
  - [ ] Timer, reset timer
  - [ ] Keyboard/mouse input
- [ ] Core features
  - [x] Custom Blocks
  - [x] Variables
  - [ ] Lists
  - [ ] Broadcasts
  - [ ] Clones
- [x] Add Graphics
  - [ ] Hide, Show blocks
  - [x] Position blocks
  - [ ] Rotation blocks
  - [ ] Size block
  - [x] Render sprites & stage
  - [ ] Render text
  - [ ] Render speech and thought bubbles
  - [ ] Sprite costumes and backdrops
  - [ ] Sprite graphical effects (Ghost, Fisheye, etc)
  - [ ] Pen canvas and clear operation
  - [ ] Pen stamps
  - [ ] Pen lines
  - [ ] Variable monitors
  - [ ] UI library
- [ ] Add sound

# Contributing

Feel free to submit any changes you make as a pull request, I'll be happy to review it.
