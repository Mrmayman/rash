# rash
A Scratch JIT compiler written in rust using [cranelift](https://cranelift.dev/).

# What?

Scratch is a visual programming language aimed at children,
and people push it to its limits (with projects like path tracers,
3d minecraft clones, neural networks and more),
running into performance bottlenecks.

Rash aims to run scratch code with a JIT compiler, where the code is
compiled to machine code and executed directly, much like Java, C#, JavaScript, etc.

# What's the progress?

You can see the progress in implementing blocks in this [Google Docs spreadsheet](https://docs.google.com/spreadsheets/d/1jYi5lsAyq6XeJPCKCpk4UkqF1YWVPX9C4d7-eTbXw9U/edit?usp=sharing)

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
  - [ ] < > (incomplete)
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

# Running

- Install the Rust language if you haven't already.
- Clone the repository: `git clone https://github.com/Mrmayman/rash.git`
- Change directory: `cd rash`
- Compile and run Rash: `cargo run --release -- path/to/file.sb3`
- To run the test suite, do: `cargo test`

# Contributing

Feel free to submit any changes you make as a pull request, I'll be happy to review it.

# Benchmarks

Pi calculation:

- Scratch: `621 ms`
- Turbowarp: `13 ms`
- Rash: `7 ms`

# Quirks

Here are some weird Scratch quirks I've come across while creating this interpreter.

## NaN Handling

In *normal* programming languages, any operation with a `NaN` value will return `NaN`. For example, `69.0 + NaN = NaN`.

However, in Scratch, any operation with a `NaN` value will be treated as an operation with zero. For example, `69.0 + NaN` will be treated as `69.0 + 0.0 = 69.0`.

The check for this is basically (in pseudocode): `if n == n { n } else { 0.0 }`. This hurts performance noticeably but is essential for correctness.

## String encoding

Scratch (appears to) encode strings in UTF-16, as opposed to Rust which encodes them in UTF-8. Basically normal text remains the same, but emojis are treated differently.

In Scratch, the skull emoji for example (💀) would be 2 characters, both invalid. `letter(1) of "💀"` and `letter(2) of "💀"` would return invalid characters, and `length of "💀"` would return *2*.

However, in Rust, the skull emoji (💀) would be 4 characters, and `"💀".len() == 4`. To fix compatibility, `str::encode_utf16()` is used.
