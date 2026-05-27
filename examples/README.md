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
- component shape state, local shapes, and cross-crate shapes
- Koruma validation wiring
- newtype-backed numeric validation
- `cfg_attr`-gated derive usage
- empty forms and date conversion

## some-lib-component-shapes

External component shape state and UI types used by the example forms.

This crate demonstrates the cross-crate component-shape workflow.

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
form files into `examples/prototyping/output`, then formats them with
`rustfmt`. Generated Storybook form titles use the example app's active locale.
Value-bound shape-backed fields keep readable prototyping metadata names such as
`email_input` and `on_email_input_event`.

Run it with:

```sh
cargo run -p prototyping
```

## i18n

Shared localization assets used by the example crates. Story apps own a small
`i18n` helper around `EmbeddedI18n`; generated/story rendering calls that helper
to pass localized strings explicitly.
