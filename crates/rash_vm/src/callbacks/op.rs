use rand::RngExt;

use crate::ScratchObject;

declare_module!(
    "op.rs", 3, floor, sin, cos, tan, round, cmp, random, ceil, trunc, nearest, fma
);

pub extern "C" fn sin(value: f64) -> f64 {
    value.to_radians().sin()
}

pub extern "C" fn cos(value: f64) -> f64 {
    value.to_radians().cos()
}

pub extern "C" fn tan(value: f64) -> f64 {
    match (value + 90.0) % 360.0 {
        0.0 => f64::NEG_INFINITY,
        180.0 | -180.0 => f64::INFINITY,
        _ => value.to_radians().tan(),
    }
}

pub extern "C" fn round(value: f64) -> f64 {
    if (value - value.trunc()).abs() == 0.5 {
        // If number ends with .5 round up (Scratch behaviour).
        // Override of Rust behaviour
        value.ceil()
    } else {
        value.round()
    }
}

/// Callback to compare objects (Scratch-spec), returns i8-shaped value
pub unsafe extern "C" fn cmp(
    a1: i64,
    a2: i64,
    a3: i64,
    a4: i64,
    b1: i64,
    b2: i64,
    b3: i64,
    b4: i64,
) -> i64 {
    let a: ScratchObject = unsafe { std::mem::transmute([a1, a2, a3, a4]) };
    let b: ScratchObject = unsafe { std::mem::transmute([b1, b2, b3, b4]) };
    a.scratch_cmp(&b) as i64
}

/// Callback from JIT code to generate a random number.
///
/// # Arguments
/// * `a` - The lower bound of the random number.
/// * `b` - The upper bound of the random number.
/// * `is_decimal` - Whether the number should be a decimal
///   (eg: 3.1415) or round (eg: 3.0). If `is_decimal` is 1,
///   the number will be a decimal. Represented this way for simplicity.
pub extern "C" fn random(a: f64, b: f64, is_decimal: i64) -> f64 {
    let mut rng = rand::rng();
    let num = rng.random_range(if a > b { b..=a } else { a..=b });
    if is_decimal == 1 { num } else { num.round() }
}

// ── Libcall implementations ──────────────────────────────────────────
// These are emitted by Cranelift when the target ISA lacks a native
// instruction for ceil/floor/trunc/nearest/fma.

pub extern "C" fn floor_f32(value: f32) -> f32 {
    value.floor()
}

pub extern "C" fn floor(value: f64) -> f64 {
    value.floor()
}

pub extern "C" fn ceil_f32(value: f32) -> f32 {
    value.ceil()
}

pub extern "C" fn ceil(value: f64) -> f64 {
    value.ceil()
}

pub extern "C" fn trunc_f32(value: f32) -> f32 {
    value.trunc()
}

pub extern "C" fn trunc(value: f64) -> f64 {
    value.trunc()
}

pub extern "C" fn nearest_f32(value: f32) -> f32 {
    // IEEE 754 "round to nearest, ties to even"
    value.round_ties_even()
}

pub extern "C" fn nearest(value: f64) -> f64 {
    value.round_ties_even()
}

pub extern "C" fn fma_f32(a: f32, b: f32, c: f32) -> f32 {
    a.mul_add(b, c)
}

pub extern "C" fn fma(a: f64, b: f64, c: f64) -> f64 {
    a.mul_add(b, c)
}
