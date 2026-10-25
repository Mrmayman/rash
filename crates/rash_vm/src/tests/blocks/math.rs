use super::{set_vars, utils::run_code};
use crate::{
    ScratchBlock,
    builder::{fadd, fdiv, fmod, fmul, fsub},
};

#[test]
fn math_add() {
    let memory = run_code(set_vars(vec![
        fadd(50.0, 25.0).into(),
        fadd(-500.0, 25.0).into(),
        fadd(-500.0, -25.0).into(),
        fadd(2.54, 6.25).into(),
        fadd(2.54, -6.25).into(),
        fadd(true, true).into(),
        fadd(1.0 / 0.0, 1.0 / 0.0).into(),
        fadd(1.0 / 0.0, -1.0 / 0.0).into(),
        fadd(-1.0 / 0.0, 1.0 / 0.0).into(),
        fadd(-1.0 / 0.0, -1.0 / 0.0).into(),
        fadd(1.0, f64::NAN).into(),
        fadd(f64::NAN, 1.0).into(),
        // FMA
        fadd(1.0, fmul(2.0, 3.0)).into(),
        fadd(fmul(2.0, 3.0), 1.0).into(),
    ]));
    assert_eq!(memory[0].convert_to_number(), 75.0);
    assert_eq!(memory[1].convert_to_number(), -475.0);
    assert_eq!(memory[2].convert_to_number(), -525.0);
    assert_eq!(memory[3].convert_to_number(), 8.79);
    assert_eq!(memory[4].convert_to_number(), -3.71);
    assert_eq!(memory[5].convert_to_number(), 2.0);

    assert!(memory[6].convert_to_number().is_infinite());
    assert!(memory[7].convert_to_number().is_nan());
    assert!(memory[8].convert_to_number().is_nan());
    assert!(memory[9].convert_to_number().is_infinite());
    assert!(memory[9].convert_to_number().is_sign_negative());

    assert_eq!(memory[10].convert_to_number(), 1.0);
    assert_eq!(memory[11].convert_to_number(), 1.0);

    assert_eq!(memory[12].convert_to_number(), 7.0);
    assert_eq!(memory[13].convert_to_number(), 7.0);
}

#[test]
fn math_sub() {
    let memory = run_code(set_vars(vec![
        fsub(50.0, 25.0).into(),
        fsub(-500.0, 25.0).into(),
        fsub(-500.0, -25.0).into(),
        fsub(2.54, 6.25).into(),
        fsub(2.54, -6.25).into(),
        fsub(true, true).into(),
        fsub(1.0 / 0.0, 1.0 / 0.0).into(),
        fsub(1.0 / 0.0, -1.0 / 0.0).into(),
        fsub(-1.0 / 0.0, 1.0 / 0.0).into(),
        fsub(-1.0 / 0.0, -1.0 / 0.0).into(),
        fsub(1.0, f64::NAN).into(),
        fsub(f64::NAN, 1.0).into(),
        // FMA
        fsub(fmul(2.0, 3.0), 1.0).into(),
    ]));
    assert_eq!(memory[0].convert_to_number(), 25.0);
    assert_eq!(memory[1].convert_to_number(), -525.0);
    assert_eq!(memory[2].convert_to_number(), -475.0);
    assert_eq!(memory[3].convert_to_number(), -3.71);
    assert_eq!(memory[4].convert_to_number(), 8.79);
    assert_eq!(memory[5].convert_to_number(), 0.0);

    assert!(memory[6].convert_to_number().is_nan());
    assert!(memory[7].convert_to_number().is_infinite());
    assert!(memory[7].convert_to_number().is_sign_positive());
    assert!(memory[8].convert_to_number().is_infinite());
    assert!(memory[8].convert_to_number().is_sign_negative());
    assert!(memory[9].convert_to_number().is_nan());

    assert_eq!(memory[10].convert_to_number(), 1.0);
    assert_eq!(memory[11].convert_to_number(), -1.0);

    assert_eq!(memory[12].convert_to_number(), 5.0);
}

