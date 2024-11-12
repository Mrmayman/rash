//! # Scratch Data Types
//! This module contains the data types used in the interpreter.
//! The data types are:
//! - Number: A 64 bit floating point number.
//! - String: A UTF-8 string.
//! - Bool: A boolean value.
//!
//! This module aims to accurately mirror the behaviour of
//! the Scratch programming language, both in terms of the
//! types themselves and the conversion behaviour between them.
//!
//! Side note: This module is probably the best documented part
//! of the entire project lol.

use colored::Colorize;

use crate::compiler::VarType;

/// The enum variant data type used to represent dynamically typed
/// objects in the interpreter.
///
/// There are a few methods to convert between the different types,
/// that accurately mirror the behaviour of the Scratch programming language.
#[repr(C)]
pub enum ScratchObject {
    Number(f64),
    String(String),
    Bool(bool),
}

// Debugging code for checking if the objects are being dropped
// Lets you know when the object is being dropped by the JIT compiled code
// impl Drop for ScratchObject {
//     fn drop(&mut self) {
//         println!("Dropping self: {self}");
//     }
// }

pub const ID_NUMBER: i64 = 0;
pub const ID_STRING: i64 = 1;
pub const ID_BOOL: i64 = 2;

// I know #[derive(Clone)] does the same thing.
// But this made it faster by 20 milliseconds
impl Clone for ScratchObject {
    fn clone(&self) -> Self {
        match *self {
            Self::Number(arg0) => Self::Number(arg0),
            Self::String(ref arg0) => Self::String(arg0.to_owned()),
            Self::Bool(arg0) => Self::Bool(arg0),
        }
    }
}

impl std::fmt::Debug for ScratchObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(n) => write!(
                f,
                "{}{}{}",
                "(".green(),
                n.to_string().bright_green().bold(),
                ")".green()
            ),
            Self::String(s) => write!(
                f,
                "{}{}{}",
                "\"".yellow(),
                s.bright_yellow().bold(),
                "\"".yellow()
            ),
            Self::Bool(b) => write!(f, "{}", b.to_string().cyan()),
        }
    }
}

impl ScratchObject {
    /// Gets the data type of the `ScratchObject`.
    #[allow(unused)]
    pub fn get_type(&self) -> VarType {
        match self {
            ScratchObject::Number(_) => VarType::Number,
            ScratchObject::String(_) => VarType::String,
            ScratchObject::Bool(_) => VarType::Bool,
        }
    }

    /// Gets a number from a `ScratchObject` using implicit convertion.
    /// Supports `0x` hexadecimal and `0b` binary literal strings.
    /// # Examples
    /// ```
    /// assert_eq!(ScratchObject::Number(2.0).convert_to_number(), 2.0);
    /// assert_eq!(ScratchObject::String("5".to_owned()).convert_to_number(), 5.0);
    /// assert_eq!(ScratchObject::String("0x10".to_owned()).convert_to_number(), 16.0);
    /// assert_eq!(ScratchObject::String("0b10".to_owned()).convert_to_number(), 2.0);
    /// assert_eq!(ScratchObject::String("something".to_owned()).convert_to_number(), 0.0);
    /// assert_eq!(ScratchObject::Bool(true).convert_to_number(), 1.0);
    /// ```
    pub fn convert_to_number(&self) -> f64 {
        match self {
            ScratchObject::Number(number) => *number,
            ScratchObject::String(string) => {
                let s = string.parse().unwrap_or({
                    // Couldn't parse the string normally, so it must be typed strangely.
                    // Checking some edge cases.

                    let lowercase = string.to_lowercase();
                    let string = lowercase.trim();

                    if string.starts_with("0x") {
                        convert_base_literal(string, 16)
                    } else if string.starts_with("0b") {
                        convert_base_literal(string, 2)
                    } else if string.starts_with("0o") {
                        convert_base_literal(string, 8)
                    } else {
                        Default::default()
                    }
                });
                if s.is_nan() {
                    0.0
                } else {
                    s
                }
            }
            ScratchObject::Bool(boolean) => {
                if *boolean {
                    1.0
                } else {
                    0.0
                }
            }
        }
    }

