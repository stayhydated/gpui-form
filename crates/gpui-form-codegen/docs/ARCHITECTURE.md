# gpui-form-codegen Architecture

`gpui-form-codegen` is the parse-time component-contract and token-generation
layer used by `gpui-form-derive`.

## Purpose

This crate exists to:

1. parse component shape expressions into a typed internal model
1. emit generated `FormFields` and `FormComponents` tokens from a
   `ComponentShape`
1. emit schema metadata aligned with the same component-shape contract

It should stay proc-macro-adjacent rather than becoming a second proc-macro
crate.

## Modules

- `src/lib.rs`: exports the component model
- `src/crate_paths.rs`: resolves generated crate paths, including renamed
  downstream dependencies
- `src/components.rs`: parses component shape options and emits metadata
- `src/names.rs`: helper naming utilities for generated identifiers
- `src/implementations/shape.rs`: `ComponentLayout` implementation for shape-backed
  component shapes

## Parse-Time Component Model

`components.rs` models component shape expressions accepted by the derive:

- `#[gpui_form(my::Shape)]`
- `#[gpui_form(my::Shape.component(my::Widget))]`
- `#[gpui_form(my::Shape.field_suffix("input"))]`

Important parse-time responsibilities:

- field shape syntax reaches this crate as a parsed shape path plus optional
  generic component metadata calls
- generic expression paths may use `_` with turbofish syntax, such as
  `gpui_form_collection::input::Input::<_>`
- `_` is resolved to the field's form-side type, including any
  `#[gpui_form(type = ...)]` override
- `component(...)` records render-component metadata for prototyping output and
  must be path-like
- component metadata method names are parsed into a small typed enum before
  argument validation, so diagnostics and duplicate checks share one supported
  method list
- generated type checks assert `ComponentShapeFor<Value>` so shape crates own
  component-specific value compatibility diagnostics
- non-optional shape-backed fields inherit the shape's required-value policy by
  default
- shape-level `ValueBindingPolicy` records whether generated prototyping code
  should use `ComponentValueBinding`
- `field_suffix("...")` records a field-level prototyping name override and
  must be a non-empty identifier suffix

Component-specific settings and value compatibility rules belong inside the
shape's `ComponentShape::new` implementation, `ComponentShapeFor<Value>` impls,
or a dedicated wrapper shape. This crate does not know about selects, inputs,
date pickers, or any other component family.

## Component Layout Emission

The shape layout emits two things:

- a `FormFields` entry with `Entity<<Shape as ComponentShape>::State>`
- a `FormComponents` constructor that delegates to
  `<Shape as ComponentShape>::new(window, cx)`

For `GpuiForm` output, emitted runtime paths target
`gpui_form::runtime::shape` through the facade dependency. Downstream crates
that use component-backed fields depend on `gpui-form` plus the crate that owns
the concrete shape type; they do not need a direct `gpui-form-runtime`
dependency for generated field code.

Generated identifiers and derive-emitted inventory suffix metadata share one
resolution path: an explicit field-level `field_suffix` first, then an explicit
component type, then the resolved component shape's final segment. The path
fallback strips `Shape` or `State`, removes a duplicate field prefix, and falls
back to `shape` when the suffix exactly matches the field name. Explicit suffix
metadata goes through the same field-name normalization. For example,
`birth_date: DatePicker` becomes `birth_date_date_picker`, while
`tags: TagsState` falls back to `tags_shape`.

No `gpui-component` UI component is hard-coded here. Reusable gpui-component-backed
representations live in `gpui-form-collection`; application-specific widgets
can define local shapes.

## Metadata Emission

Inventory/prototyping metadata records:

- explicit component field metadata via `FieldVariant::component(...)`
- the field value-presence policy as `FieldValuePresence`
- the resolved component shape path in `FieldComponentVariant`
- the optional render component type
- the value-binding flag
- the resolved component field suffix used by generated form fields

`gpui-form-prototyping-core` consumes this metadata through the same contract,
so adding a new widget family does not require changing this crate.
