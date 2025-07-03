use regex::Regex;
use std::sync::OnceLock;

pub trait NumRegex {
    fn validation_regex() -> &'static Regex;
}

macro_rules! static_regex {
    ($pattern:expr) => {{
        static REGEX: OnceLock<Regex> = OnceLock::new();
        REGEX.get_or_init(|| Regex::new($pattern).expect("Invalid regex pattern"))
    }};
}

macro_rules! impl_numregex_signed {
    ($($t:ty),*) => {
        $(
            impl NumRegex for $t {
                fn validation_regex() -> &'static Regex {
                    static_regex!(r"^[+-]?(?:0|[1-9]\d*)$")
                }
            }
        )*
    };
}

macro_rules! impl_numregex_unsigned {
    ($($t:ty),*) => {
        $(
            impl NumRegex for $t {
                fn validation_regex() -> &'static Regex {
                    static_regex!(r"^(?:0|[1-9]\d*)$")
                }
            }
        )*
    };
}

macro_rules! impl_numregex_float {
    ($($t:ty),*) => {
        $(
            impl NumRegex for $t {
                fn validation_regex() -> &'static Regex {
                    // Matches: integers, decimals, scientific notation, infinity, NaN
                    static_regex!(r"^[+-]?(?:(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?|inf|infinity|nan)$")
                }
            }
        )*
    };
}

impl_numregex_signed!(i8, i16, i32, i64, i128, isize);
impl_numregex_unsigned!(u8, u16, u32, u64, u128, usize);
impl_numregex_float!(f32, f64);

#[cfg(feature = "rust_decimal")]
impl NumRegex for rust_decimal::Decimal {
    fn validation_regex() -> &'static Regex {
        // Matches decimal numbers with optional sign and decimal point
        // Supports scientific notation as rust_decimal can parse it
        static_regex!(r"^[+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?$")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signed_integers() {
        assert!(i32::validation_regex().is_match("123"));
        assert!(i32::validation_regex().is_match("-123"));
        assert!(i32::validation_regex().is_match("+123"));
        assert!(i32::validation_regex().is_match("0"));
        assert!(!i32::validation_regex().is_match("123.45"));
        assert!(!i32::validation_regex().is_match("abc"));
        assert!(!i32::validation_regex().is_match(""));
    }

    #[test]
    fn test_unsigned_integers() {
        assert!(u32::validation_regex().is_match("123"));
        assert!(u32::validation_regex().is_match("0"));
        assert!(!u32::validation_regex().is_match("-123"));
        assert!(!u32::validation_regex().is_match("+123"));
        assert!(!u32::validation_regex().is_match("123.45"));
    }

    #[test]
    fn test_floats() {
        assert!(f64::validation_regex().is_match("123.45"));
        assert!(f64::validation_regex().is_match("-123.45"));
        assert!(f64::validation_regex().is_match("1.23e10"));
        assert!(f64::validation_regex().is_match("1.23E-10"));
        assert!(f64::validation_regex().is_match("inf"));
        assert!(f64::validation_regex().is_match("nan"));
        assert!(f64::validation_regex().is_match("123"));
        assert!(f64::validation_regex().is_match(".5"));
        assert!(!f64::validation_regex().is_match("123."));
    }

    #[cfg(feature = "rust_decimal")]
    #[test]
    fn test_decimal() {
        use rust_decimal::Decimal;

        assert!(Decimal::validation_regex().is_match("123.45"));
        assert!(Decimal::validation_regex().is_match("-123.45"));
        assert!(Decimal::validation_regex().is_match("1.23e10"));
        assert!(Decimal::validation_regex().is_match("0"));
        assert!(!Decimal::validation_regex().is_match("inf"));
        assert!(!Decimal::validation_regex().is_match("nan"));
    }

    #[test]
    fn test_regex_compilation() {
        let _ = i32::validation_regex();
        let _ = u32::validation_regex();
        let _ = f64::validation_regex();

        #[cfg(feature = "rust_decimal")]
        {
            use rust_decimal::Decimal;
            let _ = Decimal::validation_regex();
        }
    }
}
