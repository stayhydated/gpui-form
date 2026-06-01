//! Runtime storage policy used by `#[derive(GpuiForm)]`.
//!
//! Component shape construction, rendering, value compatibility, and value
//! binding are provided by `component-shape` and `component-shape-gpui`.

pub use component_shape::{
    ComponentCapabilities, ComponentPrototyping, ComponentShapeMetadata, ComponentSuffix,
    RenderCapability, ValueBindingCapability, ValueChange,
    is_valid_component_suffix as is_valid_component_field_suffix,
};
pub use component_shape_gpui::{
    DeclaredGpuiComponentShape, GpuiComponentEventOf, GpuiComponentRender, GpuiComponentShape,
    GpuiComponentShapeFor, GpuiComponentStateOf, GpuiComponentStateValueBinding,
    GpuiComponentValueBinding, NoGpuiRenderComponent, seed_value_binding_state, value_change,
};

/// Form storage policy for a GPUI component shape.
///
/// Implement this for shapes declared with `component_shape_gpui::component_shape!`
/// when the shape should be consumable by `#[derive(GpuiForm)]`.
pub trait GpuiFormComponentShapePolicy {
    /// Shape-owned value-holder storage policy.
    type ValueStoragePolicy: ComponentValueStoragePolicy;
}

mod sealed {
    pub trait ValueStoragePolicy {}
}

/// Marker trait for a component shape's generated value-holder storage policy.
pub trait ComponentValueStoragePolicy: sealed::ValueStoragePolicy {
    /// Whether a missing generated value-holder value is invalid by default.
    const REQUIRES_VALUE: bool;
}

/// Store non-optional source fields as `Option<T>` and treat `None` as missing.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RequiredValueStorage;

impl sealed::ValueStoragePolicy for RequiredValueStorage {}

impl ComponentValueStoragePolicy for RequiredValueStorage {
    const REQUIRES_VALUE: bool = true;
}

/// Store non-optional source fields directly as `T`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DirectValueStorage;

impl sealed::ValueStoragePolicy for DirectValueStorage {}

impl ComponentValueStoragePolicy for DirectValueStorage {
    const REQUIRES_VALUE: bool = false;
}

/// Storage behavior used by generated form value holders.
///
/// This lets `#[derive(GpuiForm)]` defer the `T` vs `Option<T>` choice to the
/// component shape that owns the policy while still emitting concrete holder
/// conversion code.
pub trait ValueStorage<T>: ComponentValueStoragePolicy {
    /// Concrete generated value-holder field storage.
    type Storage;

    /// Construct storage from a present form value.
    fn present(value: T) -> Self::Storage;

    /// Construct storage from a present form value, allowing `RequiredValueStorage` to
    /// encode a value equal to the declared default as missing.
    fn present_unless_default(value: T, default: T) -> Self::Storage
    where
        T: PartialEq;

    /// Convert storage into an output value, using `missing` only for policies
    /// that can represent missing values.
    fn map_into_value<Output, Present, Missing>(
        storage: Self::Storage,
        present: Present,
        missing: Missing,
    ) -> Output
    where
        Present: FnOnce(T) -> Output,
        Missing: FnOnce() -> Output;

    /// Fallible variant of [`ValueStorage::map_into_value`].
    fn try_map_into_value<Output, Error, Present>(
        storage: Self::Storage,
        present: Present,
        missing: Error,
    ) -> Result<Output, Error>
    where
        Present: FnOnce(T) -> Output;

    /// Clone and map a present value, returning `None` only for policies that
    /// can represent missing values.
    fn map_present_cloned<Output, Present>(
        storage: &Self::Storage,
        present: Present,
    ) -> Option<Output>
    where
        T: Clone,
        Present: FnOnce(T) -> Output;

    /// Return whether storage contains a source value.
    ///
    /// Generated validation uses this to make shape-owned requiredness visible
    /// to `validate()` without hard-coding whether the policy stores `Option<T>`
    /// or `T`.
    fn is_present(storage: &Self::Storage) -> bool;
}

/// Storage policies that can synthesize missing/default value-holder storage.
#[diagnostic::on_unimplemented(
    message = "gpui-form cannot synthesize direct storage for `{T}` with this component shape policy",
    note = "add an intent-scoped `default = ...`, make `{T}` implement `Default`, or use `RequiredValueStorage` for the shape's form storage policy"
)]
pub trait DefaultValueStorage<T>: ValueStorage<T> {
    /// Construct a missing/default value-holder field.
    fn default_storage() -> Self::Storage;
}

