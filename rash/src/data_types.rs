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

pub const ID_NUMBER: usize = 0;
pub const ID_STRING: usize = 1;
pub const ID_BOOL: usize = 2;

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
    /// Gets a number from a ScratchObject using implicit convertion.
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
            ScratchObject::String(string) => string.parse().unwrap_or({
                // Couldn't parse the string normally, so it must be typed strangely.
                // Checking some edge cases.
                if string.starts_with("0x") {
                    convert_base_literal(string, 16)
                } else if string.starts_with("0b") {
                    convert_base_literal(string, 2)
                } else {
                    Default::default()
                }
            }),
            ScratchObject::Bool(boolean) => *boolean as i32 as f64,
        }
    }

    /// Gets a bool from a ScratchObject using implicit convertion.
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
    /// assert_eq!(ScratchObject::Bool(true).convert_to_bool(), true);
    /// assert_eq!(ScratchObject::Bool(false).convert_to_bool(), false);
    /// ```
    pub fn convert_to_bool(&self) -> bool {
        match self {
            ScratchObject::Number(n) => *n != 0.0 && !n.is_nan(),
            ScratchObject::String(s) => s != "0" && s != "false",
            ScratchObject::Bool(b) => *b,
        }
    }

    /// Converts a ScratchObject to a string.
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
                } else if num.abs() >= POSITIVE_EXPONENTIAL_THRESHOLD {
                    // Number so big it is exponential
                    // Eg: 1000000000000000000000 is 1e+21
                    let formatted = format!("{:e}", num);
                    if !formatted.contains("e-") {
                        // Rust formats it as 1e21, ignoring the plus
                        // So we must add it ourselves to match Scratch
                        formatted.replace('e', "e+")
                    } else {
                        formatted
                    }
                } else if num.abs() < NEGATIVE_EXPONENTIAL_THRESHOLD {
                    // Number so small it is exponential
                    // Eg: 0.0000001 is 1e-7
                    format!("{:e}", num)
                } else {
                    num.to_string()
                }
            }
            ScratchObject::String(s) => s.to_owned(), // Faster than s.to_string()
            ScratchObject::Bool(b) => b.to_string(),
        }
    }
}

/// Takes in string such as "0x10" or "0b10" and converts it to number.
///
/// Converts to a number based on the base. Hexadecimal is base 16, binary is base 2
fn convert_base_literal(string: &str, base: u32) -> f64 {
    let hex_number = string.get(2..).unwrap_or_default(); // Cuts off the "0x" or "0b"
    u32::from_str_radix(hex_number, base).unwrap_or_default() as f64
}

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
    }

    #[test]
    fn conversion_number() {
        assert_eq!(ScratchObject::Number(69.0).convert_to_number(), 69.0);
        assert_eq!(
            ScratchObject::String("69.0".to_owned()).convert_to_number(),
            69.0
        );

        assert_eq!(
            ScratchObject::String("1e3".to_owned()).convert_to_number(),
            1000.0
        );
        assert_eq!(
            ScratchObject::String("1.2e3".to_owned()).convert_to_number(),
            1200.0
        );

        assert_eq!(
            ScratchObject::String("0x10".to_owned()).convert_to_number(),
            16.0
        );
        assert_eq!(
            ScratchObject::String("0b10".to_owned()).convert_to_number(),
            2.0
        );
    }

    #[test]
    fn conversion_string() {
        assert_eq!(ScratchObject::Number(0.0).convert_to_string(), "0");
        assert_eq!(ScratchObject::Number(6.9).convert_to_string(), "6.9");
        assert_eq!(ScratchObject::Number(2e22).convert_to_string(), "2e+22");
        assert_eq!(ScratchObject::Number(2e-22).convert_to_string(), "2e-22");

        assert_eq!(ScratchObject::Bool(true).convert_to_string(), "true");
        assert_eq!(ScratchObject::Bool(false).convert_to_string(), "false");
    }
}
