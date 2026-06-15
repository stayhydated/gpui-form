#[derive(Clone, Debug, Eq, thiserror::Error, PartialEq)]
pub enum PrototypingError {
    #[error("invalid {kind} `{value}` in prototyping metadata")]
    InvalidIdentifier { kind: &'static str, value: String },
    #[error("invalid {kind} `{value}` in prototyping metadata: {error}")]
    InvalidPath {
        kind: &'static str,
        value: String,
        error: String,
    },
    #[error("invalid {kind} `{value}` for field `{field_name}` in prototyping metadata: {error}")]
    InvalidFieldPath {
        field_name: String,
        kind: &'static str,
        value: String,
        error: String,
    },
    #[error(
        "invalid value type `{value}` for field `{field_name}` in prototyping metadata: {error}"
    )]
    InvalidType {
        field_name: String,
        value: String,
        error: String,
    },
    #[error(
        "invalid default expression `{value}` for field `{field_name}` in prototyping metadata: {error}"
    )]
    InvalidExpression {
        field_name: String,
        value: String,
        error: String,
    },
    #[error(
        "missing {capability} for field `{field_name}` on form `{struct_name}` in prototyping metadata"
    )]
    MissingComponentCapability {
        struct_name: String,
        field_name: String,
        capability: &'static str,
    },
}

pub type PrototypingResult<T> = Result<T, PrototypingError>;

#[cfg(test)]
mod tests {
    use super::PrototypingError;

    #[test]
    fn prototyping_error_messages_remain_stable() {
        let error = PrototypingError::MissingComponentCapability {
            struct_name: "Demo".to_string(),
            field_name: "country".to_string(),
            capability: "render component",
        };

        assert_eq!(
            error.to_string(),
            "missing render component for field `country` on form `Demo` in prototyping metadata"
        );
    }
}
