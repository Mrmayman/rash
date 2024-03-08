/// The enum variant data type used to represent dynamically typed objects in the interpreter.
/// # Conversion
/// * `to_string() -> String`
/// * `to_number() -> f64`
/// * `to_bool() -> bool`
pub enum DataValue {
    Number(f64),
    String(String),
    Bool(bool),
}

impl std::fmt::Debug for DataValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{}", n),
            Self::String(s) => write!(f, "\"{}\"", s),
            Self::Bool(b) => write!(f, "{}", b),
        }
    }
}

impl ToString for DataValue {
    fn to_string(&self) -> String {
        // If number is bigger than this then represent as exponentials.
        const POSITIVE_EXPONENTIAL_THRESHOLD: f64 = 1e21;
        // If number is smaller than this then represent as exponentials.
        const NEGATIVE_EXPONENTIAL_THRESHOLD: f64 = 2e-6;

        match self {
            DataValue::Number(num) => {
                if *num >= POSITIVE_EXPONENTIAL_THRESHOLD {
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
            DataValue::String(s) => s.to_owned(), // Faster than s.to_string()
            DataValue::Bool(b) => b.to_string(),
        }
    }
}

impl DataValue {
    /// Gets a number from a DataValue using implicit convertion.
    /// Supports `0x` hexadecimal and `0b` binary literal strings.
    /// # Examples
    /// * `Number(2.0) -> 2.0`
    /// * `String("5") -> 5.0`
    /// * `String("0x10") -> 16.0`
    /// * `String("0b10") -> 2.0`
    /// * `String("something") -> 0.0` (a default value if not valid)
    /// * `Bool(true) -> 1.0`
    pub fn to_number(&self) -> f64 {
        match self {
            DataValue::Number(number) => *number,
            DataValue::String(string) => string.parse().unwrap_or({
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
            DataValue::Bool(boolean) => *boolean as i32 as f64,
        }
    }

    /// Gets a bool from a DataValue using implicit convertion.
    /// # Rules
    /// * All non zero and NaN numbers are truthy.
    /// * All strings except for "false" and "0" are truthy.
    /// # Examples
    /// * `Number(1.0) -> true`
    /// * `Number(NaN) -> false`
    /// * `Number(0.0) -> false`
    /// * `Number(-0.0) -> true`
    /// * `String("true") -> true`
    /// * `String("something") -> true`
    /// * `String("false") -> false`
    /// * `String("0") -> false`
    /// * `String("0.0") -> true`
    /// * `Bool(true) -> true`
    /// * `Bool(false) -> false`
    pub fn to_bool(&self) -> bool {
        match self {
            DataValue::Number(n) => *n != 0.0 && !n.is_nan(),
            DataValue::String(s) => s != "0" && s != "false",
            DataValue::Bool(b) => *b,
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
        assert_eq!(DataValue::String("true".to_owned()).to_bool(), true);
        assert_eq!(DataValue::String("false".to_owned()).to_bool(), false);

        assert_eq!(DataValue::String("1".to_owned()).to_bool(), true);
        assert_eq!(DataValue::String("0".to_owned()).to_bool(), false);

        assert_eq!(DataValue::String("1.0".to_owned()).to_bool(), true);
        assert_eq!(DataValue::String("0.0".to_owned()).to_bool(), true);
        assert_eq!(DataValue::String("-1".to_owned()).to_bool(), true);
        assert_eq!(DataValue::String("-1.0".to_owned()).to_bool(), true);
        assert_eq!(DataValue::String("0e10".to_owned()).to_bool(), true);
    }

    #[test]
    fn conversion_number() {
        assert_eq!(DataValue::Number(69.0).to_number(), 69.0);
        assert_eq!(DataValue::String("69.0".to_owned()).to_number(), 69.0);

        assert_eq!(DataValue::String("1e3".to_owned()).to_number(), 1000.0);
        assert_eq!(DataValue::String("1.2e3".to_owned()).to_number(), 1200.0);

        assert_eq!(DataValue::String("0x10".to_owned()).to_number(), 16.0);
        assert_eq!(DataValue::String("0b10".to_owned()).to_number(), 2.0);
    }

    #[test]
    fn conversion_string() {
        assert_eq!(DataValue::Number(6.9).to_string(), "6.9");
        assert_eq!(DataValue::Number(2e22).to_string(), "2e+22");
        assert_eq!(DataValue::Number(2e-22).to_string(), "2e-22");

        assert_eq!(DataValue::Bool(true).to_string(), "true");
        assert_eq!(DataValue::Bool(false).to_string(), "false");
    }
}