#[test]
fn math_mul() {
    let memory = run_code(set_vars(vec![
        fmul(50.0, 2.0).into(),
        fmul(-50.0, 2.0).into(),
        fmul(-50.0, -2.0).into(),
        fmul(2.54, 6.25).into(),
        fmul(2.54, -6.25).into(),
        fmul(true, true).into(),
        fmul(1.0 / 0.0, 1.0 / 0.0).into(),
        fmul(1.0 / 0.0, 0.0).into(),
        fmul(1.0 / 0.0, 2.0).into(),
        fmul(1.0 / 0.0, -2.0).into(),
        fmul(1.0 / 0.0, -1.0 / 0.0).into(),
        fmul(-1.0 / 0.0, 1.0 / 0.0).into(),
        fmul(-1.0 / 0.0, 0.0).into(),
        fmul(-1.0 / 0.0, 2.0).into(),
        fmul(-1.0 / 0.0, -2.0).into(),
        fmul(-1.0 / 0.0, -1.0 / 0.0).into(),
        fmul(1.0, f64::NAN).into(),
        fmul(f64::NAN, 1.0).into(),
    ]));
    assert_eq!(memory[0].convert_to_number(), 100.0);
    assert_eq!(memory[1].convert_to_number(), -100.0);
    assert_eq!(memory[2].convert_to_number(), 100.0);
    assert_eq!(memory[3].convert_to_number(), 15.875);
    assert_eq!(memory[4].convert_to_number(), -15.875);
    assert_eq!(memory[5].convert_to_number(), 1.0);

    assert!(memory[6].convert_to_number().is_infinite());
    assert!(memory[6].convert_to_number().is_sign_positive());

    assert!(memory[7].convert_to_number().is_nan());

    assert!(memory[8].convert_to_number().is_infinite());
    assert!(memory[8].convert_to_number().is_sign_positive());

    assert!(memory[9].convert_to_number().is_infinite());
    assert!(memory[9].convert_to_number().is_sign_negative());

    assert!(memory[10].convert_to_number().is_infinite());
    assert!(memory[10].convert_to_number().is_sign_negative());

    assert!(memory[11].convert_to_number().is_infinite());
    assert!(memory[11].convert_to_number().is_sign_negative());

    assert!(memory[12].convert_to_number().is_nan());

    assert!(memory[13].convert_to_number().is_infinite());
    assert!(memory[13].convert_to_number().is_sign_negative());

    assert!(memory[14].convert_to_number().is_infinite());
    assert!(memory[14].convert_to_number().is_sign_positive());

    assert!(memory[15].convert_to_number().is_infinite());
    assert!(memory[15].convert_to_number().is_sign_positive());

    assert_eq!(memory[16].convert_to_number(), 0.0);
    assert_eq!(memory[17].convert_to_number(), 0.0);
}

#[test]
fn math_div() {
    let memory = run_code(set_vars(vec![
        fdiv(50.0, 2.0).into(),
        fdiv(-50.0, 2.0).into(),
        fdiv(-50.0, -2.0).into(),
        fdiv(3.5, 2.5).into(),
        fdiv(3.5, -2.5).into(),
        fdiv(true, true).into(),
        fdiv(1.0 / 0.0, 1.0 / 0.0).into(),
        fdiv(1.0 / 0.0, 0.0).into(),
        fdiv(1.0 / 0.0, 2.0).into(),
        fdiv(1.0 / 0.0, -2.0).into(),
        fdiv(1.0 / 0.0, -1.0 / 0.0).into(),
        fdiv(-1.0 / 0.0, 1.0 / 0.0).into(),
        fdiv(-1.0 / 0.0, 0.0).into(),
        fdiv(-1.0 / 0.0, 2.0).into(),
        fdiv(-1.0 / 0.0, -2.0).into(),
        fdiv(-1.0 / 0.0, -1.0 / 0.0).into(),
        fdiv(1.0, f64::NAN).into(),
        fdiv(f64::NAN, 1.0).into(),
    ]));
    assert_eq!(memory[0].convert_to_number(), 25.0);
    assert_eq!(memory[1].convert_to_number(), -25.0);
    assert_eq!(memory[2].convert_to_number(), 25.0);
    assert_eq!(memory[3].convert_to_number(), 1.4);
    assert_eq!(memory[4].convert_to_number(), -1.4);
    assert_eq!(memory[5].convert_to_number(), 1.0);

    assert!(memory[6].convert_to_number().is_nan());

    assert!(memory[7].convert_to_number().is_infinite());
    assert!(memory[7].convert_to_number().is_sign_positive());

    assert!(memory[8].convert_to_number().is_infinite());
    assert!(memory[8].convert_to_number().is_sign_positive());

    assert!(memory[9].convert_to_number().is_infinite());
    assert!(memory[9].convert_to_number().is_sign_negative());

    assert!(memory[10].convert_to_number().is_nan());
    assert!(memory[11].convert_to_number().is_nan());

    assert!(memory[12].convert_to_number().is_infinite());
    assert!(memory[12].convert_to_number().is_sign_negative());

    assert!(memory[13].convert_to_number().is_infinite());
    assert!(memory[13].convert_to_number().is_sign_negative());

    assert!(memory[14].convert_to_number().is_infinite());
    assert!(memory[14].convert_to_number().is_sign_positive());

    assert!(memory[15].convert_to_number().is_nan());

    assert!(memory[16].convert_to_number().is_infinite());
    assert!(memory[16].convert_to_number().is_sign_positive());

    assert_eq!(memory[17].convert_to_number(), 0.0);
}

