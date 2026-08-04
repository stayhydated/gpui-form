---
name: use-gpui-form-component-shapes
description: "Use when adding, reviewing, or refactoring gpui-form integration for an application-owned or third-party GPUI component shape: #[gpui_form(component(...))], shape/value compatibility, gpui-form-runtime storage policy, configured builders, generated holder behavior, MCP input metadata, or inventory prototyping. Use use-gpui-form for existing built-in shapes."
---

# Use GPUI Form Component Shapes

## Scope

Apply this skill after deciding that an existing `gpui-form-collection` or
`gpui-form-component` shape is insufficient, or when integrating an existing
custom shape into `#[derive(GpuiForm)]`.

Use `use-gpui-form` for ordinary forms with built-in shapes. Use
`use-component-shape` for framework-neutral metadata and
`use-component-shape-gpui` for shape declaration, rendering, construction, and
value-binding syntax.

## Workflow

1. Confirm the field's source type, form-side type, optionality, and desired
   empty-value behavior.
2. Reuse an existing shape when it supports that contract.
3. For a new shape, declare it through
   `#[derive(component_shape_gpui::GpuiComponentShape)]` or
   `component_shape_gpui::component_shape!` using the component-shape skills.
4. Publish compatibility with the form-side value through declared value
   metadata, `GpuiComponentShapeFor<Value>`, or shape value binding.
5. Implement `GpuiFormComponentShapePolicy` and choose direct or required
   holder storage.
6. Add a `GpuiComponentShapeBuilder<Shape>` only when individual fields need
   construction options.
7. Publish stable prototyping and MCP metadata when those workflows consume
   the shape.
8. Use the base shape or configured builder in `component(...)` and check the
   generated holder contract.

## Dependency boundary

Applications that consume a shape use `gpui-form` plus the crate that owns the
shape. They normally access runtime contracts through
`gpui_form::runtime::shape`.

A reusable integration crate depends directly on `gpui-form-runtime` when it
implements form storage policy or lower-level value binding.

## Field integration

```rust
#[derive(Clone, Debug, Default, gpui_form::GpuiForm)]
pub struct ContactForm {
    #[gpui_form(component(EmailInputShape))]
    pub email: String,
}
```

The shape must carry the declaration marker emitted by the GPUI component-shape
derive or macro. A hand-written construction trait implementation alone is not
a complete form-shape declaration.

## Choose storage

```rust
impl gpui_form_runtime::shape::GpuiFormComponentShapePolicy for EmailInputShape {
    type ValueStoragePolicy =
        gpui_form_runtime::shape::DirectValueStorage;
}
```

- Use `DirectValueStorage` when a required field always stores `T`. Provide an
  intent-scoped default or require form-side `T: Default`.
- Use `RequiredValueStorage` when the holder must represent missing input as
  `Option<T>`. Conversion is fallible while the value is absent, and generated
  Koruma validation reports the missing field.

Value compatibility and storage are separate decisions. Compatibility answers
whether the shape can edit `T`; storage answers how a non-optional source field
represents an empty form value.

## Configure construction

Implement `GpuiComponentShapeBuilder<Shape>` for a configuration type when one
shape needs field-local options. Expose a shape helper that returns the builder,
then use dot-chain syntax:

```rust
#[gpui_form(component(AssigneeSelect.searchable(true)))]
pub assignee: Option<UserId>,
```

The builder changes construction only. Compatibility, rendering, storage,
value binding, MCP metadata, and prototyping metadata remain properties of the
base shape.

## Align metadata

For generated subscriptions, the shape needs value-binding capability and an
event-to-`ValueChange<T>` contract. For inventory scaffolding, publish a stable
non-empty ASCII field suffix plus render, path, binding, and storage metadata.

MCP form schemas start from the field type's `McpToolValue` implementation.
Attach shape-specific MCP input metadata when the widget narrows or explains
that wire contract. Multi-value shapes publish metadata for each supported
value through the generated `ComponentShapeFor<Value>` implementation.

Fix missing metadata at the shape declaration. Do not compensate with
placeholder generator output or per-form compatibility wrappers.

## Routing

- Existing collection/component shape: `use-gpui-form`
- Generic component metadata and suffixes: `use-component-shape`
- GPUI declaration, rendering, builders, and value binding:
  `use-component-shape-gpui`
- Form intents, holders, validation, inventory use, and MCP submission:
  `use-gpui-form`
