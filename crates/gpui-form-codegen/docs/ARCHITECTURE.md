# gpui-form-codegen Architecture

`gpui-form-codegen` is the parse-time component-contract and token-generation
layer used by `gpui-form-derive`.

## Purpose

This crate exists to:

1. parse component shape paths and configured builder expressions into a typed internal model
1. plan semantic component field facts from a declared GPUI component shape
1. emit schema metadata aligned with the same component-shape contract

It should stay proc-macro-adjacent rather than becoming a second proc-macro
crate.

## Modules

- `src/lib.rs`: exports the component model
- `src/crate_paths.rs`: shared proc-macro crate path resolver used by
  `gpui-form-derive`, `gpui-form-component-derive`, and
  `gpui-form-collection-derive`, including renamed downstream dependencies
- `src/components.rs`: parses component shape options, plans
  `ComponentFieldIr`, and emits component metadata/type-check tokens
- `src/names.rs`: helper naming utilities for generated identifiers

## Parse-Time Component Model

`components.rs` models component shape metadata accepted by the derive:

- `#[gpui_form(component(my::Shape))]`
- `#[gpui_form(component(my::Shape::configured(...)))]`

Important parse-time responsibilities:

- field shape syntax reaches this crate as a plain shape path or as a
  configured expression rooted at a shape path
- generic expressions may use `_` with turbofish syntax, such as
  `gpui_form_collection::input::Input::<_>` or
  `gpui_form_collection::select::Select::<_>::searchable(true)`. Completed
  config values can be passed through a shape-owned `from(...)` constructor.
- `_` is resolved once during derive planning to a `ResolvedComponentShape`
  using the field's form-side type, including any intent-scoped
  `value(type = ...)` override
- generated type checks use field-named assertions for declared-shape,
  supported-value-type, and value-binding checks through
  `gpui_form::runtime::shape` contracts so diagnostics point at the field
  component shape while preserving component-specific trait messages
- non-optional shape-backed fields inherit the shape's value-storage policy by
  default
- shape metadata records whether generated prototyping code
  should use the value-binding contract bridged from
  `component-shape-gpui`
- shape-level `PROTOTYPING.field_suffix` records reusable prototyping suffix
  metadata and must be a non-empty identifier suffix

Component-specific settings and supported value-type rules belong inside the
shape's `GpuiComponentShape::new` implementation,
`GpuiComponentShapeBuilder<Shape>` impls, `GpuiComponentShapeFor<Value>` impls,
or a dedicated wrapper shape. Form storage policy remains in
`gpui-form-runtime`. This crate does not know about selects, inputs, date
pickers, or any other component family.

## Component Field IR

`ResolvedComponentShape::field_ir()` computes semantic facts needed by
`gpui-form-derive` to emit component-backed fields:

- the generated component field identifier
- the resolved component shape path and field value type
- optional configured constructor expression with `_` resolved to the same
  field value type
- the component shape storage policy, represented as a shape-owned
  value-storage policy projection

`gpui-form-codegen` does not emit `FormFields` or `FormComponents` layout token
fragments. `gpui-form-derive` renders those final declarations from
`ComponentFieldIr`, keeping layout emission at the derive boundary while
sharing the same resolved component facts with inventory metadata and type
checks. Non-component holder storage, such as hidden direct fields, is selected
inside derive planning rather than crossing the codegen component boundary.

For `GpuiForm` output, rendered runtime paths target
`gpui_form::runtime::shape` through the facade dependency. Downstream crates
that use component-backed fields depend on `gpui-form` plus the crate that owns
the concrete shape type; they do not need a direct `gpui-form-runtime`
dependency for generated field code.

Generated component entity identifiers use the source field name. Prototyping
helper suffixes derive from the planned `ResolvedComponentShape`. The
path fallback strips `Shape` or `State`, removes a duplicate field prefix, and
falls back to `shape` when the suffix exactly matches the field name. For
example, `birth_date: DatePicker` keeps a `birth_date` entity field while
publishing a `date_picker` helper suffix; `tags: TagsState` falls back to the
`shape` helper suffix. Derive-emitted inventory suffix metadata reads
`component_shape::ComponentShapeMetadata::PROTOTYPING.field_suffix` when the
declared shape publishes a suffix and otherwise uses the same path fallback.

No `gpui-component` UI component is hard-coded here. Reusable
gpui-component-backed representations live in `gpui-form-collection` and are
declared through `component-shape-gpui`; application-specific widgets can define
local shapes the same way.

## Metadata Emission

Inventory/prototyping metadata records:

- explicit component field metadata via `FieldValueSpec` and
  `FieldVariant::component(...)`
- the field value-presence policy as `FieldValuePresence`
- the resolved component shape path in `FieldComponentVariant`
- whether the component shape publishes a `RenderComponent` contract
- the value-binding flag
- the shape-level prototyping suffix, or the resolved component field suffix

`gpui-form-prototyping-core` consumes this metadata through the same contract,
so adding a new widget family does not require changing this crate.

`metadata.rs` also owns shared token-level lowering helpers for defaults and
value conversions used by both `gpui-form-derive` and
`gpui-form-prototyping-core`. Literal defaults, including literals wrapped in
simple groups, parentheses, or one-expression blocks, lower through
`Into::into(...)`; non-literal defaults are emitted as written. Conversion
helpers apply user-authored `from_source` and `into_source` expressions while
preserving the span supplied by the caller.
