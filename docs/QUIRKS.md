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
