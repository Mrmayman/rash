/// The enum variant data type used to represent dynamically typed objects in the interpreter.
/// # Conversion
/// * `to_string() -> String`
/// * `to_number() -> f64`
/// * `to_bool() -> bool`
pub enum ScratchObject {
    Number(f64),
    String(String),
    Bool(bool),
    Pointer(usize),
}

// I know #[derive(Clone)] does the same thing.
// But this made it faster by 20 milliseconds
impl Clone for ScratchObject {
    fn clone(&self) -> Self {
        match *self {
            Self::Number(arg0) => Self::Number(arg0),
            Self::String(ref arg0) => Self::String(arg0.to_owned()),
            Self::Bool(arg0) => Self::Bool(arg0),
            Self::Pointer(arg0) => Self::Pointer(arg0),
        }
    }
}

impl std::fmt::Debug for ScratchObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{}", n),
            Self::String(s) => write!(f, "\"{}\"", s),
            Self::Bool(b) => write!(f, "{}", b),
            Self::Pointer(p) => write!(f, "*{}", p),
        }
    }
}

impl ScratchObject {
    /// Gets a number from a DataValue using implicit convertion.
    /// Supports `0x` hexadecimal and `0b` binary literal strings.
    /// # Examples
    /// * `Number(2.0) -> 2.0`
    /// * `String("5") -> 5.0`
    /// * `String("0x10") -> 16.0`
    /// * `String("0b10") -> 2.0`
    /// * `String("something") -> 0.0` (a default value if not valid)
    /// * `Bool(true) -> 1.0`
    pub fn to_number(&self, memory: &[ScratchObject]) -> f64 {
        // Why two functions?
        // If I just use one function `to_number()` for everything,
        // then if a pointer is hit, it would call to_number()
        // on the data pointed by the pointer, being recursive.

        // The calling would be:
        // to_number(pointer) -> to_number(*pointer) -> actual value

        // However, we know there is only one level of nesting.
        // The calling WON'T be:
        // to_number(pointer) -> to_number(*pointer) -> to_number(*pointer) -> actual value

        // So, we can just use a different function for pointers.
        // This way, the calling will be:
        // to_number(pointer) -> to_number_unchecked(*pointer) -> actual value

        if let ScratchObject::Pointer(ptr) = self {
            memory[*ptr].to_number_unchecked()
        } else {
            self.to_number_unchecked()
        }
    }

    #[inline]
    fn to_number_unchecked(&self) -> f64 {
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
            ScratchObject::Pointer(_) => unreachable!(),
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
    pub fn to_bool(&self, memory: &[ScratchObject]) -> bool {
        // We don't use the above trick because it's slower here?
        match self {
            ScratchObject::Number(n) => *n != 0.0 && !n.is_nan(),
            ScratchObject::String(s) => s != "0" && s != "false",
            ScratchObject::Bool(b) => *b,
            ScratchObject::Pointer(p) => memory[*p].to_bool(memory),
        }
    }

    pub fn to_string(&self, memory: &[ScratchObject]) -> String {
        // If number is bigger than this then represent as exponentials.
        const POSITIVE_EXPONENTIAL_THRESHOLD: f64 = 1e21;
        // If number is smaller than this then represent as exponentials.
        const NEGATIVE_EXPONENTIAL_THRESHOLD: f64 = 2e-6;

        match self {
            ScratchObject::Number(num) => {
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
            ScratchObject::String(s) => s.to_owned(), // Faster than s.to_string()
            ScratchObject::Bool(b) => b.to_string(),
            ScratchObject::Pointer(p) => memory[*p].to_string(memory),
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
        let memory: [ScratchObject; 0] = [];

        assert_eq!(
            ScratchObject::String("true".to_owned()).to_bool(&memory),
            true
        );
        assert_eq!(
            ScratchObject::String("false".to_owned()).to_bool(&memory),
            false
        );

        assert_eq!(ScratchObject::String("1".to_owned()).to_bool(&memory), true);
        assert_eq!(
            ScratchObject::String("0".to_owned()).to_bool(&memory),
            false
        );

        assert_eq!(
            ScratchObject::String("1.0".to_owned()).to_bool(&memory),
            true
        );
        assert_eq!(
            ScratchObject::String("0.0".to_owned()).to_bool(&memory),
            true
        );
        assert_eq!(
            ScratchObject::String("-1".to_owned()).to_bool(&memory),
            true
        );
        assert_eq!(
            ScratchObject::String("-1.0".to_owned()).to_bool(&memory),
            true
        );
        assert_eq!(
            ScratchObject::String("0e10".to_owned()).to_bool(&memory),
            true
        );
    }

    #[test]
    fn conversion_number() {
        let memory: [ScratchObject; 0] = [];

        assert_eq!(ScratchObject::Number(69.0).to_number(&memory), 69.0);
        assert_eq!(
            ScratchObject::String("69.0".to_owned()).to_number(&memory),
            69.0
        );

        assert_eq!(
            ScratchObject::String("1e3".to_owned()).to_number(&memory),
            1000.0
        );
        assert_eq!(
            ScratchObject::String("1.2e3".to_owned()).to_number(&memory),
            1200.0
        );

        assert_eq!(
            ScratchObject::String("0x10".to_owned()).to_number(&memory),
            16.0
        );
        assert_eq!(
            ScratchObject::String("0b10".to_owned()).to_number(&memory),
            2.0
        );
    }

    #[test]
    fn conversion_string() {
        let memory: [ScratchObject; 0] = [];

        assert_eq!(ScratchObject::Number(6.9).to_string(&memory), "6.9");
        assert_eq!(ScratchObject::Number(2e22).to_string(&memory), "2e+22");
        assert_eq!(ScratchObject::Number(2e-22).to_string(&memory), "2e-22");

        assert_eq!(ScratchObject::Bool(true).to_string(&memory), "true");
        assert_eq!(ScratchObject::Bool(false).to_string(&memory), "false");
    }
}