#[test]
fn math_modulo() {
    let memory = run_code(set_vars(vec![
        fmod(5.5, 3.0).into(),    // 5.5 % 3.0
        fmod(-5.5, 3.0).into(),   // -5.5 % 3.0
        fmod(5.5, -3.0).into(),   // 5.5 % -3.0
        fmod(-5.5, -3.0).into(),  // -5.5 % -3.0
        fmod(10.0, 3.0).into(),   // 10.0 % 3.0
        fmod(-10.0, 3.0).into(),  // -10.0 % 3.0
        fmod(10.0, -3.0).into(),  // 10.0 % -3.0
        fmod(-10.0, -3.0).into(), // -10.0 % -3.0
        fmod(0.0, 1.0).into(),    // 0.0 % 1.0
        fmod(-1.0, 1.0).into(),   // -1.0 % 1.0
        fmod(1.0, -1.0).into(),   // 1.0 % -1.0
        fmod(1.0, 2.5).into(),    // 1.0 % 2.5
        fmod(-1.0, 2.5).into(),   // -1.0 % 2.5
        fmod(1e10, 3.0).into(),   // Large numbers
        fmod(-1e10, 3.0).into(),
        fmod(0.0001, 0.003).into(), // Small remainders
        fmod(-0.0001, 0.003).into(),
    ]));

    assert_eq!(memory[0].convert_to_number(), 2.5);
    assert!((memory[1].convert_to_number() - 0.5) <= f64::EPSILON);
    assert!((memory[2].convert_to_number() + 0.5).abs() <= f64::EPSILON);
    assert_eq!(memory[3].convert_to_number(), -2.5);

    assert!((memory[4].convert_to_number() - 1.0) <= 2.0 * f64::EPSILON);
    assert!((memory[5].convert_to_number() - 2.0).abs() <= 2.0 * f64::EPSILON);
    assert!((memory[6].convert_to_number() + 2.0).abs() <= 2.0 * f64::EPSILON);
    assert!((memory[7].convert_to_number() + 1.0).abs() <= 2.0 * f64::EPSILON);

    assert_eq!(memory[8].convert_to_number(), 0.0);
    assert_eq!(memory[9].convert_to_number(), 0.0);
    assert_eq!(memory[10].convert_to_number(), 0.0);

    assert_eq!(memory[11].convert_to_number(), 1.0);
    assert_eq!(memory[12].convert_to_number(), 1.5);

    assert!((memory[13].convert_to_number() - 1.0).abs() <= (i32::MAX as f64 + 1.0) * f64::EPSILON);
    assert!((memory[14].convert_to_number() - 2.0).abs() <= (i32::MAX as f64 + 1.0) * f64::EPSILON);

    assert_eq!(memory[15].convert_to_number(), 0.0001);
    assert!(memory[16].convert_to_number() - 0.0029 <= f64::EPSILON);
}

