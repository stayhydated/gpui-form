# gpui-form-codegen Architecture

`gpui-form-codegen` is the parse-time component-contract and token-generation
layer used by `gpui-form-derive`.

## Purpose

This crate exists to:

1. parse `component(custom(...))` options into a typed internal model
1. emit generated `FormFields` and `FormComponents` tokens from a
   `CustomComponentShape`
1. emit schema metadata aligned with the same custom-component contract

It should stay proc-macro-adjacent rather than becoming a second proc-macro
crate.

## Modules

- `src/lib.rs`: exports the component model
- `src/components.rs`: parses custom component options and emits metadata
- `src/names.rs`: helper naming utilities for generated identifiers
- `src/implementations/custom.rs`: `ComponentLayout` implementation for custom
  component shapes

## Parse-Time Component Model

`components.rs` accepts only custom component annotations:

- `component(custom(shape = my::Shape))`
- `component(custom(state = my::State))`
- `component(custom(shape = my::Shape, component = my::Widget))`
- `component(custom(shape = my::Shape, wraps_in_option = false))`
- `component(custom(shape = my::Shape, value_binding))`

Important parse-time responsibilities:

- exactly one of `shape = ...` or `state = ...` must be present
- generic shape paths may use `_` in type arguments, usually through a string
  literal such as `"gpui_form_collection::input::InputShape<_>"`
- `_` is resolved to the field's form-side type, including any
  `#[gpui_form(type = ...)]` override
- `wraps_in_option` controls generated value-holder storage
- `value_binding` records that generated prototyping code should use
  `CustomComponentValueAdapter`

## Component Layout Emission

The custom layout emits two things:

- a `FormFields` entry with `Entity<<Shape as CustomComponentShape>::State>`
- a `FormComponents` constructor that delegates to
  `<Shape as CustomComponentShape>::new(window, cx)`

No `gpui-component` UI shape is hard-coded here. Reusable gpui-component-backed
representations live in `gpui-form-collection`; application-specific widgets
can define local shapes.

## Metadata Emission

Inventory/prototyping metadata records:

- `ComponentsBehaviour::Custom`
- the resolved custom shape path
- the optional render component path
- the custom value-binding flag

`gpui-form-prototyping-core` consumes this metadata through the same contract,
so adding a new widget family does not require changing this crate.
