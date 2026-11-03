use super::InfiniteSelectValue;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};
use std::str::FromStr;
use std::{error::Error, fmt};

/// Represents an index-based selection path through nested infinite-select enums.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InfiniteSelectPath {
    indices: Vec<usize>,
}

impl InfiniteSelectPath {
    /// Creates a new empty selection path.
    pub fn new() -> Self {
        Self {
            indices: Vec::new(),
        }
    }

    /// Creates a path with the given indices.
    pub fn with_indices(indices: Vec<usize>) -> Self {
        Self { indices }
    }

    /// Returns the selection index at a given depth.
    pub fn get(&self, depth: usize) -> Option<usize> {
        self.indices.get(depth).copied()
    }

    /// Sets the selection at a given depth, truncating deeper selections.
    pub fn set(&mut self, depth: usize, index: usize) {
        self.indices.truncate(depth);
        self.indices.push(index);
    }

    /// Clears selections from a given depth onwards.
    pub fn clear_from(&mut self, depth: usize) {
        self.indices.truncate(depth);
    }

    /// Truncates the path to the given length.
    pub fn truncate(&mut self, len: usize) {
        self.indices.truncate(len);
    }

    /// Returns the current depth of the selection.
    pub fn len(&self) -> usize {
        self.indices.len()
    }

    /// Returns all indices as a slice.
    pub fn indices(&self) -> &[usize] {
        &self.indices
    }

    /// Returns true if the path is empty.
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }
}

/// Represents a key-based selection path through nested infinite-select enums.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InfiniteSelectKeyPath {
    keys: Vec<String>,
}

impl InfiniteSelectKeyPath {
    /// Creates a new empty key path.
    pub fn new() -> Self {
        Self { keys: Vec::new() }
    }

    /// Creates a key path with the given keys.
    pub fn with_keys(keys: Vec<String>) -> Self {
        Self { keys }
    }

    /// Returns the selected key at a given depth.
    pub fn get(&self, depth: usize) -> Option<&str> {
        self.keys.get(depth).map(String::as_str)
    }

    /// Sets the selected key at a given depth, truncating deeper selections.
    pub fn set(&mut self, depth: usize, key: impl Into<String>) {
        self.keys.truncate(depth);
        self.keys.push(key.into());
    }

    /// Clears selections from a given depth onwards.
    pub fn clear_from(&mut self, depth: usize) {
        self.keys.truncate(depth);
    }

    /// Truncates the path to the given length.
    pub fn truncate(&mut self, len: usize) {
        self.keys.truncate(len);
    }

    /// Returns the current depth of the selection.
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Returns all keys as a slice.
    pub fn keys(&self) -> &[String] {
        &self.keys
    }

    /// Returns true if the path is empty.
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }
}

impl fmt::Display for InfiniteSelectKeyPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, key) in self.keys.iter().enumerate() {
            if index > 0 {
                write!(f, "/")?;
            }

            for ch in key.chars() {
                match ch {
                    '\\' => write!(f, "\\\\")?,
                    '/' => write!(f, "\\/")?,
                    _ => write!(f, "{ch}")?,
                }
            }
        }

        Ok(())
    }
}

/// A parse failure for the string form of `InfiniteSelectKeyPath`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InfiniteSelectKeyPathParseError {
    input: String,
    reason: InfiniteSelectKeyPathParseErrorReason,
}

impl InfiniteSelectKeyPathParseError {
    fn dangling_escape(input: &str) -> Self {
        Self {
            input: input.to_string(),
            reason: InfiniteSelectKeyPathParseErrorReason::DanglingEscape,
        }
    }

    /// Returns the original string input that failed to parse.
    pub fn input(&self) -> &str {
        &self.input
    }

    /// Returns the typed parse failure reason.
    pub fn reason(&self) -> &InfiniteSelectKeyPathParseErrorReason {
        &self.reason
    }
}

impl fmt::Display for InfiniteSelectKeyPathParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.reason {
            InfiniteSelectKeyPathParseErrorReason::DanglingEscape => {
                write!(
                    f,
                    "infinite-select key path {:?} ends with an incomplete escape sequence",
                    self.input
                )
            },
        }
    }
}

impl Error for InfiniteSelectKeyPathParseError {}

/// The reason a string key path could not be parsed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InfiniteSelectKeyPathParseErrorReason {
    DanglingEscape,
}

