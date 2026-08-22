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

use std::cmp::Ordering;

use colored::Colorize;
use smol_str::{SmolStr, StrExt, ToSmolStr, format_smolstr};

use crate::{compiler::VarType, stat};

#[cfg(test)]
mod tests;

/// The enum variant data type used to represent dynamically typed
/// objects in the interpreter.
///
/// There are a few methods to convert between the different types,
/// that accurately mirror the behaviour of the Scratch programming language.
#[repr(C)]
#[derive(PartialEq, Clone)]
pub enum ScratchObject {
    Number(f64),     // 0
    String(SmolStr), // 1
    Bool(bool),      // 2
}

// Debugging code for checking if the objects are being dropped
// Lets you know when the object is being dropped by the JIT compiled code
// impl Drop for ScratchObject {
//     fn drop(&mut self) {
//         let bytes = unsafe { std::mem::transmute::<&Self, &[i64; 4]>(&self) };
//         println!(
//             "Dropping self: {self:?} (bytes: {:X} {:X} {:X} {:X})",
//             bytes[0] as i32 as i64, bytes[1], bytes[2], bytes[3]
//         );
//         if let ScratchObject::String(s) = self {
//             let mut n = String::new();
//             std::mem::swap(&mut n, s);
//             std::mem::forget(n);
//         }
//     }
// }

pub const ID_NUMBER: i64 = 0;
pub const ID_STRING: i64 = 1;
pub const ID_BOOL: i64 = 2;

