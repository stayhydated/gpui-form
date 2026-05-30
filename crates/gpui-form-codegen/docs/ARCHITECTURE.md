# gpui-form-codegen Architecture

`gpui-form-codegen` is the parse-time component-contract and token-generation
layer used by `gpui-form-derive`.

## Purpose

This crate exists to:

1. parse component shape paths into a typed internal model
1. emit generated `FormFields` and `FormComponents` tokens from a
   `ComponentShape`
1. emit schema metadata aligned with the same component-shape contract

It should stay proc-macro-adjacent rather than becoming a second proc-macro
crate.

## Modules

- `src/lib.rs`: exports the component model
- `src/crate_paths.rs`: shared proc-macro crate path resolver used by
  `gpui-form-derive`, `gpui-form-component-derive`, and
  `gpui-form-collection-derive`, including renamed downstream dependencies
- `src/components.rs`: parses component shape options and emits metadata
- `src/names.rs`: helper naming utilities for generated identifiers
- `src/implementations/shape.rs`: `ComponentLayout` implementation for shape-backed
  component shapes

## Parse-Time Component Model

`components.rs` models component shape metadata accepted by the derive:

- `#[gpui_form(component(my::Shape))]`

Important parse-time responsibilities:

- field shape syntax reaches this crate as a parsed shape path
- generic expression paths may use `_` with turbofish syntax, such as
  `gpui_form_collection::input::Input::<_>`
- `_` is resolved once during derive planning to a `ResolvedComponentShape`
  using the field's form-side type, including any intent-scoped
  `value(type = ...)` override
- generated type checks use field-named assertions for declared-shape,
  value-compatibility, and value-binding checks so diagnostics point at the
  field component shape while preserving component-specific trait messages
- non-optional shape-backed fields inherit the shape's value-storage policy by
  default
- shape-level `ValueBindingPolicy` records whether generated prototyping code
  should use `ComponentValueBinding`
- shape-level `PROTOTYPING.field_suffix` records reusable prototyping suffix
  metadata and must be a non-empty identifier suffix

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

Generated identifiers derive their suffix from the planned
`ResolvedComponentShape`. The path fallback strips `Shape` or `State`, removes
a duplicate field prefix, and falls back to `shape` when the suffix exactly
matches the field name. For example, `birth_date: DatePicker` becomes
`birth_date_date_picker`, while `tags: TagsState` falls back to `tags_shape`.
Derive-emitted inventory suffix metadata reads
`ComponentShape::PROTOTYPING.field_suffix` when the declared shape publishes a
suffix and otherwise uses the same path fallback.

No `gpui-component` UI component is hard-coded here. Reusable gpui-component-backed
representations live in `gpui-form-collection`; application-specific widgets
can define local shapes.

## Metadata Emission

Inventory/prototyping metadata records:

- explicit component field metadata via `FieldVariant::builder(...).component(...)`
- the field value-presence policy as `FieldValuePresence`
- the resolved component shape path in `FieldComponentVariant`
- whether the component shape publishes a `RenderComponent` contract
- the value-binding flag
- the shape-level prototyping suffix, or the resolved component field suffix

`gpui-form-prototyping-core` consumes this metadata through the same contract,
so adding a new widget family does not require changing this crate.