#[test]
fn math_floor() {
    let memory = run_code(set_vars(vec![
        ScratchBlock::OpFloor(5.5.into()).into(),
        ScratchBlock::OpFloor((-3.2).into()).into(),
        ScratchBlock::OpFloor(0.0.into()).into(),
        ScratchBlock::OpFloor((-0.8).into()).into(),
        ScratchBlock::OpFloor(2.999.into()).into(),
        ScratchBlock::OpFloor((-1.1).into()).into(),
        ScratchBlock::OpFloor(10.0.into()).into(),
        ScratchBlock::OpFloor((-10.999).into()).into(),
        ScratchBlock::OpFloor(123_456.789.into()).into(),
        ScratchBlock::OpFloor((-123_456.789).into()).into(),
        ScratchBlock::OpFloor(1e-9.into()).into(),
        ScratchBlock::OpFloor((-1e-9).into()).into(),
        ScratchBlock::OpFloor(1e10.into()).into(),
        ScratchBlock::OpFloor((-1e10).into()).into(),
        ScratchBlock::OpFloor(f64::NAN.into()).into(),
        ScratchBlock::OpFloor(f64::INFINITY.into()).into(),
        ScratchBlock::OpFloor(f64::NEG_INFINITY.into()).into(),
        ScratchBlock::OpFloor((-0.0).into()).into(),
        ScratchBlock::OpFloor((-0.5).into()).into(), // rounds toward -inf
        ScratchBlock::OpFloor(1e-320.into()).into(), // smallest subnormal range
        ScratchBlock::OpFloor((-1e-320).into()).into(), // subnormal, negative
        ScratchBlock::OpFloor(f64::MAX.into()).into(), // already integral
        ScratchBlock::OpFloor(f64::MIN.into()).into(), // already integral
        ScratchBlock::OpFloor((1.0 + f64::EPSILON).into()).into(), // nextafter(1, +inf)
        ScratchBlock::OpFloor((2.0 - f64::EPSILON).into()).into(), // nextafter(2, -inf)
    ]));

    assert_eq!(memory[0].convert_to_number(), 5.0);
    assert_eq!(memory[1].convert_to_number(), -4.0);
    assert_eq!(memory[2].convert_to_number(), 0.0);
    assert_eq!(memory[3].convert_to_number(), -1.0);
    assert_eq!(memory[4].convert_to_number(), 2.0);
    assert_eq!(memory[5].convert_to_number(), -2.0);
    assert_eq!(memory[6].convert_to_number(), 10.0);
    assert_eq!(memory[7].convert_to_number(), -11.0);
    assert_eq!(memory[8].convert_to_number(), 123456.0);
    assert_eq!(memory[9].convert_to_number(), -123457.0);
    assert_eq!(memory[10].convert_to_number(), 0.0);
    assert_eq!(memory[11].convert_to_number(), -1.0);
    assert_eq!(memory[12].convert_to_number(), 1e10);
    assert_eq!(memory[13].convert_to_number(), -1e10);

    // IEEE-754 edge cases
    assert_eq!(memory[14].convert_to_number(), 0.0);
    assert_eq!(memory[15].convert_to_number(), f64::INFINITY);
    assert_eq!(memory[16].convert_to_number(), f64::NEG_INFINITY);
    // floor(-0.0) == -0.0, which == 0.0 under IEEE comparison
    assert_eq!(memory[17].convert_to_number(), 0.0);
    assert_eq!(memory[18].convert_to_number(), -1.0); // -0.5 -> -1
    assert_eq!(memory[19].convert_to_number(), 0.0); // subnormal -> 0
    assert_eq!(memory[20].convert_to_number(), -1.0); // -subnormal -> -1
    assert_eq!(memory[21].convert_to_number(), f64::MAX); // identity above 2^52
    assert_eq!(memory[22].convert_to_number(), f64::MIN); // identity above 2^52
    assert_eq!(memory[23].convert_to_number(), 1.0); // nextafter(1) still floors to 1
    assert_eq!(memory[24].convert_to_number(), 1.0); // nextafter(2, 0) floors to 1
}

#[test]
fn math_ceil() {
    let memory = run_code(set_vars(vec![
        ScratchBlock::OpCeil(5.5.into()).into(),
        ScratchBlock::OpCeil((-3.2).into()).into(),
        ScratchBlock::OpCeil(0.0.into()).into(),
        ScratchBlock::OpCeil((-0.8).into()).into(),
        ScratchBlock::OpCeil(2.999.into()).into(),
        ScratchBlock::OpCeil((-1.1).into()).into(),
        ScratchBlock::OpCeil(10.0.into()).into(),
        ScratchBlock::OpCeil((-10.999).into()).into(),
        ScratchBlock::OpCeil(123_456.789.into()).into(),
        ScratchBlock::OpCeil((-123_456.789).into()).into(),
        ScratchBlock::OpCeil(1e-9.into()).into(),
        ScratchBlock::OpCeil((-1e-9).into()).into(),
        ScratchBlock::OpCeil(1e10.into()).into(),
        ScratchBlock::OpCeil((-1e10).into()).into(),
        ScratchBlock::OpCeil(f64::NAN.into()).into(),
        ScratchBlock::OpCeil(f64::INFINITY.into()).into(),
        ScratchBlock::OpCeil(f64::NEG_INFINITY.into()).into(),
        ScratchBlock::OpCeil((-0.0).into()).into(),
        ScratchBlock::OpCeil((-0.5).into()).into(), // rounds toward +inf -> -0
        ScratchBlock::OpCeil(1e-320.into()).into(), // smallest subnormal range -> 1
        ScratchBlock::OpCeil((-1e-320).into()).into(), // -subnormal -> -0
        ScratchBlock::OpCeil(f64::MAX.into()).into(), // already integral
        ScratchBlock::OpCeil(f64::MIN.into()).into(), // already integral
        ScratchBlock::OpCeil((1.0 + f64::EPSILON).into()).into(), // nextafter(1, +inf) -> 2
        ScratchBlock::OpCeil((2.0 - f64::EPSILON).into()).into(), // nextafter(2, -inf) -> 2
    ]));

    assert_eq!(memory[0].convert_to_number(), 6.0);
    assert_eq!(memory[1].convert_to_number(), -3.0);
    assert_eq!(memory[2].convert_to_number(), 0.0);
    assert_eq!(memory[3].convert_to_number(), 0.0); // -0.0 == 0.0 numerically
    assert_eq!(memory[4].convert_to_number(), 3.0);
    assert_eq!(memory[5].convert_to_number(), -1.0);
    assert_eq!(memory[6].convert_to_number(), 10.0);
    assert_eq!(memory[7].convert_to_number(), -10.0);
    assert_eq!(memory[8].convert_to_number(), 123457.0);
    assert_eq!(memory[9].convert_to_number(), -123456.0);
    assert_eq!(memory[10].convert_to_number(), 1.0); // any tiny positive -> 1
    assert_eq!(memory[11].convert_to_number(), 0.0); // any tiny negative -> -0.0
    assert_eq!(memory[12].convert_to_number(), 1e10);
    assert_eq!(memory[13].convert_to_number(), -1e10);

    // IEEE-754 edge cases
    assert_eq!(memory[14].convert_to_number(), 0.0);
    assert_eq!(memory[15].convert_to_number(), f64::INFINITY);
    assert_eq!(memory[16].convert_to_number(), f64::NEG_INFINITY);
    assert_eq!(memory[17].convert_to_number(), 0.0); // ceil(-0.0) == -0.0
    assert_eq!(memory[18].convert_to_number(), 0.0); // -0.5 -> -0.0
    assert_eq!(memory[19].convert_to_number(), 1.0); // subnormal -> 1
    assert_eq!(memory[20].convert_to_number(), 0.0); // -subnormal -> -0.0
    assert_eq!(memory[21].convert_to_number(), f64::MAX); // identity above 2^52
    assert_eq!(memory[22].convert_to_number(), f64::MIN); // identity above 2^52
    assert_eq!(memory[23].convert_to_number(), 2.0); // nextafter(1) ceils to 2
    assert_eq!(memory[24].convert_to_number(), 2.0); // nextafter(2, 0) ceils to 2
}