impl std::fmt::Debug for ScratchObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{}", n.to_string().bright_green().bold()),
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
    #[must_use]
    pub fn get_type(&self) -> VarType {
        match self {
            ScratchObject::Number(_) => VarType::Number,
            ScratchObject::String(_) => VarType::String,
            ScratchObject::Bool(_) => VarType::Bool,
        }
    }

    /// Gets a number from a `ScratchObject` using implicit convertion.
    /// Supports `0x` hexadecimal and `0b` binary literal strings.
    ///
    /// # Examples
    /// ```
    /// # use rash_vm::ScratchObject;
    /// assert_eq!(ScratchObject::Number(2.0).convert_to_number(), 2.0);
    /// assert_eq!(ScratchObject::String("5".into()).convert_to_number(), 5.0);
    /// assert_eq!(ScratchObject::String("0x10".into()).convert_to_number(), 16.0);
    /// assert_eq!(ScratchObject::String("0b10".into()).convert_to_number(), 2.0);
    /// assert_eq!(ScratchObject::String("something".into()).convert_to_number(), 0.0);
    /// assert_eq!(ScratchObject::Bool(true).convert_to_number(), 1.0);
    /// ```
    #[inline]
    #[must_use]
    pub fn convert_to_number(&self) -> f64 {
        match self {
            ScratchObject::Number(number) => *number,
            ScratchObject::String(string) => string_to_number(string),
            ScratchObject::Bool(boolean) => {
                if *boolean {
                    1.0
                } else {
                    0.0
                }
            }
        }
    }

    #[inline]
    #[must_use]
    pub fn convert_to_number_with_decimal_check(&self) -> (f64, bool) {
        let decimal = match self {
            ScratchObject::Number(n) => n.fract() != 0.0,
            ScratchObject::String(s) => s.contains('.'),
            ScratchObject::Bool(_) => true,
        };
        (self.convert_to_number(), decimal)
    }

    /// Gets a bool from a `ScratchObject` using implicit convertion.
    ///
    /// # Rules
    /// - All non zero and NaN numbers are truthy.
    /// - All strings except for "false" and "0" are truthy.
    ///
    /// # Examples
    /// ```
    /// # use rash_vm::ScratchObject;
    /// assert_eq!(ScratchObject::Number(1.0).convert_to_bool(), true);
    /// assert_eq!(
    ///     ScratchObject::Number(std::f64::NAN).convert_to_bool(),
    ///     false
    /// );
    /// assert_eq!(ScratchObject::Number(0.0).convert_to_bool(), false);
    /// assert_eq!(ScratchObject::Number(-0.0).convert_to_bool(), true);
    /// assert_eq!(
    ///     ScratchObject::String("true".into()).convert_to_bool(),
    ///     true
    /// );
    /// assert_eq!(ScratchObject::String("something".into()).convert_to_bool(), true);
    /// assert_eq!(ScratchObject::String("false".into()).convert_to_bool(), false);
    /// assert_eq!(ScratchObject::String("0".into()).convert_to_bool(), false);
    /// assert_eq!(ScratchObject::String("0.0".into()).convert_to_bool(), true);
    /// assert_eq!(ScratchObject::String("".into()).convert_to_bool(), false);
    /// assert_eq!(ScratchObject::Bool(true).convert_to_bool(), true);
    /// assert_eq!(ScratchObject::Bool(false).convert_to_bool(), false);
    /// ```
    #[inline]
    #[must_use]
    pub fn convert_to_bool(&self) -> bool {
        match self {
            ScratchObject::Number(n) => !(n.is_nan() || (*n == 0.0 && n.is_sign_positive())),
            ScratchObject::String(s) => s != "0" && s.to_lowercase() != "false" && !s.is_empty(),
            ScratchObject::Bool(b) => *b,
        }
    }

    /// Converts a `ScratchObject` to a string.
    ///
    /// Not to be confused with any `Display` implementations
    /// as that is for pretty-printing whereas this is for conversion
    /// in the actual interpreter.
    ///
    /// # Examples
    /// ```
    /// # use rash_vm::ScratchObject;
    /// assert_eq!(ScratchObject::Number(0.0).convert_to_string(), "0");
    /// assert_eq!(ScratchObject::Number(6.9).convert_to_string(), "6.9");
    /// assert_eq!(ScratchObject::Number(2e22).convert_to_string(), "2e+22");
    /// assert_eq!(ScratchObject::Number(2e-22).convert_to_string(), "2e-22");
    /// assert_eq!(ScratchObject::Bool(true).convert_to_string(), "true");
    /// assert_eq!(ScratchObject::Bool(false).convert_to_string(), "false");
    /// ```
    #[inline]
    #[must_use]
    pub fn convert_to_string(&self) -> SmolStr {
        match self {
            ScratchObject::Number(num) => number_to_string(*num),
            ScratchObject::String(s) => s.clone(),
            ScratchObject::Bool(true) => stat("true"),
            ScratchObject::Bool(false) => stat("false"),
        }
    }

    #[inline]
    #[must_use]
    pub fn scratch_cmp(&self, other: &ScratchObject) -> Ordering {
        #[inline]
        fn makebool(b: bool) -> f64 {
            if b { 1.0 } else { 0.0 }
        }

        #[inline]
        fn check_str(a: &ScratchObject, cmp_str: &mut bool, n: &mut f64) {
            match a {
                ScratchObject::Number(num) => *n = *num,
                ScratchObject::String(s) => {
                    if s.is_empty() || s.trim().is_empty() {
                        *cmp_str = true;
                    }
                    if let Ok(p) = s.parse::<f64>() {
                        if p.is_nan() {
                            *cmp_str = true;
                        } else {
                            *n = p;
                        }
                    } else {
                        *cmp_str = true;
                    }
                }
                ScratchObject::Bool(b) => *n = f64::from(u8::from(*b)),
            }
        }

        let a = self;
        let b = other;

        match (a, b) {
            // fast cases
            (ScratchObject::Number(a), ScratchObject::Number(b)) => return a.total_cmp(b),
            (ScratchObject::Number(a), ScratchObject::Bool(b)) => {
                return a.total_cmp(&makebool(*b));
            }
            (ScratchObject::Bool(b), ScratchObject::Number(a)) => {
                return makebool(*b).total_cmp(a);
            }
            (ScratchObject::Bool(a), ScratchObject::Bool(b)) => return a.cmp(b),
            _ => {} // We deal with strings below
        }

        let mut cmp_str = false;
        let mut n1 = 0.0f64;
        let mut n2 = 0.0f64;

        // even if both are strings, we still have
        // to try comparing them as numbers
        check_str(a, &mut cmp_str, &mut n1);
        if !cmp_str {
            check_str(b, &mut cmp_str, &mut n2);
        }

        if cmp_str {
            let s1 = a.convert_to_string();
            let s2 = b.convert_to_string();
            return s1.cmp(&s2);
        }

        n1.total_cmp(&n2)
    }
}

#[inline]
#[must_use]
pub fn number_to_string(num: f64) -> SmolStr {
    // If number is bigger than this then represent as exponentials.
    const POSITIVE_EXPONENTIAL_THRESHOLD: f64 = 1e21;
    // If number is smaller than this then represent as exponentials.
    const NEGATIVE_EXPONENTIAL_THRESHOLD: f64 = 2e-6;

    if num == 0.0 {
        stat("0")
    } else if num.is_infinite() {
        if num.is_sign_positive() {
            stat("Infinity")
        } else {
            stat("-Infinity")
        }
    } else if num.abs() >= POSITIVE_EXPONENTIAL_THRESHOLD {
        // Number so big it is exponential
        // Eg: 1000000000000000000000 is 1e+21
        let formatted = format_smolstr!("{num:e}");
        if formatted.contains("e-") {
            formatted
        } else {
            // Rust formats it as 1e21, ignoring the plus
            // So we must add it ourselves to match Scratch
            formatted.replace_smolstr("e", "e+")
        }
    } else if num.abs() < NEGATIVE_EXPONENTIAL_THRESHOLD {
        // Number so small it is exponential
        // Eg: 0.0000001 is 1e-7
        format_smolstr!("{num:e}")
    } else {
        num.to_smolstr()
    }
}

#[inline]
#[must_use]
pub fn string_to_number(string: &str) -> f64 {
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
    if s.is_nan() { 0.0 } else { s }
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