    pub fn convert_to_number_with_decimal_check(&self) -> (f64, bool) {
        let decimal = match self {
            ScratchObject::Number(n) => n.fract() != 0.0,
            ScratchObject::String(s) => s.contains('.'),
            ScratchObject::Bool(_) => false,
        };
        (self.convert_to_number(), decimal)
    }

    /// Gets a bool from a `ScratchObject` using implicit convertion.
    /// # Rules
    /// * All non zero and NaN numbers are truthy.
    /// * All strings except for "false" and "0" are truthy.
    /// # Examples
    /// ```
    /// assert_eq!(ScratchObject::Number(1.0).convert_to_bool(), true);
    /// assert_eq!(ScratchObject::Number(std::f64::NAN).convert_to_bool(), false);
    /// assert_eq!(ScratchObject::Number(0.0).convert_to_bool(), false);
    /// assert_eq!(ScratchObject::Number(-0.0).convert_to_bool(), true);
    /// assert_eq!(ScratchObject::String("true".to_owned()).convert_to_bool(), true);
    /// assert_eq!(ScratchObject::String("something".to_owned()).convert_to_bool(), true);
    /// assert_eq!(ScratchObject::String("false".to_owned()).convert_to_bool(), false);
    /// assert_eq!(ScratchObject::String("0".to_owned()).convert_to_bool(), false);
    /// assert_eq!(ScratchObject::String("0.0".to_owned()).convert_to_bool(), true);
    /// assert_eq!(ScratchObject::String("".to_owned()).convert_to_bool(), false);
    /// assert_eq!(ScratchObject::Bool(true).convert_to_bool(), true);
    /// assert_eq!(ScratchObject::Bool(false).convert_to_bool(), false);
    /// ```
    pub fn convert_to_bool(&self) -> bool {
        match self {
            ScratchObject::Number(n) => *n != 0.0 && !n.is_nan(),
            ScratchObject::String(s) => s != "0" && s.to_lowercase() != "false" && !s.is_empty(),
            ScratchObject::Bool(b) => *b,
        }
    }

    /// Converts a `ScratchObject` to a string.
    ///
    /// Not to be confused with [`ScratchObject::to_string`]
    /// as that is for pretty-printing whereas this is for conversion
    /// in the actual interpreter.
    ///
    /// # Examples
    /// ```
    /// assert_eq!(ScratchObject::Number(0.0).convert_to_string(), "0");
    /// assert_eq!(ScratchObject::Number(6.9).convert_to_string(), "6.9");
    /// assert_eq!(ScratchObject::Number(2e22).convert_to_string(), "2e+22");
    /// assert_eq!(ScratchObject::Number(2e-22).convert_to_string(), "2e-22");
    /// assert_eq!(ScratchObject::Bool(true).convert_to_string(), "true");
    /// assert_eq!(ScratchObject::Bool(false).convert_to_string(), "false");
    /// ```
    pub fn convert_to_string(&self) -> String {
        // If number is bigger than this then represent as exponentials.
        const POSITIVE_EXPONENTIAL_THRESHOLD: f64 = 1e21;
        // If number is smaller than this then represent as exponentials.
        const NEGATIVE_EXPONENTIAL_THRESHOLD: f64 = 2e-6;

        match self {
            ScratchObject::Number(num) => {
                if *num == 0.0 {
                    "0".to_owned()
                } else if num.is_infinite() {
                    if num.is_sign_positive() {
                        "Infinity".to_owned()
                    } else {
                        "-Infinity".to_owned()
                    }
                } else if num.abs() >= POSITIVE_EXPONENTIAL_THRESHOLD {
                    // Number so big it is exponential
                    // Eg: 1000000000000000000000 is 1e+21
                    let formatted = format!("{num:e}");
                    if formatted.contains("e-") {
                        formatted
                    } else {
                        // Rust formats it as 1e21, ignoring the plus
                        // So we must add it ourselves to match Scratch
                        formatted.replace('e', "e+")
                    }
                } else if num.abs() < NEGATIVE_EXPONENTIAL_THRESHOLD {
                    // Number so small it is exponential
                    // Eg: 0.0000001 is 1e-7
                    format!("{num:e}")
                } else {
                    num.to_string()
                }
            }
            ScratchObject::String(s) => s.to_owned(), // Faster than s.to_string()
            ScratchObject::Bool(true) => "true".to_owned(),
            ScratchObject::Bool(false) => "false".to_owned(),
        }
    }
}

