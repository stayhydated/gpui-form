# gpui-form-codegen Architecture

`gpui-form-codegen` is the parse-time component-contract and token-generation
layer used by `gpui-form-derive`.

## Purpose

This crate exists to:

1. parse `component = Shape` expressions into a typed internal model
1. emit generated `FormFields` and `FormComponents` tokens from a
   `ComponentShape`
1. emit schema metadata aligned with the same component-shape contract

It should stay proc-macro-adjacent rather than becoming a second proc-macro
crate.

## Modules

- `src/lib.rs`: exports the component model
- `src/components.rs`: parses component shape options and emits metadata
- `src/names.rs`: helper naming utilities for generated identifiers
- `src/implementations/shape.rs`: `ComponentLayout` implementation for shape-backed
  component shapes

## Parse-Time Component Model

`components.rs` accepts component shape annotations:

- `component = my::Shape`
- `component = my::Shape.component(my::Widget)`
- `component = my::Shape.value_binding()`
- `component = my::Shape.field_suffix("input")`
- `component = gpui_form_collection::select::Select::<_>::searchable(true).partial(true)`
- `component = gpui_form_component::infinite_select::InfiniteSelect::<_>::searchable(true).max_depth(3)`
Important parse-time responsibilities:
- expression syntax uses a shape path plus optional component metadata or
  behavior setters
- generic expression paths may use `_` with turbofish syntax, such as
  `gpui_form_collection::input::Input::<_>`
- `_` is resolved to the field's form-side type, including any
  `#[gpui_form(type = ...)]` override
- required-value holder behavior is inferred from known shape metadata
- `value_binding()` records that generated prototyping code should use
  `ComponentValueBinding`
- `field_suffix("...")` records a field-level prototyping name override
- `searchable(true)` and `.partial(true)` record select behavior metadata
- `searchable(true)` and `.max_depth(...)` record infinite-select behavior metadata
- direct behavior setter chains are expanded to bon builder checks for
  rust-analyzer without requiring users to write `builder()` in attributes

## Component Layout Emission

The shape layout emits two things:

- a `FormFields` entry with `Entity<<Shape as ComponentShape>::State>`
- a `FormComponents` constructor that delegates to
  `<Shape as ComponentShape>::new(window, cx)`

Generated identifiers use an explicit field-level `field_suffix` first, then
shape-level `ComponentShape::PROTOTYPING.field_suffix`, then the resolved
component shape's final segment. The shape-name fallback strips `Shape` or
`State`, removes a duplicate field prefix, and falls back to `shape` when the
shape name is exactly the field name. Explicit suffix metadata goes through the
same field-name normalization. For example, `email: EmailInputShape` becomes
`email_input`, while `tags: TagsState` falls back to `tags_shape`.

No `gpui-component` UI component is hard-coded here. Reusable gpui-component-backed
representations live in `gpui-form-collection`; application-specific widgets
can define local shapes.

## Metadata Emission

Inventory/prototyping metadata records:

- the inferred `ComponentsBehaviour` for known reusable shapes, including
  select and infinite-select options
- the resolved component shape path
- the optional render component path
- the value-binding flag
- the optional component-shape prototyping field suffix

`gpui-form-prototyping-core` consumes this metadata through the same contract,
so adding a new widget family does not require changing this crate.
