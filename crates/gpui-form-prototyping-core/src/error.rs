use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrototypingError {
    InvalidIdentifier {
        kind: &'static str,
        value: String,
    },
    InvalidPath {
        kind: &'static str,
        value: String,
        error: String,
    },
    InvalidType {
        field_name: String,
        value: String,
        error: String,
    },
    InvalidExpression {
        field_name: String,
        value: String,
        error: String,
    },
}

impl fmt::Display for PrototypingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidIdentifier { kind, value } => {
                write!(f, "invalid {kind} `{value}` in prototyping metadata")
            },
            Self::InvalidPath { kind, value, error } => {
                write!(
                    f,
                    "invalid {kind} `{value}` in prototyping metadata: {error}"
                )
            },
            Self::InvalidType {
                field_name,
                value,
                error,
            } => {
                write!(
                    f,
                    "invalid value type `{value}` for field `{field_name}` in prototyping metadata: {error}"
                )
            },
            Self::InvalidExpression {
                field_name,
                value,
                error,
            } => {
                write!(
                    f,
                    "invalid default expression `{value}` for field `{field_name}` in prototyping metadata: {error}"
                )
            },
        }
    }
}

impl std::error::Error for PrototypingError {}

pub type PrototypingResult<T> = Result<T, PrototypingError>;