#[test]
fn math_round() {
    let memory = run_code(set_vars(vec![
        ScratchBlock::OpRound(2.3.into()).into(),
        ScratchBlock::OpRound(2.5.into()).into(),
        ScratchBlock::OpRound(2.7.into()).into(),
        ScratchBlock::OpRound(3.0.into()).into(),
        ScratchBlock::OpRound((-2.3).into()).into(),
        ScratchBlock::OpRound((-2.5).into()).into(),
        ScratchBlock::OpRound((-2.7).into()).into(),
        ScratchBlock::OpRound((-3.0).into()).into(),
    ]));
    assert_eq!(memory[0].convert_to_number(), 2.0);
    assert_eq!(memory[1].convert_to_number(), 3.0);
    assert_eq!(memory[2].convert_to_number(), 3.0);
    assert_eq!(memory[3].convert_to_number(), 3.0);
    assert_eq!(memory[4].convert_to_number(), -2.0);
    assert_eq!(memory[5].convert_to_number(), -2.0);
    assert_eq!(memory[6].convert_to_number(), -3.0);
    assert_eq!(memory[7].convert_to_number(), -3.0);
}

use std::f64::consts::E;

#[test]
fn math_log_base10() {
    let memory = run_code(set_vars(vec![
        // exact powers of ten (should be very close to integer results)
        ScratchBlock::OpLog(1000.0.into()).into(),
        ScratchBlock::OpLog(100.0.into()).into(),
        ScratchBlock::OpLog(10.0.into()).into(),
        ScratchBlock::OpLog(1.0.into()).into(),
        ScratchBlock::OpLog(0.1.into()).into(),
        ScratchBlock::OpLog(1e-300.into()).into(), // subnormal, still finite
        // IEEE‑754 special cases
        ScratchBlock::OpLog(0.0.into()).into(),    // -Infinity
        ScratchBlock::OpLog((-0.0).into()).into(), // -Infinity (negative zero)
        ScratchBlock::OpLog((-1.0).into()).into(), // NaN (log of negative)
        ScratchBlock::OpLog(f64::NAN.into()).into(),
        ScratchBlock::OpLog(f64::INFINITY.into()).into(),
        ScratchBlock::OpLog(f64::NEG_INFINITY.into()).into(), // NaN
    ]));

    // ordinary finite results – allow a tiny tolerance for floating‑point rounding
    let approx = |a: f64, b: f64| (a - b).abs() < 1e-12;

    assert!(approx(memory[0].convert_to_number(), 3.0));
    assert!(approx(memory[1].convert_to_number(), 2.0));
    assert!(approx(memory[2].convert_to_number(), 1.0));
    assert!(approx(memory[3].convert_to_number(), 0.0));
    assert!(approx(memory[4].convert_to_number(), -1.0));
    assert!(approx(memory[5].convert_to_number(), -300.0));

    // IEEE‑754 special values
    assert_eq!(memory[6].convert_to_number(), f64::NEG_INFINITY);
    assert_eq!(memory[7].convert_to_number(), f64::NEG_INFINITY);
    assert!(memory[8].convert_to_number().is_nan());
    assert_eq!(memory[9].convert_to_number(), f64::NEG_INFINITY);
    assert_eq!(memory[10].convert_to_number(), f64::INFINITY);
    assert!(memory[11].convert_to_number().is_nan());
}

