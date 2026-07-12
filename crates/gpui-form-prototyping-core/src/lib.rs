//! Prototyping utilities that generate GPUI form scaffolds from shape metadata.

pub mod code_gen;
pub mod error;
pub mod implementations;
pub mod imports;

pub use code_gen::{FormLayout, FormParts, FormShapeAdapter};
pub use error::{PrototypingError, PrototypingResult};
