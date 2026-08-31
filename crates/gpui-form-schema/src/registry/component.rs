use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageCapability {
    OptionalValue,
    RequiredValue,
    DirectValue,
    ShapePolicy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ComponentCapabilities {
    shape: ShapeComponentCapabilities,
    storage: StorageCapability,
}

impl ComponentCapabilities {
    pub const fn new() -> Self {
        Self {
            shape: ShapeComponentCapabilities::new(),
            storage: StorageCapability::ShapePolicy,
        }
    }

    pub const fn with_render(mut self, render: RenderCapability) -> Self {
        self.shape = self.shape.with_render(render);
        self
    }

    pub const fn with_value_binding(mut self, value_binding: ValueBindingCapability) -> Self {
        self.shape = self.shape.with_value_binding(value_binding);
        self
    }

    pub const fn with_storage(mut self, storage: StorageCapability) -> Self {
        self.storage = storage;
        self
    }

    pub const fn render(self) -> RenderCapability {
        self.shape.render()
    }

    pub const fn value_binding(self) -> ValueBindingCapability {
        self.shape.value_binding()
    }

    pub const fn storage(self) -> StorageCapability {
        self.storage
    }

    const fn shape(self) -> ShapeComponentCapabilities {
        self.shape
    }

    pub const fn render_component(self) -> bool {
        self.shape.render_component()
    }

    pub const fn value_binding_enabled(self) -> bool {
        self.shape.value_binding_enabled()
    }
}

impl Default for ComponentCapabilities {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FieldComponentVariant {
    pub(super) shape_use: ComponentShapeUse,
    pub(super) storage: StorageCapability,
}

impl FieldComponentVariant {
    pub const fn new(shape_path: RustPath) -> Self {
        Self {
            shape_use: ComponentShapeUse::new(ComponentFieldName::new(""), shape_path),
            storage: StorageCapability::ShapePolicy,
        }
    }

    /// Attach the complete component capabilities metadata for this field.
    pub const fn with_capabilities(mut self, capabilities: ComponentCapabilities) -> Self {
        self.shape_use = self.shape_use.with_capabilities(capabilities.shape());
        self.storage = capabilities.storage();
        self
    }

    /// Mark that this component shape publishes render metadata for generated
    /// prototyping code.
    pub const fn with_render_component(mut self, enabled: bool) -> Self {
        let capabilities = self.shape_use.capabilities().with_render(if enabled {
            RenderCapability::Component
        } else {
            RenderCapability::None
        });
        self.shape_use = self.shape_use.with_capabilities(capabilities);
        self
    }

    /// Marks this component shape as value-bound for generated prototyping code.
    pub const fn with_value_binding(mut self, enabled: bool) -> Self {
        let capabilities = self
            .shape_use
            .capabilities()
            .with_value_binding(if enabled {
                ValueBindingCapability::Inherited
            } else {
                ValueBindingCapability::None
            });
        self.shape_use = self.shape_use.with_capabilities(capabilities);
        self
    }

    /// Attach the value-holder storage capability inferred for this field.
    pub const fn with_storage_capability(mut self, storage: StorageCapability) -> Self {
        self.storage = storage;
        self
    }

    /// Attach the component shape's preferred prototyping field suffix.
    pub const fn with_prototyping_field_suffix(mut self, suffix: Option<ComponentSuffix>) -> Self {
        self.shape_use = self.shape_use.with_prototyping(ShapeComponentPrototyping {
            field_suffix: suffix,
        });
        self
    }

    pub(super) const fn with_field_metadata(
        mut self,
        field_name: ComponentFieldName<'static>,
        field_type: RustType,
    ) -> Self {
        self.shape_use = ComponentShapeUse::new(field_name, self.shape_use.shape_path())
            .with_field_type(field_type)
            .with_capabilities(self.shape_use.capabilities())
            .with_prototyping(self.shape_use.prototyping());
        self
    }

    /// Returns the serialized component shape path.
    ///
    /// Tooling that needs a parsed Rust path should prefer
    /// `gpui_form_schema::resolved::ResolvedComponentMetadata::shape_path`.
    pub const fn shape_path(&self) -> RustPath {
        self.shape_use.shape_path()
    }

    pub const fn capabilities(&self) -> ComponentCapabilities {
        ComponentCapabilities::new()
            .with_render(self.shape_use.capabilities().render())
            .with_value_binding(self.shape_use.capabilities().value_binding())
            .with_storage(self.storage)
    }

    pub const fn render_component(&self) -> bool {
        self.capabilities().render_component()
    }

    pub const fn value_binding(&self) -> bool {
        self.capabilities().value_binding_enabled()
    }

    pub const fn prototyping_field_suffix(&self) -> Option<ComponentSuffix> {
        self.shape_use.prototyping().field_suffix
    }
}