#[test]
fn math_ln() {
    let memory = run_code(set_vars(vec![
        // ordinary finite values
        ScratchBlock::OpLn(E.into()).into(),      // ln(e) ≈ 1
        ScratchBlock::OpLn(1.0.into()).into(),    // ln(1) = 0
        ScratchBlock::OpLn(2.0.into()).into(),    // ln(2) ≈ 0.693147...
        ScratchBlock::OpLn(1e-300.into()).into(), // ln(1e-300) ≈ -690.7755...
        // IEEE‑754 special cases
        ScratchBlock::OpLn(0.0.into()).into(),    // -Infinity
        ScratchBlock::OpLn((-0.0).into()).into(), // -Infinity
        ScratchBlock::OpLn((-1.0).into()).into(), // NaN
        ScratchBlock::OpLn(f64::NAN.into()).into(),
        ScratchBlock::OpLn(f64::INFINITY.into()).into(),
        ScratchBlock::OpLn(f64::NEG_INFINITY.into()).into(), // NaN
    ]));

    let approx = |a: f64, b: f64| (a - b).abs() < 1e-12;

    assert!(approx(memory[0].convert_to_number(), 1.0));
    assert!(approx(memory[1].convert_to_number(), 0.0));
    assert!(approx(memory[2].convert_to_number(), 2.0_f64.ln()));
    assert!(approx(memory[3].convert_to_number(), (1e-300_f64).ln()));

    assert_eq!(memory[4].convert_to_number(), f64::NEG_INFINITY);
    assert_eq!(memory[5].convert_to_number(), f64::NEG_INFINITY);
    assert!(memory[6].convert_to_number().is_nan());
    assert_eq!(memory[7].convert_to_number(), f64::NEG_INFINITY);
    assert_eq!(memory[8].convert_to_number(), f64::INFINITY);
    assert!(memory[9].convert_to_number().is_nan());
}

#[test]
fn math_exp() {
    let memory = run_code(set_vars(vec![
        ScratchBlock::OpExp(0.0.into()).into(),          // e^0 = 1
        ScratchBlock::OpExp(1.0.into()).into(),          // e^1 = e
        ScratchBlock::OpExp(2.0_f64.ln().into()).into(), // e^ln(2) = 2
        ScratchBlock::OpExp((-1.0).into()).into(),       // e^-1 = 1/e
        ScratchBlock::OpExp((-0.0_f64).into()).into(),   // e^-0 = 1 (negative zero)
        ScratchBlock::OpExp(1000.0.into()).into(),       // overflow  -> +Infinity
        ScratchBlock::OpExp((-1000.0).into()).into(),    // underflow -> +0.0
        ScratchBlock::OpExp(f64::NAN.into()).into(),
        ScratchBlock::OpExp(f64::INFINITY.into()).into(),
        ScratchBlock::OpExp(f64::NEG_INFINITY.into()).into(), // e^-Inf -> +0
    ]));

    let approx = |a: f64, b: f64| (a - b).abs() < 1e-12;

    assert!(approx(memory[0].convert_to_number(), 1.0));
    assert!(approx(memory[1].convert_to_number(), E));
    assert!(approx(memory[2].convert_to_number(), 2.0));
    assert!(approx(memory[3].convert_to_number(), 1.0 / E));
    assert!(approx(memory[4].convert_to_number(), 1.0));

    // IEEE-754 special values (exact)
    assert_eq!(memory[5].convert_to_number(), f64::INFINITY);
    assert_eq!(memory[6].convert_to_number(), 0.0);
    assert_eq!(memory[7].convert_to_number(), 1.0);
    assert_eq!(memory[8].convert_to_number(), f64::INFINITY);
    assert_eq!(memory[9].convert_to_number(), 0.0);
}

