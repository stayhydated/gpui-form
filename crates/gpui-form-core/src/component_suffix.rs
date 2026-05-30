/// Returns whether a component/prototyping suffix is a non-empty ASCII
/// identifier suffix.
pub const fn is_valid_ascii_identifier_suffix(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.is_empty() || (bytes.len() == 1 && bytes[0] == b'_') {
        return false;
    }
    if !is_ascii_ident_start(bytes[0]) {
        return false;
    }

    let mut idx = 1;
    while idx < bytes.len() {
        if !is_ascii_ident_continue(bytes[idx]) {
            return false;
        }
        idx += 1;
    }

    true
}

const fn is_ascii_ident_start(byte: u8) -> bool {
    byte == b'_' || byte >= b'a' && byte <= b'z' || byte >= b'A' && byte <= b'Z'
}

const fn is_ascii_ident_continue(byte: u8) -> bool {
    is_ascii_ident_start(byte) || byte >= b'0' && byte <= b'9'
}

/// Preferred generated component field/helper suffix.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ComponentSuffix(&'static str);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComponentSuffixError {
    value: String,
}

impl ComponentSuffixError {
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl std::fmt::Display for ComponentSuffixError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "`field_suffix` must be a non-empty ASCII identifier suffix, got `{}`",
            self.value
        )
    }
}

impl std::error::Error for ComponentSuffixError {}

pub const fn is_valid_component_suffix(value: &str) -> bool {
    is_valid_ascii_identifier_suffix(value)
}

pub fn validate_component_suffix(value: &str) -> Result<(), ComponentSuffixError> {
    if is_valid_component_suffix(value) {
        Ok(())
    } else {
        Err(ComponentSuffixError {
            value: value.to_string(),
        })
    }
}

impl ComponentSuffix {
    pub const fn new(value: &'static str) -> Self {
        assert!(
            is_valid_component_suffix(value),
            "component suffix must be a non-empty ASCII identifier suffix"
        );
        Self(value)
    }

    pub const fn new_opt(value: Option<&'static str>) -> Option<Self> {
        match value {
            Some(value) => Some(Self::new(value)),
            None => None,
        }
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::{is_valid_component_suffix, validate_component_suffix};

    #[test]
    fn component_suffix_validation_accepts_identifier_suffixes() {
        assert!(is_valid_component_suffix("input"));
        assert!(is_valid_component_suffix("number_input"));
        assert!(is_valid_component_suffix("_internal"));
    }

    #[test]
    fn component_suffix_validation_rejects_invalid_suffixes() {
        for value in ["", "_", "2input", "input-field", "input field"] {
            assert!(
                validate_component_suffix(value).is_err(),
                "`{value}` should be rejected as a component suffix"
            );
        }
    }
}