impl FromStr for InfiniteSelectKeyPath {
    type Err = InfiniteSelectKeyPathParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.is_empty() {
            return Ok(Self::new());
        }

        let mut keys = Vec::new();
        let mut current = String::new();
        let mut escaped = false;

        for ch in value.chars() {
            if escaped {
                current.push(ch);
                escaped = false;
                continue;
            }

            match ch {
                '\\' => escaped = true,
                '/' => {
                    keys.push(std::mem::take(&mut current));
                },
                _ => current.push(ch),
            }
        }

        if escaped {
            return Err(InfiniteSelectKeyPathParseError::dangling_escape(value));
        }

        keys.push(current);
        Ok(Self::with_keys(keys))
    }
}

impl Serialize for InfiniteSelectKeyPath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for InfiniteSelectKeyPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(D::Error::custom)
    }
}

/// Returns the current selection path for a concrete infinite-select value.
pub fn path_from_value<T: InfiniteSelectValue>(value: &T) -> InfiniteSelectPath {
    value.selection_path()
}

/// Returns the current key path for a concrete infinite-select value.
pub fn key_path_from_value<T: InfiniteSelectValue>(value: &T) -> InfiniteSelectKeyPath {
    value.selection_key_path()
}

/// The failing segment of an index- or key-based infinite-select path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InfiniteSelectPathSegment {
    Index(usize),
    Key(String),
}

impl InfiniteSelectPathSegment {
    /// Returns the segment as an index when this is an index-based path error.
    pub fn as_index(&self) -> Option<usize> {
        match self {
            Self::Index(index) => Some(*index),
            Self::Key(_) => None,
        }
    }

    /// Returns the segment as a key when this is a key-based path error.
    pub fn as_key(&self) -> Option<&str> {
        match self {
            Self::Index(_) => None,
            Self::Key(key) => Some(key),
        }
    }
}

/// The reason an infinite-select path failed to resolve.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InfiniteSelectPathErrorReason {
    EmptyPath,
    MissingSelectionOptions,
    InvalidIndex { option_count: usize },
    UnknownKey { available_keys: Vec<String> },
}

/// A typed path-resolution failure for infinite-select helpers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InfiniteSelectPathError {
    depth: usize,
    segment: Option<InfiniteSelectPathSegment>,
    reason: InfiniteSelectPathErrorReason,
}

impl InfiniteSelectPathError {
    fn empty() -> Self {
        Self {
            depth: 0,
            segment: None,
            reason: InfiniteSelectPathErrorReason::EmptyPath,
        }
    }

    pub(super) fn missing_selection_options(
        depth: usize,
        segment: InfiniteSelectPathSegment,
    ) -> Self {
        Self {
            depth,
            segment: Some(segment),
            reason: InfiniteSelectPathErrorReason::MissingSelectionOptions,
        }
    }

    fn invalid_index(depth: usize, index: usize, option_count: usize) -> Self {
        Self {
            depth,
            segment: Some(InfiniteSelectPathSegment::Index(index)),
            reason: InfiniteSelectPathErrorReason::InvalidIndex { option_count },
        }
    }

    fn unknown_key(depth: usize, key: &str, available_keys: Vec<String>) -> Self {
        Self {
            depth,
            segment: Some(InfiniteSelectPathSegment::Key(key.to_string())),
            reason: InfiniteSelectPathErrorReason::UnknownKey { available_keys },
        }
    }

    /// Returns the depth where path resolution failed.
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Returns the failing path segment, when available.
    pub fn segment(&self) -> Option<&InfiniteSelectPathSegment> {
        self.segment.as_ref()
    }

    /// Returns the typed failure reason.
    pub fn reason(&self) -> &InfiniteSelectPathErrorReason {
        &self.reason
    }
}