#[test]
fn math_10exp() {
    let memory = run_code(set_vars(vec![
        ScratchBlock::OpExp10(0.0.into()).into(),        // 10^0  = 1
        ScratchBlock::OpExp10(1.0.into()).into(),        // 10^1  = 10
        ScratchBlock::OpExp10(2.0.into()).into(),        // 10^2  = 100
        ScratchBlock::OpExp10(3.0.into()).into(),        // 10^3  = 1000
        ScratchBlock::OpExp10((-1.0).into()).into(),     // 10^-1 = 0.1
        ScratchBlock::OpExp10((-0.0_f64).into()).into(), // 10^-0 = 1 (negative zero)
        ScratchBlock::OpExp10(0.5.into()).into(),        // 10^0.5 = sqrt(10)
        ScratchBlock::OpExp10(308.0.into()).into(),      // ~1e308, still finite
        ScratchBlock::OpExp10(309.0.into()).into(),      // overflow  -> +Infinity
        ScratchBlock::OpExp10((-320.0).into()).into(),   // ~1e-320, subnormal
        ScratchBlock::OpExp10((-400.0).into()).into(),   // underflow -> +0.0
        ScratchBlock::OpExp10(f64::NAN.into()).into(),
        ScratchBlock::OpExp10(f64::INFINITY.into()).into(),
        ScratchBlock::OpExp10(f64::NEG_INFINITY.into()).into(), // 10^-Inf -> +0
    ]));

    let approx = |a: f64, b: f64| (a - b).abs() < 1e-12;
    let rel_approx = |a: f64, b: f64| (a - b).abs() <= 1e-10 * b.abs();

    assert!(approx(memory[0].convert_to_number(), 1.0));
    assert!(approx(memory[1].convert_to_number(), 10.0));
    assert!(approx(memory[2].convert_to_number(), 100.0));
    assert!(approx(memory[3].convert_to_number(), 1000.0));
    assert!(approx(memory[4].convert_to_number(), 0.1));
    assert!(approx(memory[5].convert_to_number(), 1.0));
    assert!(approx(memory[6].convert_to_number(), 10.0_f64.sqrt()));

    // largest finite power-of-ten result and overflow just past it
    let big = memory[7].convert_to_number();
    assert!(big.is_finite());
    assert!(rel_approx(big, 1e308));
    assert_eq!(memory[8].convert_to_number(), f64::INFINITY);

    // subnormal range: positive but below the smallest *normal* double
    let sub = memory[9].convert_to_number();
    assert!(sub > 0.0);
    assert!(sub < f64::MIN_POSITIVE); // i.e. it really is subnormal
    assert!(rel_approx(sub, 1e-320));

    assert_eq!(memory[10].convert_to_number(), 0.0);
    assert_eq!(memory[11].convert_to_number(), 1.0);
    assert_eq!(memory[12].convert_to_number(), f64::INFINITY);
    assert_eq!(memory[13].convert_to_number(), 0.0);
}

#[test]
fn math_abs() {
    let memory = run_code(set_vars(vec![
        ScratchBlock::OpAbs(2.3.into()).into(),
        ScratchBlock::OpAbs((-2.3).into()).into(),
        ScratchBlock::OpAbs(0.0.into()).into(),
        ScratchBlock::OpAbs((-0.0).into()).into(),
        ScratchBlock::OpAbs(f64::INFINITY.into()).into(),
        ScratchBlock::OpAbs(f64::NEG_INFINITY.into()).into(),
    ]));
    assert_eq!(memory[0].convert_to_number(), 2.3);
    assert_eq!(memory[1].convert_to_number(), 2.3);
    assert_eq!(memory[2].convert_to_number(), 0.0);
    assert_eq!(memory[3].convert_to_number(), 0.0);
    assert_eq!(memory[4].convert_to_number(), f64::INFINITY);
    assert_eq!(memory[5].convert_to_number(), f64::INFINITY);
}

#[test]
fn math_sqrt() {
    let memory = run_code(set_vars(vec![
        ScratchBlock::OpSqrt(1.0.into()).into(),
        ScratchBlock::OpSqrt(2.0.into()).into(),
        ScratchBlock::OpSqrt(0.0.into()).into(),
        ScratchBlock::OpSqrt((-0.0).into()).into(),
        ScratchBlock::OpSqrt((-1.0).into()).into(),
        ScratchBlock::OpSqrt(f64::INFINITY.into()).into(),
        ScratchBlock::OpSqrt(f64::NEG_INFINITY.into()).into(),
    ]));
    assert_eq!(memory[0].convert_to_number(), 1.0);
    assert_eq!(memory[1].convert_to_number(), 1.4142135623730951);
    assert_eq!(memory[2].convert_to_number(), 0.0);
    assert_eq!(memory[3].convert_to_number(), 0.0);
    assert!(memory[4].convert_to_number().is_nan());
    assert!(memory[5].convert_to_number().is_infinite());
    assert!(memory[5].convert_to_number().is_sign_positive());
    assert!(memory[6].convert_to_number().is_nan());
}

