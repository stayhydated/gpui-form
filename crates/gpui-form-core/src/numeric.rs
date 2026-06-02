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
    if value.is_empty() {
        return true;
    }

    let chars: Vec<char> = value.chars().collect();

    // Allow just "-" for intermediate input
    if chars.len() == 1 && chars[0] == '-' {
        return true;
    }

    // First character: must be 0-9 or '-'
    if !chars[0].is_ascii_digit() && chars[0] != '-' {
        return false;
    }

    // If there are more characters, validate from second character onwards
    if chars.len() > 1 {
        let start_idx = if chars[0] == '-' { 1 } else { 0 };

        // Check for invalid leading zeros: "0X" where X is a digit (not decimal point)
        // Allow: "0", "0.", "0.5", "-0", "-0.", "-0.5"
        // Reject: "00", "01", "-00", "-01"
        if chars[start_idx] == '0'
            && chars.len() > start_idx + 1
            && chars[start_idx + 1].is_ascii_digit()
        {
            return false;
        }

        // Second character onwards: can't be '-'
        if !chars[start_idx..]
            .iter()
            .all(|&c| c.is_ascii_digit() || c == '.')
        {
            return false;
        }
    }

    // If required, check if it can parse
    if require_parse {
        value.parse::<T>().is_ok()
    } else {
        true
    }
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
    if value.is_empty() {
        return true;
    }

    let chars: Vec<char> = value.chars().collect();

    // For unsigned types, first character must be 0-9
    if !chars[0].is_ascii_digit() {
        return false;
    }

    // Check for invalid leading zeros: "0X" where X is a digit
    // Allow: "0"
    // Reject: "00", "01", "001"
    if chars[0] == '0' && chars.len() > 1 && chars[1].is_ascii_digit() {
        return false;
    }

    // All characters must be digits
    if !chars.iter().all(|&c| c.is_ascii_digit()) {
        return false;
    }

    // If required, check if it can parse
    if require_parse {
        value.parse::<T>().is_ok()
    } else {
        true
    }
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
}
