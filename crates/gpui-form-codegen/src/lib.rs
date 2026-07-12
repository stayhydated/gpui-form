//! Shared code-generation helpers for `gpui-form` proc macro crates.

pub mod components;
pub mod crate_paths;
pub mod metadata;

mod names;

pub use crate_paths::{CratePaths, resolve_crate_path};