#[test]
fn math_trig() {
    let memory = run_code(set_vars(vec![
        ScratchBlock::OpSin(0.0.into()).into(),
        ScratchBlock::OpSin(30.0.into()).into(),
        ScratchBlock::OpSin(60.0.into()).into(),
        ScratchBlock::OpSin(90.0.into()).into(),
        ScratchBlock::OpSin(100.0.into()).into(),
        ScratchBlock::OpSin((-30.0).into()).into(),
        ScratchBlock::OpSin((-60.0).into()).into(),
        ScratchBlock::OpSin((-90.0).into()).into(),
        ScratchBlock::OpSin((-100.0).into()).into(),
        ScratchBlock::OpCos(0.0.into()).into(),
        ScratchBlock::OpCos(30.0.into()).into(),
        ScratchBlock::OpCos(60.0.into()).into(),
        ScratchBlock::OpCos(90.0.into()).into(),
        ScratchBlock::OpCos(100.0.into()).into(),
        ScratchBlock::OpCos((-30.0).into()).into(),
        ScratchBlock::OpCos((-60.0).into()).into(),
        ScratchBlock::OpCos((-90.0).into()).into(),
        ScratchBlock::OpCos((-100.0).into()).into(),
        ScratchBlock::OpTan(0.0.into()).into(),
        ScratchBlock::OpTan(30.0.into()).into(),
        ScratchBlock::OpTan(60.0.into()).into(),
        ScratchBlock::OpTan(90.0.into()).into(),
        ScratchBlock::OpTan(100.0.into()).into(),
        ScratchBlock::OpTan(180.0.into()).into(),
        ScratchBlock::OpTan(270.0.into()).into(),
        ScratchBlock::OpTan((-30.0).into()).into(),
        ScratchBlock::OpTan((-60.0).into()).into(),
        ScratchBlock::OpTan((-90.0).into()).into(),
        ScratchBlock::OpTan((-100.0).into()).into(),
        ScratchBlock::OpTan((-180.0).into()).into(),
        ScratchBlock::OpTan((-270.0).into()).into(),
    ]));

    // SIN

    assert_eq!(memory[0].convert_to_number(), 0.0);

    assert!(0.5 - memory[1].convert_to_number() < f64::EPSILON);
    assert!(0.8660254038 - memory[2].convert_to_number() <= 80000.0 * f64::EPSILON);
    assert_eq!(memory[3].convert_to_number(), 1.0);
    assert!(memory[4].convert_to_number() - 0.984807753 <= 60000.0 * f64::EPSILON);

    assert!(memory[5].convert_to_number() - 0.5 < f64::EPSILON);
    assert!(memory[6].convert_to_number() - 0.8660254038 <= 80000.0 * f64::EPSILON);
    assert_eq!(memory[7].convert_to_number(), -1.0);
    assert!(memory[8].convert_to_number() + 0.984807753 <= 60000.0 * f64::EPSILON);

    // COS

    assert_eq!(memory[9].convert_to_number(), 1.0);

    assert!(0.8660254038 - memory[10].convert_to_number() <= 80000.0 * f64::EPSILON);
    assert!(0.5 - memory[11].convert_to_number() < f64::EPSILON);
    assert!(memory[12].convert_to_number() < f64::EPSILON);
    assert!(memory[13].convert_to_number() + 0.1736481777 < 150000.0 * f64::EPSILON);

    assert!(memory[14].convert_to_number() - 0.8660254038 <= 80000.0 * f64::EPSILON);
    assert!(memory[15].convert_to_number() - 0.5 < f64::EPSILON);
    assert!(memory[16].convert_to_number() < f64::EPSILON);
    assert!(memory[17].convert_to_number() + 0.1736481777 < 150000.0 * f64::EPSILON);

    // TAN

    assert_eq!(memory[18].convert_to_number(), 0.0);

    assert!((memory[19].convert_to_number() - 0.5773502692).abs() < 46722.0 * f64::EPSILON);
    assert!((memory[20].convert_to_number() - 1.7320508076).abs() < 140168.0 * f64::EPSILON);
    assert!(memory[21].convert_to_number().is_infinite());
    assert!(memory[21].convert_to_number().is_sign_positive());
    assert!(memory[22].convert_to_number() - 5.6712818196 < 79765.0 * f64::EPSILON);
    assert!(memory[23].convert_to_number().abs() < f64::EPSILON);
    assert!(memory[24].convert_to_number().is_infinite());
    assert!(memory[24].convert_to_number().is_sign_negative());

    assert!((memory[25].convert_to_number() + 0.5773502692).abs() < 46722.0 * f64::EPSILON);
    assert!((memory[26].convert_to_number() + 1.7320508076).abs() < 140168.0 * f64::EPSILON);
    assert!(memory[27].convert_to_number().is_infinite());
    assert!(memory[27].convert_to_number().is_sign_negative());
    assert!(memory[28].convert_to_number() - 5.6712818196 < 79765.0 * f64::EPSILON);
    assert!(memory[29].convert_to_number() < f64::EPSILON);
    assert!(memory[30].convert_to_number().is_infinite());
    assert!(memory[30].convert_to_number().is_sign_positive());
}