impl fmt::Display for InfiniteSelectPathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.segment, &self.reason) {
            (None, InfiniteSelectPathErrorReason::EmptyPath) => {
                write!(f, "infinite-select path is empty")
            },
            (
                Some(InfiniteSelectPathSegment::Index(index)),
                InfiniteSelectPathErrorReason::MissingSelectionOptions,
            ) => {
                write!(
                    f,
                    "no selectable options exist at depth {} for index {}",
                    self.depth, index
                )
            },
            (
                Some(InfiniteSelectPathSegment::Key(key)),
                InfiniteSelectPathErrorReason::MissingSelectionOptions,
            ) => {
                write!(
                    f,
                    "no selectable options exist at depth {} for key {:?}",
                    self.depth, key
                )
            },
            (
                Some(InfiniteSelectPathSegment::Index(index)),
                InfiniteSelectPathErrorReason::InvalidIndex { option_count },
            ) => write!(
                f,
                "index {} is out of bounds at depth {} ({} options available)",
                index, self.depth, option_count
            ),
            (
                Some(InfiniteSelectPathSegment::Key(key)),
                InfiniteSelectPathErrorReason::UnknownKey { available_keys },
            ) => write!(
                f,
                "key {:?} is not valid at depth {} (available keys: {:?})",
                key, self.depth, available_keys
            ),
            _ => write!(f, "invalid infinite-select path at depth {}", self.depth),
        }
    }
}

impl Error for InfiniteSelectPathError {}

/// Rebuilds a value from an index-based selection path.
pub fn build_from_path<T: InfiniteSelectValue>(
    path: &InfiniteSelectPath,
) -> Result<T, InfiniteSelectPathError> {
    if path.is_empty() {
        return Err(InfiniteSelectPathError::empty());
    }

    let variants = T::variants();
    let root_index = path
        .get(0)
        .expect("non-empty paths include the root selection");
    let Some(mut current_value) = variants.get(root_index).cloned() else {
        return Err(InfiniteSelectPathError::invalid_index(
            0,
            root_index,
            variants.len(),
        ));
    };

    for depth in 1..path.len() {
        let index = path
            .get(depth)
            .expect("path length guarantees a selection at each iterated depth");
        let values = child_values_for_level(&current_value, depth - 1);

        if values.is_empty() {
            return Err(InfiniteSelectPathError::missing_selection_options(
                depth,
                InfiniteSelectPathSegment::Index(index),
            ));
        }

        let Some(value) = values.get(index) else {
            return Err(InfiniteSelectPathError::invalid_index(
                depth,
                index,
                values.len(),
            ));
        };

        current_value = value.clone();
    }

    Ok(current_value)
}

/// Rebuilds a value from a key-based selection path.
pub fn build_from_key_path<T: InfiniteSelectValue>(
    path: &InfiniteSelectKeyPath,
) -> Result<T, InfiniteSelectPathError> {
    if path.is_empty() {
        return Err(InfiniteSelectPathError::empty());
    }

    let root_key = path
        .get(0)
        .expect("non-empty key paths include the root selection");
    let variants = T::variants();
    let Some(mut current_value) = variants
        .iter()
        .find(|variant| variant.variant_key() == root_key)
        .cloned()
    else {
        return Err(InfiniteSelectPathError::unknown_key(
            0,
            root_key,
            variants
                .iter()
                .map(|variant| variant.variant_key().to_string())
                .collect(),
        ));
    };

    for depth in 1..path.len() {
        let key = path
            .get(depth)
            .expect("path length guarantees a selection at each iterated depth");
        let values = child_values_for_level(&current_value, depth - 1);

        if values.is_empty() {
            return Err(InfiniteSelectPathError::missing_selection_options(
                depth,
                InfiniteSelectPathSegment::Key(key.to_string()),
            ));
        }

        let available_keys: Vec<String> = values
            .iter()
            .filter_map(|value| value.selection_key_path().get(depth).map(str::to_string))
            .collect();

        let Some(value) = values.iter().find(|value| {
            value
                .selection_key_path()
                .get(depth)
                .is_some_and(|candidate| candidate == key)
        }) else {
            return Err(InfiniteSelectPathError::unknown_key(
                depth,
                key,
                available_keys,
            ));
        };

        current_value = value.clone();
    }

    Ok(current_value)
}

fn child_values_for_level<T: InfiniteSelectValue>(current_value: &T, level: usize) -> Vec<T> {
    let option_count = if level == 0 {
        current_value.child_variant_keys().len()
    } else {
        current_value.inner_child_variant_keys().len()
    };

    (0..option_count)
        .filter_map(|index| {
            if level == 0 {
                current_value.set_child_by_index(index)
            } else {
                current_value.inner_set_child_by_index(index)
            }
        })
        .collect()
}
