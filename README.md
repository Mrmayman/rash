# rash
A Scratch JIT compiler written in rust using [cranelift](https://cranelift.dev/).

# Hmm... What?

Scratch is a visual programming language aimed at children,
and people push it to its limits (with projects like path tracers,
3d minecraft clones, neural networks and more),
running into performance bottlenecks.

Rash aims to run scratch code with a JIT compiler, where the code is
compiled to machine code and executed directly, much like Java.

# What's the progress?

Not much right now. Many basic operators are done, and basic logic is implemented.
But there's still a lot left. Rash is currently command line only, and there's no
graphical output yet.

You can see the progress in implementing blocks at
https://docs.google.com/spreadsheets/d/1jYi5lsAyq6XeJPCKCpk4UkqF1YWVPX9C4d7-eTbXw9U/edit?usp=sharing

# Running

Clone the repository:

`git clone https://github.com/Mrmayman/rash.git`

Change directory:

`cd rash`

Install the Rust language if you haven't already.
Run the project:

`cargo run --release`

To run the test suite, do:

`cargo test`

Note: It can't load SB3 files now, the code is hardcoded (more info in `src/main.rs`).

# Contributing

Feel free to submit any changes you make as a pull request, I'll be happy to review it.

# Benchmarks

Pi calculation:

- Scratch: `2803 ms`
- Turbowarp: `30 ms`
- Rash: `6.3 ms`

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