/// Storage policies whose holder-to-model conversion cannot fail because the
/// storage representation always contains a value.
pub trait InfallibleValueStorage<T>: ValueStorage<T> {
    /// Convert storage into an output value without a missing-value fallback.
    fn map_into_value<Output, Present>(storage: Self::Storage, present: Present) -> Output
    where
        Present: FnOnce(T) -> Output;
}

impl<T> ValueStorage<T> for RequiredValueStorage {
    type Storage = Option<T>;

    fn present(value: T) -> Self::Storage {
        Some(value)
    }

    fn present_unless_default(value: T, default: T) -> Self::Storage
    where
        T: PartialEq,
    {
        if value == default { None } else { Some(value) }
    }

    fn map_into_value<Output, Present, Missing>(
        storage: Self::Storage,
        present: Present,
        missing: Missing,
    ) -> Output
    where
        Present: FnOnce(T) -> Output,
        Missing: FnOnce() -> Output,
    {
        match storage {
            Some(value) => present(value),
            None => missing(),
        }
    }

    fn try_map_into_value<Output, Error, Present>(
        storage: Self::Storage,
        present: Present,
        missing: Error,
    ) -> Result<Output, Error>
    where
        Present: FnOnce(T) -> Output,
    {
        storage.map(present).ok_or(missing)
    }

    fn map_present_cloned<Output, Present>(
        storage: &Self::Storage,
        present: Present,
    ) -> Option<Output>
    where
        T: Clone,
        Present: FnOnce(T) -> Output,
    {
        storage.clone().map(present)
    }

    fn is_present(storage: &Self::Storage) -> bool {
        storage.is_some()
    }
}

impl<T> DefaultValueStorage<T> for RequiredValueStorage {
    fn default_storage() -> Self::Storage {
        None
    }
}

impl<T> ValueStorage<T> for DirectValueStorage {
    type Storage = T;

    fn present(value: T) -> Self::Storage {
        value
    }

    fn present_unless_default(value: T, _default: T) -> Self::Storage
    where
        T: PartialEq,
    {
        value
    }

    fn map_into_value<Output, Present, Missing>(
        storage: Self::Storage,
        present: Present,
        _missing: Missing,
    ) -> Output
    where
        Present: FnOnce(T) -> Output,
        Missing: FnOnce() -> Output,
    {
        present(storage)
    }

    fn try_map_into_value<Output, Error, Present>(
        storage: Self::Storage,
        present: Present,
        _missing: Error,
    ) -> Result<Output, Error>
    where
        Present: FnOnce(T) -> Output,
    {
        Ok(present(storage))
    }

    fn map_present_cloned<Output, Present>(
        storage: &Self::Storage,
        present: Present,
    ) -> Option<Output>
    where
        T: Clone,
        Present: FnOnce(T) -> Output,
    {
        Some(present(storage.clone()))
    }

    fn is_present(_storage: &Self::Storage) -> bool {
        true
    }
}

impl<T: Default> DefaultValueStorage<T> for DirectValueStorage {
    fn default_storage() -> Self::Storage {
        T::default()
    }
}

impl<T> InfallibleValueStorage<T> for DirectValueStorage {
    fn map_into_value<Output, Present>(storage: Self::Storage, present: Present) -> Output
    where
        Present: FnOnce(T) -> Output,
    {
        present(storage)
    }
}

#[cfg(test)]
mod tests {
    use super::{ComponentPrototyping, is_valid_component_field_suffix};

    #[test]
    fn component_field_suffix_validator_accepts_identifier_suffixes() {
        assert!(is_valid_component_field_suffix("input"));
        assert!(is_valid_component_field_suffix("_input"));
        assert!(is_valid_component_field_suffix("input_2"));
    }

    #[test]
    fn component_field_suffix_validator_rejects_non_identifier_suffixes() {
        assert!(!is_valid_component_field_suffix(""));
        assert!(!is_valid_component_field_suffix("_"));
        assert!(!is_valid_component_field_suffix("2input"));
        assert!(!is_valid_component_field_suffix("input-field"));
    }

    #[test]
    #[should_panic(expected = "component suffix must be a non-empty ASCII identifier suffix")]
    fn component_prototyping_rejects_invalid_field_suffixes() {
        let _ = ComponentPrototyping::new().field_suffix("input-field");
    }
}
