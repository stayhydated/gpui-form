# Examples

This directory is the canonical index for runnable `gpui-form` workspace
examples.

## some-lib

Source structs and enums that demonstrate the supported derive surface:

- basic input, number input, checkbox, switch, select, combobox, slider,
  color-picker, date-picker, file-picker, and otp input fields
- select, date-picker conversion, and skipped-field workflows
- nested `InfiniteSelect` enums, including index-path and key-path round trips
  plus typed path-error reporting, custom persisted keys, and key-path string
  serialization
- component-shape-backed collection and runtime fields
- Koruma validation wiring
- newtype-backed numeric validation
- `cfg_attr`-gated derive usage
- empty forms and date conversion

## some-lib-forms

Storybook-style GPUI app that renders generated forms around the example types.
The checked-in `location_form` example shows the runtime-owned
`Select` flow with `form_fields()` instead of manual child-select
rebuilding.

Run it with:

```sh
cargo run -p some-lib-forms
```

## gpui-form-component-story

Storybook-style GPUI app for the reusable runtime components themselves:
infinite select, date picker, and file picker.

Run it with:

```sh
cargo run -p gpui-form-component-story
```

## prototyping

Generator example that walks `GpuiFormShape` inventory data and emits scaffolded
form files into `examples/some-lib-forms/src/forms` and mirrors the generated
output under `examples/prototyping/output`, then formats both with `rustfmt`.
Generated Storybook form titles use the example app's active locale. Component
entity fields use source field names such as `email`, while prototyping helper
names keep readable suffixes such as `on_email_input_event`. Holder conversion
debug rows use the derive-emitted inventory flag for `into_original` versus
`try_into_original`.

Run it with:

```sh
cargo run -p prototyping
```

## i18n

Shared localization assets used by the example crates. The example crates keep
small `i18n` modules for embedded asset registration and startup, while GPUI
story rendering calls `gpui_es_fluent` app-global helpers for localized labels
and messages.