/// Takes in string such as "0x10" or "0b10" and converts it to number.
///
/// Converts to a number based on the base. Hexadecimal is base 16, binary is base 2
fn convert_base_literal(string: &str, base: u32) -> f64 {
    let hex_number = string.get(2..).unwrap_or_default(); // Cuts off the "0x" or "0b"
    if hex_number.starts_with('+') || hex_number.starts_with('-') {
        return 0.0;
    }
    f64::from(u32::from_str_radix(hex_number, base).unwrap_or_default())
}

/// Tests for checking the conversion between values of different types.
///
/// Based on
/// https://github.com/scratchcpp/libscratchcpp/blob/5e1e3b62ae2e5198da2ca8f7d32890abbdf75b91/test/scratch_classes/value_test.cpp
///
/// Massive credit to adazem009
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversion_bool() {
        assert_eq!(
            ScratchObject::String("true".to_owned()).convert_to_bool(),
            true
        );
        assert_eq!(
            ScratchObject::String("false".to_owned()).convert_to_bool(),
            false
        );
        assert_eq!(
            ScratchObject::String("False".to_owned()).convert_to_bool(),
            false
        );

        assert_eq!(
            ScratchObject::String("1".to_owned()).convert_to_bool(),
            true
        );
        assert_eq!(
            ScratchObject::String("0".to_owned()).convert_to_bool(),
            false
        );

        assert_eq!(
            ScratchObject::String("1.0".to_owned()).convert_to_bool(),
            true
        );
        assert_eq!(
            ScratchObject::String("0.0".to_owned()).convert_to_bool(),
            true
        );
        assert_eq!(
            ScratchObject::String("-1".to_owned()).convert_to_bool(),
            true
        );
        assert_eq!(
            ScratchObject::String("-1.0".to_owned()).convert_to_bool(),
            true
        );
        assert_eq!(
            ScratchObject::String("0e10".to_owned()).convert_to_bool(),
            true
        );
        assert_eq!(
            ScratchObject::String(String::new()).convert_to_bool(),
            false
        );

        assert_eq!(ScratchObject::Number(1.0).convert_to_bool(), true);
        assert_eq!(ScratchObject::Number(0.0).convert_to_bool(), false);
        assert_eq!(ScratchObject::Number(f64::NAN).convert_to_bool(), false);
    }

    macro_rules! string_to_number {
        ($(($input_str:expr, $expected:expr)),* $(,)?) => {
            $(
                assert_eq!(
                    ScratchObject::String($input_str.to_owned()).convert_to_number(),
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

        assert!(ScratchObject::String("Infinity".to_owned())
            .convert_to_number()
            .is_sign_positive());
        assert!(ScratchObject::String("Infinity".to_owned())
            .convert_to_number()
            .is_infinite());

        assert!(ScratchObject::String("-Infinity".to_owned())
            .convert_to_number()
            .is_sign_negative());
        assert!(ScratchObject::String("-Infinity".to_owned())
            .convert_to_number()
            .is_infinite());
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
}
