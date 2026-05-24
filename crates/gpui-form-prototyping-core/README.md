# gpui-form-prototyping-core

Scaffolding utilities built on top of `GpuiFormShape` inventory data.

Use this crate when you want to generate GPUI form code from the metadata
emitted by `#[derive(GpuiForm)]` instead of wiring forms by hand.

Most application code should still start with
[`gpui-form`](../gpui-form/README.md).

## Quick Example

```rs
use gpui_form::schema::registry::{GpuiFormShape, inventory};
use gpui_form_prototyping_core::FormShapeAdapter;

for shape in inventory::iter::<GpuiFormShape>() {
    let parts = FormShapeAdapter::new(shape)
        .parts()
        .expect("shape metadata should be valid");

    let _imports = parts.imports;
}
```

## Main API

- `FormShapeAdapter::parts()` returns validated identifiers, imports, and form
  fragments for one shape
- `FormShapeAdapter::generate_file(&impl FormLayout)` renders a full file with
  your chosen layout
- `FormLayout` lets callers define the overall file structure
- `PrototypingError` reports malformed metadata without panicking

## Example Workflow

The workspace example in [`examples/prototyping`](../../examples/prototyping)
shows the normal flow:

1. enable `gpui-form`'s `inventory` feature
1. iterate `inventory::iter::<GpuiFormShape>()`
1. adapt each shape with `FormShapeAdapter`
1. render a file through a custom `FormLayout`
1. clear stale generated modules and write the generated form files
1. run `rustfmt` over the written files before treating the scaffold as ready

When the layout emits `gpui_storybook::Story`, pass the `cx: &gpui::App`
provided by `Story::title` into the application i18n helper so generated form
titles follow the active Storybook locale.

Generated infinite-select and file-picker fields use the same runtime helpers
that hand-written forms use. Generated text inputs use the form-side
`FieldVariant::value_type` and parse non-`String` values with `FromStr` instead
of assuming every text field stores `String`.

Custom fields remain inert by default. If a field's shape opts into
`value_binding`, the adapter emits generic seed and subscription hooks through
`gpui_form_component::custom::CustomComponentValueBinding<T>`. Generated
scaffolds use `CustomComponentShape::PROTOTYPING.field_suffix` when available
for field and handler suffixes, such as `email_input` and
`on_email_input_event`, then fall back to the custom shape name heuristic.
Value-bound scaffolds use `seed_value_binding_state`, `form_value_change`,
`FormValueChange<T>`, and runtime aliases for state and actual component event
projections so the output stays readable.

## Feature Flags

- `fluent`: use `es-fluent` keys for generated labels, descriptions, and
  validation messages through an application helper named
  `crate::i18n::localize(...)`

## Most Users Should Use Instead

- [`gpui-form`](../gpui-form/README.md) for hand-written forms plus derives
- [`gpui-form-schema`](../gpui-form-schema/README.md) if you only need the
  metadata layer
