/// Validates numeric input for signed types (i*, f*) with custom type support.
///
/// This validation allows:
/// - Empty strings
/// - Just "-" for intermediate input
/// - Valid signed numbers with optional decimal points
///
/// This validation rejects:
/// - Leading zeros before digits (e.g., "00", "01", "-00", "-01")
/// - Multiple "-" signs
/// - "-" anywhere except the first character
///
/// When `require_parse` is true, also validates that the value can be parsed
/// to the target type T.
pub fn validate_signed_numeric<T: std::str::FromStr>(value: &str, require_parse: bool) -> bool {
    if value.is_empty() || value == "-" {
        return true;
    }

    // First character: must be 0-9 or '-'
    let first = value.as_bytes()[0];
    if !first.is_ascii_digit() && first != b'-' {
        return false;
    }

    let digits = value.strip_prefix('-').unwrap_or(value).as_bytes();

    // A zero may precede a decimal point, but not another digit.
    if matches!(digits, [b'0', next, ..] if next.is_ascii_digit()) {
        return false;
    }

    // Numeric input is ASCII; byte iteration also rejects every non-ASCII character.
    if !digits
        .iter()
        .all(|byte| byte.is_ascii_digit() || *byte == b'.')
    {
        return false;
    }

    !require_parse || value.parse::<T>().is_ok()
}

/// Validates numeric input for unsigned types (u*) with custom type support.
///
/// This validation allows:
/// - Empty strings
/// - Valid unsigned numbers (digits only)
///
/// This validation rejects:
/// - Leading zeros before digits (e.g., "00", "01", "001")
/// - Any non-digit characters (including "-")
///
/// When `require_parse` is true, also validates that the value can be parsed
/// to the target type T.
pub fn validate_unsigned_numeric<T: std::str::FromStr>(value: &str, require_parse: bool) -> bool {
    let [first, rest @ ..] = value.as_bytes() else {
        return true;
    };

    // For unsigned types, first character must be 0-9
    if !first.is_ascii_digit() {
        return false;
    }

    // Check for invalid leading zeros: "0X" where X is a digit
    // Allow: "0"
    // Reject: "00", "01", "001"
    if *first == b'0' && rest.first().is_some_and(u8::is_ascii_digit) {
        return false;
    }

    // All characters must be digits
    if !rest.iter().all(u8::is_ascii_digit) {
        return false;
    }

    !require_parse || value.parse::<T>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_signed_numeric() {
        // Valid inputs
        assert!(validate_signed_numeric::<i32>("", true));
        assert!(validate_signed_numeric::<i32>("-", true));
        assert!(validate_signed_numeric::<i32>("0", true));
        assert!(validate_signed_numeric::<i32>("-0", true));
        assert!(validate_signed_numeric::<i32>("123", true));
        assert!(validate_signed_numeric::<i32>("-123", true));

        // Valid floats
        assert!(validate_signed_numeric::<f64>("0.5", true));
        assert!(validate_signed_numeric::<f64>("-0.5", true));
        assert!(validate_signed_numeric::<f64>("123.456", true));

        // Invalid: leading zeros
        assert!(!validate_signed_numeric::<i32>("00", true));
        assert!(!validate_signed_numeric::<i32>("01", true));
        assert!(!validate_signed_numeric::<i32>("-00", true));
        assert!(!validate_signed_numeric::<i32>("-01", true));

        // Invalid: multiple minus signs or in wrong position
        assert!(!validate_signed_numeric::<i32>("1-2", true));
        assert!(!validate_signed_numeric::<i32>("--1", true));
    }

    #[test]
    fn test_validate_unsigned_numeric() {
        // Valid inputs
        assert!(validate_unsigned_numeric::<u32>("", true));
        assert!(validate_unsigned_numeric::<u32>("0", true));
        assert!(validate_unsigned_numeric::<u32>("123", true));

        // Invalid: leading zeros
        assert!(!validate_unsigned_numeric::<u32>("00", true));
        assert!(!validate_unsigned_numeric::<u32>("01", true));
        assert!(!validate_unsigned_numeric::<u32>("001", true));

        // Invalid: minus sign
        assert!(!validate_unsigned_numeric::<u32>("-", true));
        assert!(!validate_unsigned_numeric::<u32>("-1", true));

        // Invalid: non-digits
        assert!(!validate_unsigned_numeric::<u32>("1.5", true));
        assert!(!validate_unsigned_numeric::<u32>("1a", true));
    }

    #[test]
    fn test_validate_without_parse() {
        // Custom types without parse check
        assert!(validate_signed_numeric::<i32>(
            "999999999999999999999",
            false
        ));
        assert!(validate_unsigned_numeric::<u32>(
            "999999999999999999999",
            false
        ));

        // Reject invalid patterns.
        assert!(!validate_signed_numeric::<i32>("00", false));
        assert!(!validate_unsigned_numeric::<u32>("00", false));
    }

    #[test]
    fn intermediate_signed_input_keeps_its_syntax_contract() {
        for value in ["-.", "1..2", "0.", "-0."] {
            assert!(validate_signed_numeric::<i32>(value, false), "{value:?}");
            assert!(!validate_signed_numeric::<i32>(value, true), "{value:?}");
        }

        for value in [".", "-00.1", "1-2", "--", "é", "１", "1é", "0１"] {
            for require_parse in [false, true] {
                assert!(
                    !validate_signed_numeric::<i32>(value, require_parse),
                    "{value:?}, require_parse: {require_parse}"
                );
                assert!(
                    !validate_unsigned_numeric::<u32>(value, require_parse),
                    "{value:?}, require_parse: {require_parse}"
                );
            }
        }
    }
}
