use crate::{ScratchObject, stat};

/// Tests for checking the conversion between values of different types.
///
/// Based on
/// https://github.com/scratchcpp/libscratchcpp/blob/5e1e3b62ae2e5198da2ca8f7d32890abbdf75b91/test/scratch_classes/value_test.cpp
///
/// Massive credit to adazem009

#[test]
fn conversion_bool() {
    assert_eq!(ScratchObject::String(stat("True")).convert_to_bool(), true);
    assert_eq!(ScratchObject::String(stat("true")).convert_to_bool(), true);
    assert_eq!(
        ScratchObject::String(stat("false")).convert_to_bool(),
        false
    );
    assert_eq!(
        ScratchObject::String(stat("False")).convert_to_bool(),
        false
    );

    assert_eq!(ScratchObject::String(stat("1")).convert_to_bool(), true);
    assert_eq!(ScratchObject::String(stat("0")).convert_to_bool(), false);

    assert_eq!(ScratchObject::String(stat("1.0")).convert_to_bool(), true);
    assert_eq!(ScratchObject::String(stat("0.0")).convert_to_bool(), true);
    assert_eq!(ScratchObject::String(stat("-1")).convert_to_bool(), true);
    assert_eq!(ScratchObject::String(stat("-1.0")).convert_to_bool(), true);
    assert_eq!(ScratchObject::String(stat("0e10")).convert_to_bool(), true);
    assert_eq!(ScratchObject::String(stat("")).convert_to_bool(), false);

    assert_eq!(ScratchObject::Number(20.0).convert_to_bool(), true);
    assert_eq!(ScratchObject::Number(-20.0).convert_to_bool(), true);
    assert_eq!(ScratchObject::Number(1.0).convert_to_bool(), true);
    assert_eq!(ScratchObject::Number(0.0).convert_to_bool(), false);
    assert_eq!(ScratchObject::Number(f64::NAN).convert_to_bool(), false);
}

macro_rules! string_to_number {
    ($(($input_str:expr, $expected:expr)),* $(,)?) => {
        $(
            assert_eq!(
                ScratchObject::String(stat($input_str)).convert_to_number(),
                $expected
            );
        )*
    };
}

#[test]
fn conversion_number() {
    string_to_number!(
        ("2147483647", 2147483647.0),
        ("-2147483647", -2147483647.0),
        ("255.625", 255.625),
        ("-255.625", -255.625),
        ("0.15", 0.15),
        ("-0.15", -0.15),
        ("0", 0.0),
        ("0.0", 0.0),
        ("-0", -0.0),
        ("-0.0", -0.0),
        ("+.15", 0.15),
        (".15", 0.15),
        ("-.15", -0.15),
        ("0+5", 0.0),
        ("0-5", 0.0),
        ("9432.4e-12", 9.4324e-9),
        ("-9432.4e-12", -9.4324e-9),
        ("9432.4e6", 9.4324e+9),
        ("9432.4e+6", 9.4324e+9),
        ("-9432.4e+6", -9.4324e+9),
        ("1 2 3", 0.0),
        ("false", 0.0),
        ("true", 0.0),
        // TODO: Infinity > 0, -Infinity < 0
        ("NaN", 0.0),
        ("something", 0.0),
        // Hexadecimal
        ("0xafe", 2814.0),
        ("0xafe", 2814.0),
        ("   0xafe", 2814.0),
        ("0xafe   ", 2814.0),
        ("   0xafe   ", 2814.0),
        ("0x0afe", 2814.0),
        ("0xBaCD", 47821.0),
        ("0XBaCD", 47821.0),
        ("0xAbG", 0.0),
        ("0xabf.d", 0.0),
        ("+0xa", 0.0),
        ("-0xa", 0.0),
        ("0x+a", 0.0),
        ("0x-a", 0.0),
        // Octal
        ("0o506", 326.0),
        ("   0o506", 326.0),
        ("0o506", 326.0),
        ("   0o506   ", 326.0),
        ("0o0506", 326.0),
        ("0O17206", 7814.0),
        ("0o5783", 0.0),
        ("0o573.2", 0.0),
        ("+0o2", 0.0),
        ("-0o2", 0.0),
        ("0o+2", 0.0),
        ("0o-2", 0.0),
        // Binary
        ("0b101101", 45.0),
        ("   0b101101", 45.0),
        ("0b101101   ", 45.0),
        ("   0b101101   ", 45.0),
        ("0b0101101", 45.0),
        ("0B1110100110", 934.0),
        ("0b100112001", 0.0),
        ("0b10011001.1", 0.0),
        ("+0b1", 0.0),
        ("-0b1", 0.0),
        ("0b+1", 0.0),
        ("0b-1", 0.0),
    );

    assert_eq!(ScratchObject::Number(69.0).convert_to_number(), 69.0);
    assert_eq!(ScratchObject::Bool(true).convert_to_number(), 1.0);
    assert_eq!(ScratchObject::Bool(false).convert_to_number(), 0.0);

    assert!(
        ScratchObject::String(stat("Infinity"))
            .convert_to_number()
            .is_sign_positive()
    );
    assert!(
        ScratchObject::String(stat("Infinity"))
            .convert_to_number()
            .is_infinite()
    );

    assert!(
        ScratchObject::String(stat("-Infinity"))
            .convert_to_number()
            .is_sign_negative()
    );
    assert!(
        ScratchObject::String(stat("-Infinity"))
            .convert_to_number()
            .is_infinite()
    );
}

macro_rules! number_to_string {
    ($(($input:expr, $expected:expr)),* $(,)?) => {
        $(
            assert_eq!(
                ScratchObject::Number($input).convert_to_string(),
                $expected
            );
        )*
    };
}

#[test]
fn conversion_string() {
    number_to_string!(
        (0.0, "0"),
        (-0.0, "0"),
        (2.0, "2"),
        (-2.0, "-2"),
        (2.54, "2.54"),
        (-2.54, "-2.54"),
        (2550.625021000115, "2550.625021000115"),
        (-2550.625021000115, "-2550.625021000115"),
        (9.4324e+20, "943240000000000000000"),
        (-2.591e-2, "-0.02591"),
        (9.4324e+21, "9.4324e+21"),
        (-2.591e-13, "-2.591e-13"),
        (0.01, "0.01"),
        (f64::INFINITY, "Infinity"),
        (f64::NEG_INFINITY, "-Infinity"),
        (f64::NAN, "NaN")
    );

    assert_eq!(ScratchObject::Bool(true).convert_to_string(), "true");
    assert_eq!(ScratchObject::Bool(false).convert_to_string(), "false");
}
